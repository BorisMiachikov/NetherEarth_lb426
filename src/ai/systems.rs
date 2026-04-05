use bevy::prelude::*;

use crate::{
    command::command::RobotCommand,
    core::Team,
    movement::velocity::MovementTarget,
    robot::{
        components::{Nuclear, RobotMarker},
        registry::ModuleRegistry,
    },
    structure::{
        capture::Capturable,
        warbase::{ProductionQueue, Warbase},
    },
};

use super::{
    scoring::{capture_priority, select_blueprint, select_nuclear_blueprint, threat_ratio},
    state::{AICommander, GameOutcome, GameResult},
};

// ── Победа / поражение ───────────────────────────────────────────────────────

/// Проверяет уничтожение варбейсов → устанавливает GameResult.
pub fn check_victory_defeat(
    warbases: Query<&Team, With<Warbase>>,
    mut result: ResMut<GameResult>,
    game_time: Res<crate::core::time::GameTime>,
    factories: Query<&Team, With<crate::structure::factory::Factory>>,
) {
    if result.outcome.is_some() {
        return; // Уже определён итог
    }

    let player_alive = warbases.iter().any(|t| *t == Team::Player);
    let enemy_alive = warbases.iter().any(|t| *t == Team::Enemy);

    if !player_alive || !enemy_alive {
        let player_factories = factories
            .iter()
            .filter(|t| **t == Team::Player)
            .count() as u32;
        let enemy_factories = factories
            .iter()
            .filter(|t| **t == Team::Enemy)
            .count() as u32;

        result.outcome = Some(if !enemy_alive {
            GameOutcome::PlayerWin
        } else {
            GameOutcome::PlayerLose
        });
        result.game_days = game_time.game_day;
        result.player_factories = player_factories;
        result.enemy_factories = enemy_factories;

        info!(
            "Игра окончена: {:?} на день {}",
            result.outcome, game_time.game_day
        );
    }
}

// ── Постройка роботов ────────────────────────────────────────────────────────

/// Периодически добавляет роботов в очередь постройки вражеского варбейса.
pub fn ai_build_robots(
    time: Res<Time>,
    mut ai: ResMut<AICommander>,
    mut warbases: Query<(&mut ProductionQueue, &Transform, &Team), With<Warbase>>,
    registry: Res<ModuleRegistry>,
    factories: Query<&Team, With<crate::structure::factory::Factory>>,
    result: Res<GameResult>,
) {
    if result.outcome.is_some() {
        return;
    }

    ai.build_timer += time.delta_secs();
    if ai.build_timer < ai.config.build_interval {
        return;
    }
    ai.build_timer = 0.0;

    // Считаем захваченные фабрики ИИ
    let ai_factory_count = factories.iter().filter(|t| **t == Team::Enemy).count() as u32;

    // Выбираем blueprint — ядерный при достаточном количестве фабрик и редко
    let use_nuclear = ai_factory_count >= ai.config.nuclear_factory_threshold
        && ai.decision_counter % 7 == 0;

    let blueprint = if use_nuclear {
        select_nuclear_blueprint()
    } else {
        select_blueprint(ai.decision_counter, &registry)
    };
    ai.decision_counter = ai.decision_counter.wrapping_add(1);

    // Проверка валидности blueprint
    if blueprint.validate().is_err() {
        return;
    }
    let build_cost = blueprint.cost(&registry);

    // Найти вражеский варбейс и добавить в очередь
    for (mut queue, _, team) in &mut warbases {
        if *team != Team::Enemy {
            continue;
        }
        // Ограничение очереди: не более 3 одновременно
        if queue.queue.len() >= 3 {
            continue;
        }
        queue.enqueue(blueprint.clone(), build_cost.build_time);
        ai.robots_built += 1;
        break;
    }
}

// ── Назначение приказов роботам ──────────────────────────────────────────────

/// Периодически назначает приказы простаивающим роботам ИИ.
pub fn ai_assign_commands(
    time: Res<Time>,
    mut ai: ResMut<AICommander>,
    mut idle_robots: Query<(Entity, &mut RobotCommand, &Transform), With<RobotMarker>>,
    all_robots: Query<(Entity, &Transform, &Team), With<RobotMarker>>,
    capturable: Query<(Entity, &Transform, &Team), With<Capturable>>,
    warbases: Query<(&Transform, &Team), With<Warbase>>,
    result: Res<GameResult>,
) {
    if result.outcome.is_some() {
        return;
    }

    ai.decision_timer += time.delta_secs();
    if ai.decision_timer < ai.config.decision_interval {
        return;
    }
    ai.decision_timer = 0.0;

    // Позиция вражеского варбейса (для оценки близости фабрик)
    let enemy_warbase_pos = warbases
        .iter()
        .find(|(_, t)| **t == Team::Enemy)
        .map(|(tf, _)| tf.translation)
        .unwrap_or(Vec3::ZERO);

    // Позиция варбейса игрока (цель для агрессии)
    let player_warbase_pos = warbases
        .iter()
        .find(|(_, t)| **t == Team::Player)
        .map(|(tf, _)| tf.translation);

    let enemy_count = all_robots
        .iter()
        .filter(|(_, _, t)| **t == Team::Player)
        .count() as u32;
    let friendly_count = all_robots
        .iter()
        .filter(|(_, _, t)| **t == Team::Enemy)
        .count() as u32;

    let threat = threat_ratio(enemy_count, friendly_count);
    let be_aggressive = threat > ai.config.aggression;

    for (entity, mut cmd, robot_tf) in &mut idle_robots {
        // Пропустить не-Enemy роботов
        let team = all_robots
            .iter()
            .find(|(e, _, _)| *e == entity)
            .map(|(_, _, t)| *t);
        if team != Some(Team::Enemy) {
            continue;
        }
        if !matches!(*cmd, RobotCommand::Idle) {
            continue;
        }

        // 1) Найти ближайшую нейтральную или вражескую фабрику для захвата
        let best_capture = capturable
            .iter()
            .filter(|(_, _, t)| **t != Team::Enemy)
            .max_by(|(_, tf_a, t_a), (_, tf_b, t_b)| {
                let prio_a =
                    capture_priority(tf_a.translation, enemy_warbase_pos, **t_a == Team::Neutral);
                let prio_b =
                    capture_priority(tf_b.translation, enemy_warbase_pos, **t_b == Team::Neutral);
                prio_a.partial_cmp(&prio_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(e, _, _)| e);

        // 2) Принять решение: захватывать или атаковать
        *cmd = if !be_aggressive && best_capture.is_some() {
            RobotCommand::SeekAndCapture(best_capture)
        } else if let Some(warbase_pos) = player_warbase_pos {
            // Агрессия: двигаться к варбейсу игрока
            RobotCommand::SeekAndDestroy(None)
        } else {
            RobotCommand::SeekAndCapture(best_capture)
        };

        // SeekAndDestroy(None) — update_seek_destroy найдёт цель с учётом VisionRange.
        // Если видимых врагов нет — робот будет исследовать карту.
    }
}

// ── Обновление SeekAndDestroy (7.11 + видимость) ────────────────────────────

/// Управляет роботами с приказом SeekAndDestroy:
/// - ищет врагов в радиусе VisionRange
/// - при обнаружении — преследует
/// - при отсутствии видимых врагов — исследует карту (exploration)
pub fn update_seek_destroy(
    mut commands: Commands,
    mut seekers: Query<
        (Entity, &mut RobotCommand, &Transform, &Team, &crate::robot::components::VisionRange),
        With<RobotMarker>,
    >,
    mov_targets: Query<Option<&MovementTarget>, With<RobotMarker>>,
    all_robots: Query<(Entity, &Transform, &Team), With<RobotMarker>>,
    map: Res<crate::map::grid::MapGrid>,
) {
    // Иммутабельный снапшот: только роботы с SeekAndDestroy
    let snapshot: Vec<(Entity, Vec3, Team, f32, Option<Entity>, Option<Vec3>)> = seekers
        .iter()
        .filter_map(|(entity, cmd, tf, team, vision)| {
            let RobotCommand::SeekAndDestroy(tracked) = *cmd else {
                return None;
            };
            let current_mov = mov_targets.get(entity).ok().flatten().map(|m| m.0);
            Some((entity, tf.translation, *team, vision.0, tracked, current_mov))
        })
        .collect();

    for (entity, robot_pos, team, vision_range, tracked_opt, current_mov) in snapshot {
        // Ищем ближайшего видимого врага
        let visible_enemy = all_robots
            .iter()
            .filter(|(e, _, t)| *e != entity && **t != team)
            .filter(|(_, t, _)| robot_pos.distance(t.translation) <= vision_range)
            .min_by_key(|(_, t, _)| (robot_pos.distance(t.translation) * 100.0) as u32)
            .map(|(e, t, _)| (e, t.translation));

        let Ok((_, mut cmd, _, _, _)) = seekers.get_mut(entity) else {
            continue;
        };

        if let Some((target_e, target_pos)) = visible_enemy {
            // Враг виден — преследуем. Обновляем путь только если цель изменилась
            // или значительно сдвинулась (> 3 units), чтобы не гонять A* каждый кадр.
            let needs_update = tracked_opt != Some(target_e)
                || current_mov.map_or(true, |p| p.distance(target_pos) > 3.0);

            if needs_update {
                *cmd = RobotCommand::SeekAndDestroy(Some(target_e));
                commands.entity(entity).insert(MovementTarget(target_pos));
            }
        } else {
            // Врагов не видно — исследуем карту
            let near_target = current_mov
                .map_or(true, |t| robot_pos.xz().distance(t.xz()) < 2.0);

            if near_target {
                let explore = exploration_target(entity, robot_pos, map.width, map.height);
                commands.entity(entity).insert(MovementTarget(explore));
            }
            // Если ещё идём к точке исследования — не прерываем
        }
    }
}

/// Выбирает точку исследования в квадранте, противоположном текущей позиции.
/// Детерминировано по entity id + текущей позиции.
fn exploration_target(entity: Entity, pos: Vec3, map_w: u32, map_h: u32) -> Vec3 {
    use crate::map::grid::CELL_SIZE;

    let half_w = (map_w / 2).max(1);
    let half_h = (map_h / 2).max(1);

    // Противоположный квадрант
    let x_base = if pos.x < half_w as f32 { half_w } else { 0 };
    let z_base = if pos.z < half_h as f32 { half_h } else { 0 };

    // Псевдослучайный сдвиг внутри квадранта
    let seed = (entity.to_bits() as u32)
        .wrapping_mul(2654435761)
        .wrapping_add(pos.x as u32)
        .wrapping_add(pos.z as u32);
    let dx = seed % half_w;
    let dz = seed.wrapping_mul(2246822519) % half_h;

    Vec3::new(
        (x_base + dx) as f32 * CELL_SIZE,
        0.3,
        (z_base + dz) as f32 * CELL_SIZE,
    )
}

// ── Ядерный детонатор (7.8) ──────────────────────────────────────────────────

/// Робот с ядерным зарядом без активного MovementTarget вблизи вражеского варбейса —
/// самоуничтожается, вызывая ядерный взрыв.
pub fn arm_nuclear_on_arrival(
    nuclear_robots: Query<
        (Entity, &Transform, &Nuclear, &Team),
        (With<RobotMarker>, Without<MovementTarget>),
    >,
    warbases: Query<(&Transform, &Team), With<Warbase>>,
    mut commands: Commands,
) {
    for (entity, tf, nuc, robot_team) in &nuclear_robots {
        if nuc.armed {
            continue;
        }
        let near_enemy_warbase = warbases.iter().any(|(wb_tf, wb_team)| {
            *wb_team != *robot_team
                && tf.translation.distance(wb_tf.translation) <= nuc.blast_radius * 1.5
        });

        if near_enemy_warbase {
            info!("Ядерный заряд активирован!");
            // Взвести и уничтожить себя
            let mut armed_nuc = nuc.clone();
            armed_nuc.armed = true;
            commands.entity(entity).insert(armed_nuc);
            commands.trigger(crate::core::events::EntityDamaged {
                entity,
                amount: 9999.0,
                attacker: None,
            });
        }
    }
}

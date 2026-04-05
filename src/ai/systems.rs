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

        // Для SeekAndDestroy сразу назначим MovementTarget к варбейсу
        if matches!(*cmd, RobotCommand::SeekAndDestroy(_)) {
            if let Some(pos) = player_warbase_pos {
                // Ищем ближайшего врага, иначе идём к варбейсу
                let nearest_enemy = all_robots
                    .iter()
                    .filter(|(e, _, t)| *e != entity && **t == Team::Player)
                    .min_by_key(|(_, tf, _)| {
                        (tf.translation.distance(robot_tf.translation) * 100.0) as u32
                    })
                    .map(|(e, tf, _)| (e, tf.translation));

                if let Some((target_e, target_pos)) = nearest_enemy {
                    *cmd = RobotCommand::SeekAndDestroy(Some(target_e));
                    // MovementTarget будет выставлен через update_seek_destroy
                } else {
                    // Нет врагов — двигаться к варбейсу
                    *cmd = RobotCommand::MoveTo(pos);
                }
            }
        }
    }
}

// ── Обновление SeekAndDestroy (7.11) ────────────────────────────────────────

/// Непрерывно проверяет цель SeekAndDestroy: если цель исчезла — найти новую.
pub fn update_seek_destroy(
    mut commands: Commands,
    mut robots: Query<(Entity, &mut RobotCommand, &Transform, &Team), With<RobotMarker>>,
    all_robots: Query<(Entity, &Transform, &Team), With<RobotMarker>>,
) {
    // Собрать Entity → позиция/команда для роботов с SeekAndDestroy
    let updates: Vec<(Entity, Option<(Entity, Vec3)>)> = robots
        .iter()
        .filter_map(|(entity, cmd, tf, team)| {
            let RobotCommand::SeekAndDestroy(target_opt) = &*cmd else {
                return None;
            };

            let target_valid = target_opt
                .map(|t| all_robots.get(t).is_ok())
                .unwrap_or(false);

            if target_valid {
                return None; // Цель жива, ничего не делаем
            }

            // Найти нового врага
            let new_target = all_robots
                .iter()
                .filter(|(e, _, t)| *e != entity && **t != *team)
                .min_by_key(|(_, t, _)| {
                    (t.translation.distance(tf.translation) * 100.0) as u32
                })
                .map(|(e, t, _)| (e, t.translation));

            Some((entity, new_target))
        })
        .collect();

    for (entity, new_target) in updates {
        let Ok((_, mut cmd, _, _)) = robots.get_mut(entity) else {
            continue;
        };
        if let Some((target_e, target_pos)) = new_target {
            *cmd = RobotCommand::SeekAndDestroy(Some(target_e));
            commands.entity(entity).insert(MovementTarget(target_pos));
        } else {
            *cmd = RobotCommand::Idle;
        }
    }
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

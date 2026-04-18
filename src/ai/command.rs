use bevy::prelude::*;

use crate::{
    command::command::RobotCommand,
    core::Team,
    movement::{exploration_target, velocity::MovementTarget},
    robot::components::{Nuclear, RobotMarker, VisionRange},
    structure::{capture::Capturable, warbase::Warbase},
};

use super::{
    scoring::{capture_priority, threat_ratio},
    state::{AICommander, GameResult},
};

/// Периодически назначает приказы простаивающим роботам ИИ.
pub fn ai_assign_commands(
    time: Res<Time>,
    mut ai: ResMut<AICommander>,
    mut idle_robots: Query<(Entity, &mut RobotCommand, &Transform, Option<&Nuclear>), With<RobotMarker>>,
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

    let enemy_warbase_pos = warbases
        .iter()
        .find(|(_, t)| **t == Team::Enemy)
        .map(|(tf, _)| tf.translation)
        .unwrap_or(Vec3::ZERO);

    let enemy_count = all_robots.iter().filter(|(_, _, t)| **t == Team::Player).count() as u32;
    let friendly_count = all_robots.iter().filter(|(_, _, t)| **t == Team::Enemy).count() as u32;

    let be_aggressive = threat_ratio(enemy_count, friendly_count) > ai.config.aggression;

    for (entity, mut cmd, _robot_tf, nuclear) in &mut idle_robots {
        let team = all_robots
            .iter()
            .find(|(e, _, _)| *e == entity)
            .map(|(_, _, t)| *t);
        if team != Some(Team::Enemy) || !matches!(*cmd, RobotCommand::Idle) {
            continue;
        }

        if nuclear.is_some() {
            *cmd = RobotCommand::DestroyEnemyBase(None);
            continue;
        }

        let best_capture = capturable
            .iter()
            .filter(|(_, _, t)| **t != Team::Enemy)
            .max_by(|(_, tf_a, t_a), (_, tf_b, t_b)| {
                let prio_a = capture_priority(tf_a.translation, enemy_warbase_pos, **t_a == Team::Neutral);
                let prio_b = capture_priority(tf_b.translation, enemy_warbase_pos, **t_b == Team::Neutral);
                prio_a.partial_cmp(&prio_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(e, _, _)| e);

        *cmd = if !be_aggressive && best_capture.is_some() {
            RobotCommand::SeekAndCapture(best_capture)
        } else {
            RobotCommand::SeekAndDestroy(None)
        };
    }
}

/// Управляет роботами с приказом SeekAndDestroy:
/// - ищет врагов в радиусе VisionRange
/// - при обнаружении — преследует
/// - при отсутствии видимых врагов — исследует карту
pub fn update_seek_destroy(
    mut commands: Commands,
    mut seekers: Query<
        (Entity, &mut RobotCommand, &Transform, &Team, &VisionRange),
        With<RobotMarker>,
    >,
    mov_targets: Query<Option<&MovementTarget>, With<RobotMarker>>,
    all_robots: Query<(Entity, &Transform, &Team), With<RobotMarker>>,
    map: Res<crate::map::grid::MapGrid>,
) {
    let snapshot: Vec<(Entity, Vec3, Team, f32, Option<Entity>, Option<Vec3>)> = seekers
        .iter()
        .filter_map(|(entity, cmd, tf, team, vision)| {
            let RobotCommand::SeekAndDestroy(tracked) = *cmd else { return None; };
            let current_mov = mov_targets.get(entity).ok().flatten().map(|m| m.0);
            Some((entity, tf.translation, *team, vision.0, tracked, current_mov))
        })
        .collect();

    for (entity, robot_pos, team, vision_range, tracked_opt, current_mov) in snapshot {
        let from_cell = map.world_to_grid(robot_pos);
        let visible_enemy = all_robots
            .iter()
            .filter(|(e, _, t)| *e != entity && **t != team)
            .filter(|(_, t, _)| robot_pos.distance(t.translation) <= vision_range)
            .filter(|(_, t, _)| {
                let to_cell = map.world_to_grid(t.translation);
                match (from_cell, to_cell) {
                    (Some(f), Some(t)) => map.has_line_of_sight(f, t),
                    _ => false,
                }
            })
            .min_by_key(|(_, t, _)| (robot_pos.distance(t.translation) * 100.0) as u32)
            .map(|(e, t, _)| (e, t.translation));

        let Ok((_, mut cmd, _, _, _)) = seekers.get_mut(entity) else { continue; };

        if let Some((target_e, target_pos)) = visible_enemy {
            let needs_update = tracked_opt != Some(target_e)
                || current_mov.map_or(true, |p| p.distance(target_pos) > 3.0);
            if needs_update {
                *cmd = RobotCommand::SeekAndDestroy(Some(target_e));
                commands.entity(entity).try_insert(MovementTarget(target_pos));
            }
        } else {
            let near_target = current_mov.map_or(true, |t| robot_pos.xz().distance(t.xz()) < 2.0);
            if near_target {
                let explore = exploration_target(entity, robot_pos, map.width, map.height);
                commands.entity(entity).try_insert(MovementTarget(explore));
            }
        }
    }
}

/// Направляет роботов с приказом DestroyEnemyBase к вражескому варбейсу.
/// Варбейс должен быть в радиусе видимости — иначе исследует карту.
pub fn seek_destroy_base(
    mut commands: Commands,
    mut robots: Query<
        (Entity, &mut RobotCommand, &Transform, &Team, &Nuclear, &VisionRange, Option<&MovementTarget>),
        With<RobotMarker>,
    >,
    warbases: Query<(Entity, &Transform, &Team), With<Warbase>>,
    map: Res<crate::map::grid::MapGrid>,
) {
    for (entity, mut cmd, tf, robot_team, nuc, vision, cur_target) in &mut robots {
        let RobotCommand::DestroyEnemyBase(ref mut target_opt) = *cmd else { continue; };

        let robot_pos = tf.translation;
        let from_cell = map.world_to_grid(robot_pos);
        let los_ok = |pos: Vec3| -> bool {
            let to_cell = map.world_to_grid(pos);
            match (from_cell, to_cell) {
                (Some(f), Some(t)) => map.has_line_of_sight(f, t),
                _ => false,
            }
        };

        let visible_warbase = target_opt
            .and_then(|e| warbases.get(e).ok())
            .filter(|(_, _, t)| **t != *robot_team)
            .filter(|(_, t, _)| robot_pos.distance(t.translation) <= vision.0)
            .filter(|(_, t, _)| los_ok(t.translation))
            .map(|(e, t, _)| (e, t.translation))
            .or_else(|| {
                warbases
                    .iter()
                    .filter(|(_, _, t)| **t != *robot_team)
                    .find(|(_, t, _)| robot_pos.distance(t.translation) <= vision.0 && los_ok(t.translation))
                    .map(|(e, t, _)| (e, t.translation))
            });

        if let Some((wb_entity, wb_pos)) = visible_warbase {
            if *target_opt != Some(wb_entity) {
                *target_opt = Some(wb_entity);
            }
            if robot_pos.distance(wb_pos) <= nuc.blast_radius * 0.5 {
                commands.entity(entity).remove::<MovementTarget>();
            } else {
                commands.entity(entity).try_insert(MovementTarget(wb_pos));
            }
        } else {
            let near_target = cur_target.map_or(true, |t| robot_pos.xz().distance(t.0.xz()) < 2.0);
            if near_target {
                let explore = exploration_target(entity, robot_pos, map.width, map.height);
                commands.entity(entity).try_insert(MovementTarget(explore));
            }
        }
    }
}

/// Взводит ядерный заряд и самоуничтожает робота у вражеского варбейса.
pub fn arm_nuclear_on_arrival(
    nuclear_robots: Query<
        (Entity, &Transform, &Nuclear, &Team, &RobotCommand),
        (With<RobotMarker>, Without<MovementTarget>),
    >,
    warbases: Query<(&Transform, &Team), With<Warbase>>,
    mut commands: Commands,
) {
    for (entity, tf, nuc, robot_team, cmd) in &nuclear_robots {
        if nuc.armed || !matches!(cmd, RobotCommand::DestroyEnemyBase(_)) {
            continue;
        }
        let near_enemy_warbase = warbases.iter().any(|(wb_tf, wb_team)| {
            *wb_team != *robot_team && tf.translation.distance(wb_tf.translation) <= nuc.blast_radius * 0.5
        });
        if near_enemy_warbase {
            info!("Ядерный заряд активирован у варбейса!");
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

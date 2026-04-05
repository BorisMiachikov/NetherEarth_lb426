use bevy::prelude::*;

use crate::{
    core::Team,
    movement::{steering::CurrentPath, velocity::MovementTarget},
    robot::components::RobotMarker,
};

use super::command::RobotCommand;

/// Диспетчер команд: обрабатывает смену RobotCommand.
pub fn process_commands(
    mut commands: Commands,
    query: Query<(Entity, &RobotCommand, &Transform, &Team), (With<RobotMarker>, Changed<RobotCommand>)>,
    all_robots: Query<(Entity, &Transform, &Team), With<RobotMarker>>,
) {
    for (entity, cmd, tf, team) in &query {
        match cmd {
            RobotCommand::MoveTo(target) => {
                info!("[process_commands] MoveTo → inserting MovementTarget {:?}", target);
                commands.entity(entity).insert(MovementTarget(*target));
            }
            RobotCommand::Idle => {
                commands
                    .entity(entity)
                    .remove::<MovementTarget>()
                    .insert(CurrentPath::default());
            }
            RobotCommand::SeekAndDestroy(_) => {
                // Найти ближайшего врага
                if let Some(target_pos) = nearest_enemy(tf.translation, *team, &all_robots) {
                    commands.entity(entity).insert(MovementTarget(target_pos));
                }
            }
            RobotCommand::SeekAndCapture(_) => {
                // Навигация обрабатывается системой seek_capture_navigation (FixedUpdate)
            }
            RobotCommand::Defend(pos) => {
                // Встать на позицию, затем ждать (атака — Фаза 5)
                commands.entity(entity).insert(MovementTarget(*pos));
            }
            RobotCommand::Patrol(points) => {
                if !points.is_empty() {
                    commands
                        .entity(entity)
                        .insert(MovementTarget(points[0]));
                }
            }
        }
    }
}

/// Обработка Patrol: когда робот достиг точки, двигается к следующей.
pub fn update_patrol(
    mut query: Query<(&mut RobotCommand, &Transform, Option<&MovementTarget>), With<RobotMarker>>,
    mut commands: Commands,
    entities: Query<Entity, With<RobotMarker>>,
) {
    for entity in &entities {
        let Ok((mut cmd, tf, target)) = query.get_mut(entity) else { continue };

        let RobotCommand::Patrol(ref points) = *cmd else { continue };
        if points.is_empty() { continue; }

        // Найти текущую целевую точку
        let Some(mov_target) = target else { continue };
        let closest = points
            .iter()
            .enumerate()
            .min_by_key(|(_, p)| {
                (p.xz().distance(mov_target.0.xz()) * 1000.0) as u32
            })
            .map(|(i, _)| i)
            .unwrap_or(0);

        let dist = tf.translation.xz().distance(mov_target.0.xz());
        if dist < 0.5 {
            // Перейти к следующей точке
            let next = (closest + 1) % points.len();
            let next_pos = points[next];
            commands.entity(entity).insert(MovementTarget(next_pos));
        }
    }
}

fn nearest_enemy(pos: Vec3, my_team: Team, robots: &Query<(Entity, &Transform, &Team), With<RobotMarker>>) -> Option<Vec3> {
    robots
        .iter()
        .filter(|(_, _, &t)| t != my_team)
        .min_by_key(|(_, tf, _)| (tf.translation.distance(pos) * 1000.0) as u32)
        .map(|(_, tf, _)| tf.translation)
}

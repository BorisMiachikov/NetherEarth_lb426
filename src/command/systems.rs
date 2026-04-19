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
) {
    for (entity, cmd, _tf, _team) in &query {
        match cmd {
            RobotCommand::MoveTo(target) => {
                commands.entity(entity).try_insert(MovementTarget(*target));
            }
            RobotCommand::Idle => {
                commands
                    .entity(entity)
                    .remove::<MovementTarget>()
                    .insert(CurrentPath::default());
            }
            RobotCommand::SeekAndDestroy(_) => {
                // Навигация полностью управляется update_seek_destroy (с учётом VisionRange).
                // НЕ трогаем MovementTarget здесь — иначе возникает конфликт с деferred-командами.
            }
            RobotCommand::SeekAndCapture(_) => {
                // Навигация — seek_capture_navigation (FixedUpdate)
            }
            RobotCommand::DestroyEnemyBase(_) => {
                // Навигация — seek_destroy_base (FixedUpdate)
            }
            RobotCommand::Defend(pos) => {
                commands.entity(entity).try_insert(MovementTarget(*pos));
            }
            RobotCommand::Patrol(points) => {
                if !points.is_empty() {
                    commands.entity(entity).try_insert(MovementTarget(points[0]));
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
        let Ok((cmd, tf, target)) = query.get_mut(entity) else { continue };

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
            commands.entity(entity).try_insert(MovementTarget(next_pos));
        }
    }
}


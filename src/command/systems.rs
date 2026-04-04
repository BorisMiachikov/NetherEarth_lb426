use bevy::prelude::*;

use crate::movement::{steering::CurrentPath, velocity::MovementTarget};

use super::command::RobotCommand;

/// Обрабатывает команды роботов:
/// - MoveTo → устанавливает MovementTarget (pathfinding запустится автоматически)
/// - Остальные → лог (Фаза 4+)
pub fn process_commands(
    mut commands: Commands,
    query: Query<(Entity, &RobotCommand), Changed<RobotCommand>>,
) {
    for (entity, cmd) in &query {
        match cmd {
            RobotCommand::MoveTo(target) => {
                commands.entity(entity).insert(MovementTarget(*target));
            }
            RobotCommand::Idle => {
                // Очищаем путь
                commands.entity(entity).remove::<MovementTarget>();
            }
            other => {
                info!("Команда {:?} не реализована (Фаза 4+)", other);
            }
        }
    }
}

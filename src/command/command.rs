use bevy::prelude::*;

/// Текущий приказ робота.
#[derive(Component, Debug, Clone, PartialEq)]
pub enum RobotCommand {
    Idle,
    MoveTo(Vec3),
    SeekAndDestroy(Option<Entity>),
    SeekAndCapture(Option<Entity>),
    Defend(Vec3),
    Patrol(Vec<Vec3>),
}

impl Default for RobotCommand {
    fn default() -> Self {
        Self::Idle
    }
}

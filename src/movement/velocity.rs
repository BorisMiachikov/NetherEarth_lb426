use bevy::prelude::*;

/// Текущая скорость движения (world units/sec).
#[derive(Component, Default, Debug, Clone)]
#[allow(dead_code)]
pub struct Velocity(pub Vec3);

/// Цель движения. Когда задана — робот движется к ней.
#[derive(Component, Debug, Clone)]
pub struct MovementTarget(pub Vec3);

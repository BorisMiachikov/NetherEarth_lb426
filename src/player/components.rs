use bevy::prelude::*;

/// Маркер: игровой скаут (летающий юнит под управлением игрока).
#[derive(Component)]
pub struct PlayerScout;

/// Параметры движения скаута.
#[derive(Component, Debug, Clone)]
pub struct ScoutMovement {
    pub speed: f32,
    pub altitude: f32,
    pub min_alt: f32,
    pub max_alt: f32,
}

impl Default for ScoutMovement {
    fn default() -> Self {
        Self {
            speed: 8.0,
            altitude: 3.0,
            min_alt: 1.0,
            max_alt: 10.0,
        }
    }
}

/// Намерение движения, вычисляемое из ввода. Обнуляется каждый кадр.
#[derive(Component, Default, Debug, Clone)]
pub struct ScoutMoveIntent {
    /// Направление в плоскости XZ, нормализованное (или нулевое).
    pub horizontal: Vec2,
    /// -1.0 = вниз, +1.0 = вверх, 0.0 = нет.
    pub vertical: f32,
}

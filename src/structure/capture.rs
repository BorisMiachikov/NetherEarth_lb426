use bevy::prelude::*;

/// Маркер: структура может быть захвачена.
#[derive(Component)]
pub struct Capturable;

/// Прогресс захвата структуры.
#[derive(Component, Debug, Clone)]
pub struct CaptureProgress {
    /// Текущий прогресс [0..required].
    pub progress: f32,
    /// Время захвата в секундах (зависит от типа шасси).
    pub required: f32,
}

impl CaptureProgress {
    pub fn new(required: f32) -> Self {
        Self {
            progress: 0.0,
            required,
        }
    }

    pub fn is_captured(&self) -> bool {
        self.progress >= self.required
    }

    pub fn fraction(&self) -> f32 {
        (self.progress / self.required).clamp(0.0, 1.0)
    }
}

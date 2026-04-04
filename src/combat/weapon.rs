use bevy::prelude::*;

/// Текущая боевая цель робота.
#[derive(Component, Debug, Clone, Copy)]
pub struct CombatTarget(pub Entity);

/// Таймеры перезарядки — по одному на каждый слот оружия.
#[derive(Component, Debug, Clone)]
pub struct WeaponCooldowns {
    pub cooldowns: [f32; 3],
}

impl Default for WeaponCooldowns {
    fn default() -> Self {
        Self { cooldowns: [0.0; 3] }
    }
}

/// Визуальная вспышка выстрела (hitscan).
#[derive(Component)]
pub struct MuzzleFlash {
    pub target_pos: Vec3,
    pub timer: f32,
}

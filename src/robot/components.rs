use bevy::prelude::*;
use serde::Deserialize;

// ── Типы ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum ChassisType {
    Wheels,
    Bipod,
    Tracks,
    AntiGrav,
}

impl ChassisType {
    pub fn can_fly(self) -> bool {
        matches!(self, ChassisType::AntiGrav)
    }
}

impl std::fmt::Display for ChassisType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum WeaponType {
    Cannon,
    Missile,
    Phasers,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeaponData {
    pub weapon_type: WeaponType,
    pub damage: f32,
    pub range: f32,
    pub reload_time: f32,
}

// ── Компоненты ────────────────────────────────────────────────────────────────

/// Маркер: сущность является роботом.
#[derive(Component)]
pub struct RobotMarker;

/// Шасси робота.
#[derive(Component, Debug, Clone)]
pub struct Chassis {
    pub chassis_type: ChassisType,
    pub base_hp: f32,
    pub speed: f32,
    pub mobility: f32,
    pub capture_time: f32,
}

/// До 3 слотов оружия.
#[derive(Component, Debug, Clone)]
pub struct WeaponSlots {
    pub slots: [Option<WeaponData>; 3],
}

impl WeaponSlots {
    pub fn empty() -> Self {
        Self {
            slots: [None, None, None],
        }
    }

    pub fn count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    pub fn total_weight(&self) -> f32 {
        self.slots
            .iter()
            .flatten()
            .map(|w| match w.weapon_type {
                WeaponType::Cannon => 10.0,
                WeaponType::Missile => 25.0,
                WeaponType::Phasers => 30.0,
            })
            .sum()
    }
}

/// Электроника (опциональный модуль).
#[derive(Component, Debug, Clone)]
pub struct Electronics {
    pub radar_range: f32,
    pub accuracy_bonus: f32,
    pub fire_rate_bonus: f32,
    pub capture_time_reduction: f32,
}

impl Default for Electronics {
    fn default() -> Self {
        Self {
            radar_range: 20.0,
            accuracy_bonus: 0.3,
            fire_rate_bonus: 0.2,
            capture_time_reduction: 0.3,
        }
    }
}

/// Ядерный заряд (опциональный модуль).
#[derive(Component, Debug, Clone)]
pub struct Nuclear {
    pub blast_radius: f32,
    pub detonation_delay: f32,
    pub armed: bool,
}

impl Default for Nuclear {
    fn default() -> Self {
        Self {
            blast_radius: 8.0,
            detonation_delay: 2.0,
            armed: false,
        }
    }
}

/// Расчётные характеристики робота (пересчитываются при изменении модулей).
#[derive(Component, Debug, Clone, Default)]
pub struct RobotStats {
    pub max_hp: f32,
    pub speed: f32,
    pub capture_time: f32,
}

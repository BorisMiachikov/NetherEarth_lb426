use bevy::prelude::*;
use serde::Deserialize;

/// Тип фабрики определяет, какой ресурс она производит.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, Deserialize)]
pub enum FactoryType {
    Chassis,
    Cannon,
    Missile,
    Phasers,
    Electronics,
    Nuclear,
}

impl std::fmt::Display for FactoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            FactoryType::Chassis => "Шасси",
            FactoryType::Cannon => "Пушка",
            FactoryType::Missile => "Ракета",
            FactoryType::Phasers => "Фазеры",
            FactoryType::Electronics => "Электроника",
            FactoryType::Nuclear => "Ядерный",
        };
        write!(f, "{name}")
    }
}

/// Маркер: эта сущность — фабрика.
#[derive(Component)]
pub struct Factory;

/// Скорость производства ресурсов (за игровой день).
#[derive(Component, Debug, Clone)]
pub struct ProductionRate {
    pub resource_per_day: u32,
    pub general_per_day: u32,
}

impl Default for ProductionRate {
    fn default() -> Self {
        Self {
            resource_per_day: 5,
            general_per_day: 2,
        }
    }
}

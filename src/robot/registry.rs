use bevy::prelude::*;
use serde::Deserialize;

use super::components::{ChassisType, WeaponData, WeaponType};

// ── RON-определения ──────────────────────────────────────────────────────────

#[derive(Deserialize, Debug, Clone)]
pub struct ChassisDef {
    pub chassis_type: ChassisType,
    pub base_hp: f32,
    pub speed: f32,
    pub mobility: f32,
    pub can_fly: bool,
    pub capture_time: f32,
    pub cost_chassis: u32,
    pub cost_general: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WeaponDef {
    pub weapon_type: WeaponType,
    pub damage: f32,
    pub range: f32,
    pub reload_time: f32,
    pub cost_resource: u32,
    pub cost_general: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ElectronicsDef {
    pub accuracy_bonus: f32,
    pub fire_rate_bonus: f32,
    pub radar_range: f32,
    pub capture_time_reduction: f32,
    pub cost_electronics: u32,
    pub cost_general: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NuclearDef {
    pub blast_radius: f32,
    pub detonation_delay: f32,
    pub cost_nuclear: u32,
    pub cost_general: u32,
}

// ── Реестр ───────────────────────────────────────────────────────────────────

/// Глобальный реестр характеристик модулей, загруженных из RON.
#[derive(Resource, Debug, Clone)]
pub struct ModuleRegistry {
    pub chassis: Vec<ChassisDef>,
    pub weapons: Vec<WeaponDef>,
    pub electronics: ElectronicsDef,
    pub nuclear: NuclearDef,
}

impl ModuleRegistry {
    pub fn chassis(&self, ct: ChassisType) -> Option<&ChassisDef> {
        self.chassis.iter().find(|c| c.chassis_type == ct)
    }

    pub fn weapon(&self, wt: WeaponType) -> Option<&WeaponDef> {
        self.weapons.iter().find(|w| w.weapon_type == wt)
    }
}

/// Загружает ModuleRegistry из RON-файлов.
pub fn load_module_registry() -> Result<ModuleRegistry, String> {
    let chassis: Vec<ChassisDef> = load_ron("configs/chassis.ron")?;
    let weapons: Vec<WeaponDef> = load_ron("configs/weapons.ron")?;
    let electronics: ElectronicsDef = load_ron("configs/electronics.ron")?;
    let nuclear: NuclearDef = load_ron("configs/nuclear.ron")?;

    Ok(ModuleRegistry {
        chassis,
        weapons,
        electronics,
        nuclear,
    })
}

fn load_ron<T: for<'de> serde::Deserialize<'de>>(path: &str) -> Result<T, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Не удалось прочитать {path}: {e}"))?;
    ron::from_str(&content).map_err(|e| format!("Ошибка парсинга {path}: {e}"))
}

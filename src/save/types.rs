use serde::{Deserialize, Serialize};

use crate::{
    core::Team,
    robot::components::{ChassisType, WeaponType},
    structure::factory::FactoryType,
};

pub const SAVE_VERSION: u32 = 1;
pub const SAVE_SLOT_COUNT: usize = 3;

/// Полный снимок состояния игры для сохранения/загрузки.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveData {
    pub version: u32,
    pub game_day: u32,
    pub day_elapsed: f32,
    pub seconds_per_day: f32,
    pub resources: SavedResources,
    pub scout_position: [f32; 3],
    pub robots: Vec<SavedRobot>,
    pub factories: Vec<SavedFactory>,
    pub warbases: Vec<SavedWarbase>,
    pub ai: SavedAI,
}

/// Ресурсы игрока.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedResources {
    pub general: i32,
    pub chassis: i32,
    pub cannon: i32,
    pub missile: i32,
    pub phasers: i32,
    pub electronics: i32,
    pub nuclear: i32,
}

/// Сохранённый робот.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedRobot {
    pub position: [f32; 3],
    pub team: Team,
    pub chassis: ChassisType,
    pub weapons: Vec<WeaponType>,
    pub has_electronics: bool,
    pub has_nuclear: bool,
    pub current_hp: f32,
    pub nuclear_armed: bool,
    pub command: SavedCommand,
}

/// Приказ без ссылок на Entity (Entity IDs не переживают сохранение).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SavedCommand {
    Idle,
    MoveTo([f32; 3]),
    SeekAndDestroy,
    SeekAndCapture,
    DestroyEnemyBase,
    Defend([f32; 3]),
    Patrol(Vec<[f32; 3]>),
}

/// Сохранённая фабрика.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedFactory {
    pub position: [f32; 3],
    pub factory_type: FactoryType,
    pub team: Team,
    pub capture_progress: f32,
    pub capture_required: f32,
}

/// Сохранённый варбейс.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedWarbase {
    pub position: [f32; 3],
    pub team: Team,
}

/// Состояние ИИ-командира.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SavedAI {
    pub decision_timer: f32,
    pub build_timer: f32,
    pub decision_counter: u32,
    pub robots_built: u32,
}

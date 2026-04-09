use bevy::prelude::*;
use serde::Deserialize;

use super::grid::{CellType, MapGrid};

#[derive(Deserialize, Clone)]
pub struct GridPos {
    pub x: u32,
    pub y: u32,
}

#[derive(Deserialize)]
pub enum CellTypeDef {
    Blocked,
    /// Скала: непроходима для всех, блокирует LOS.
    Rock,
    /// Яма/расщелина: непроходима для Wheels/Bipod/Tracks.
    Pit,
    /// Песок: замедляет Wheels и Bipod.
    Sand,
}

#[derive(Deserialize)]
pub struct MapCellDef {
    pub x: u32,
    pub y: u32,
    pub cell_type: CellTypeDef,
}

#[derive(Deserialize, Clone, Debug)]
pub enum FactoryTypeDef {
    General,
    Chassis,
    Cannon,
    Missile,
    Phasers,
    Electronics,
    Nuclear,
}

#[derive(Deserialize, Clone, Debug)]
pub enum TeamDef {
    Player,
    Enemy,
    Neutral,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FactoryDef {
    pub x: u32,
    pub y: u32,
    pub factory_type: FactoryTypeDef,
    #[serde(default = "TeamDef::neutral")]
    pub team: TeamDef,
}

impl TeamDef {
    fn neutral() -> Self {
        TeamDef::Neutral
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct WarbaseDef {
    pub x: u32,
    pub y: u32,
    pub team: TeamDef,
}

#[derive(Deserialize)]
pub struct MapData {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<MapCellDef>,
    pub player_spawn: GridPos,
    pub factories: Vec<FactoryDef>,
    pub warbases: Vec<WarbaseDef>,
}

/// Конфигурация спавна игрока.
#[derive(Resource, Debug)]
pub struct MapSpawnPoints {
    pub player_spawn: (u32, u32),
}

/// Данные о структурах, прочитанные из RON — используются StructurePlugin при спавне.
#[derive(Resource, Debug, Clone)]
pub struct MapStructures {
    pub factories: Vec<FactoryDef>,
    pub warbases: Vec<WarbaseDef>,
}

/// Загружает карту из RON-файла.
pub fn load_map_from_ron(
    path: &str,
) -> Result<(MapGrid, MapSpawnPoints, MapStructures), String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Не удалось прочитать {path}: {e}"))?;

    let data: MapData = ron::from_str(&content)
        .map_err(|e| format!("Ошибка парсинга RON {path}: {e}"))?;

    let mut grid = MapGrid::new(data.width, data.height);
    for cell in data.cells {
        let ct = match cell.cell_type {
            CellTypeDef::Blocked => CellType::Blocked,
            CellTypeDef::Rock    => CellType::Rock,
            CellTypeDef::Pit     => CellType::Pit,
            CellTypeDef::Sand    => CellType::Sand,
        };
        grid.set(cell.x, cell.y, ct);
    }

    let spawn = MapSpawnPoints {
        player_spawn: (data.player_spawn.x, data.player_spawn.y),
    };

    let structures = MapStructures {
        factories: data.factories,
        warbases: data.warbases,
    };

    Ok((grid, spawn, structures))
}

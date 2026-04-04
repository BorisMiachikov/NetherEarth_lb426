use bevy::prelude::*;
use serde::Deserialize;

use super::grid::{CellType, MapGrid};

#[derive(Deserialize)]
pub struct GridPos {
    pub x: u32,
    pub y: u32,
}

#[derive(Deserialize)]
pub enum CellTypeDef {
    Blocked,
    Structure,
}

#[derive(Deserialize)]
pub struct MapCellDef {
    pub x: u32,
    pub y: u32,
    pub cell_type: CellTypeDef,
}

#[derive(Deserialize)]
pub struct MapData {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<MapCellDef>,
    pub player_spawn: GridPos,
    pub enemy_warbase: GridPos,
    pub player_warbase: GridPos,
}

/// Конфигурация спавна, извлечённая из карты.
#[derive(Resource, Debug)]
pub struct MapSpawnPoints {
    pub player_spawn: (u32, u32),
    pub player_warbase: (u32, u32),
    pub enemy_warbase: (u32, u32),
}

/// Загружает карту из RON-файла, инициализирует MapGrid и MapSpawnPoints.
/// Путь относительно рабочей директории (т.е. корня проекта).
pub fn load_map_from_ron(path: &str) -> Result<(MapGrid, MapSpawnPoints), String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Не удалось прочитать {path}: {e}"))?;

    let data: MapData = ron::from_str(&content)
        .map_err(|e| format!("Ошибка парсинга RON {path}: {e}"))?;

    let mut grid = MapGrid::new(data.width, data.height);

    for cell in data.cells {
        let ct = match cell.cell_type {
            CellTypeDef::Blocked => CellType::Blocked,
            CellTypeDef::Structure => CellType::Structure,
        };
        grid.set(cell.x, cell.y, ct);
    }

    let spawn = MapSpawnPoints {
        player_spawn: (data.player_spawn.x, data.player_spawn.y),
        player_warbase: (data.player_warbase.x, data.player_warbase.y),
        enemy_warbase: (data.enemy_warbase.x, data.enemy_warbase.y),
    };

    Ok((grid, spawn))
}

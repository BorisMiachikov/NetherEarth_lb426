use crate::map::loader::{
    CellTypeDef, FactoryDef, GridPos, MapCellDef, MapData, WarbaseDef,
};
use crate::map::grid::{CellType, MapGrid};
use crate::ui::menu::ScenarioDef;

use super::state::{EditorState, MapSize};
use super::tools::ct_to_cell_type_def;

/// Строит `MapData` из текущего состояния редактора + MapGrid.
pub fn build_map_data(editor: &EditorState, grid: &MapGrid) -> MapData {
    let size = editor.map_size.value();
    let mut cells: Vec<MapCellDef> = Vec::new();

    for y in 0..size {
        for x in 0..size {
            if let Some(ct) = grid.get(x, y) {
                match ct {
                    CellType::Open => {} // Open — по умолчанию, не сериализуем
                    _ => {
                        cells.push(MapCellDef {
                            x,
                            y,
                            cell_type: ct_to_cell_type_def(ct),
                        });
                    }
                }
            }
        }
    }

    MapData {
        width: size,
        height: size,
        cells,
        player_spawn: GridPos {
            x: editor.player_spawn.0,
            y: editor.player_spawn.1,
        },
        factories: editor.factories.clone(),
        warbases: editor.warbases.clone(),
    }
}

/// Сохраняет карту в `data/maps/{name}.ron`.
/// Возвращает Ok(path) или Err(сообщение).
pub fn save_map(editor: &EditorState, grid: &MapGrid) -> Result<String, String> {
    let map_data = build_map_data(editor, grid);

    // Ручная RON-сериализация (ron::ser::to_string_pretty требует Serialize)
    // Используем ron через serde (MapData должна derive Serialize — добавим при необходимости)
    // Пока пишем форматированный RON вручную
    let ron_str = serialize_map_data_ron(&map_data);

    let _ = std::fs::create_dir_all("data/maps");
    let path = format!("data/maps/{}.ron", editor.file_name);
    std::fs::write(&path, ron_str).map_err(|e| format!("Ошибка записи {path}: {e}"))?;

    // Обновляем/создаём сценарий
    save_scenario(editor)?;

    Ok(path)
}

fn save_scenario(editor: &EditorState) -> Result<(), String> {
    let def = ScenarioDef {
        name: editor.map_name.clone(),
        description: editor.map_description.clone(),
        map_path: format!("data/maps/{}.ron", editor.file_name),
    };
    let ron_str = format!(
        "(\n    name: {:?},\n    description: {:?},\n    map_path: {:?},\n)\n",
        def.name, def.description, def.map_path
    );
    let _ = std::fs::create_dir_all("data/scenarios");
    let path = format!("data/scenarios/{}.ron", editor.file_name);
    std::fs::write(&path, ron_str).map_err(|e| format!("Ошибка записи сценария {path}: {e}"))?;
    Ok(())
}

fn serialize_map_data_ron(data: &MapData) -> String {
    let mut out = String::from("(\n");
    out.push_str(&format!("    width: {},\n", data.width));
    out.push_str(&format!("    height: {},\n", data.height));
    out.push_str("    cells: [\n");
    for c in &data.cells {
        out.push_str(&format!(
            "        (x: {}, y: {}, cell_type: {:?}),\n",
            c.x, c.y, c.cell_type
        ));
    }
    out.push_str("    ],\n");
    out.push_str(&format!(
        "    player_spawn: (x: {}, y: {}),\n",
        data.player_spawn.x, data.player_spawn.y
    ));
    out.push_str("    factories: [\n");
    for f in &data.factories {
        out.push_str(&format!(
            "        (x: {}, y: {}, factory_type: {:?}, team: {:?}),\n",
            f.x, f.y, f.factory_type, f.team
        ));
    }
    out.push_str("    ],\n");
    out.push_str("    warbases: [\n");
    for w in &data.warbases {
        out.push_str(&format!(
            "        (x: {}, y: {}, team: {:?}),\n",
            w.x, w.y, w.team
        ));
    }
    out.push_str("    ],\n");
    out.push_str(")\n");
    out
}

/// Загружает карту из файла в EditorState + MapGrid.
pub fn load_map_into_editor(
    file_stem: &str,
    editor: &mut EditorState,
    grid: &mut MapGrid,
) -> Result<(), String> {
    use crate::map::loader::{load_map_from_ron};
    use crate::map::grid::MapGrid as MG;

    let path = format!("data/maps/{file_stem}.ron");
    let (new_grid, spawn, structures) = load_map_from_ron(&path)?;

    // Определяем MapSize по размеру карты
    editor.map_size = match new_grid.width {
        32 => MapSize::Small,
        96 => MapSize::Large,
        _  => MapSize::Normal,
    };

    // Копируем ячейки нового grid в текущий (ресайз если нужно)
    *grid = MG::new(new_grid.width, new_grid.height);
    for y in 0..new_grid.height {
        for x in 0..new_grid.width {
            if let Some(ct) = new_grid.get(x, y) {
                grid.set(x, y, ct);
            }
        }
    }

    editor.file_name = file_stem.to_owned();
    editor.map_name = file_stem.to_owned();
    editor.player_spawn = spawn.player_spawn;
    editor.factories = structures.factories;
    editor.warbases = structures.warbases;
    editor.undo_stack.clear();
    editor.redo_stack.clear();
    editor.dirty = false;
    Ok(())
}

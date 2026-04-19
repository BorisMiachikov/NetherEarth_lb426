use bevy::prelude::*;

use crate::map::{
    grid::{CellType, MapGrid},
    loader::{CellTypeDef, FactoryDef, WarbaseDef},
};

use super::{
    state::{EditorAction, EditorState, EditorTool, PlacedStructureKind},
    terrain::rebuild_terrain_cell,
};

/// Применяет активный инструмент по клику ЛКМ.
pub fn apply_tool(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<EditorState>,
    mut grid: ResMut<MapGrid>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Redo: Ctrl+Shift+Z — проверяем РАНЬШЕ Undo, иначе перехватывается
    if keys.just_pressed(KeyCode::KeyZ)
        && keys.pressed(KeyCode::ControlLeft)
        && keys.pressed(KeyCode::ShiftLeft)
    {
        redo(&mut editor, &mut grid, &mut commands, &mut meshes, &mut materials);
        return;
    }
    // Undo: Ctrl+Z
    if keys.just_pressed(KeyCode::KeyZ) && keys.pressed(KeyCode::ControlLeft) {
        undo(&mut editor, &mut grid, &mut commands, &mut meshes, &mut materials);
        return;
    }
    // Запросы от кнопок UI
    let do_undo = std::mem::take(&mut editor.undo_requested);
    let do_redo = std::mem::take(&mut editor.redo_requested);
    if do_undo {
        undo(&mut editor, &mut grid, &mut commands, &mut meshes, &mut materials);
        return;
    }
    if do_redo {
        redo(&mut editor, &mut grid, &mut commands, &mut meshes, &mut materials);
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some((cx, cy)) = editor.hovered_cell else {
        return;
    };

    let size = editor.map_size.value();
    if cx >= size || cy >= size {
        return;
    }

    match editor.current_tool.clone() {
        EditorTool::TerrainBrush => {
            apply_brush(&mut editor, &mut grid, &mut commands, &mut meshes, &mut materials, cx, cy);
        }
        EditorTool::PlaceFactory => {
            // Нельзя ставить на занятую/непроходимую клетку
            if grid.get(cx, cy) == Some(CellType::Open) {
                let def = FactoryDef {
                    x: cx,
                    y: cy,
                    factory_type: editor.factory_type.clone(),
                    team: editor.place_team.clone(),
                };
                editor.push_action(EditorAction::StructurePlaced {
                    kind: PlacedStructureKind::Factory(def.clone()),
                });
                editor.factories.push(def);
            }
        }
        EditorTool::PlaceWarbase => {
            if grid.get(cx, cy) == Some(CellType::Open) {
                let def = WarbaseDef {
                    x: cx,
                    y: cy,
                    team: editor.place_team.clone(),
                };
                editor.push_action(EditorAction::StructurePlaced {
                    kind: PlacedStructureKind::Warbase(def.clone()),
                });
                editor.warbases.push(def);
            }
        }
        EditorTool::PlacePlayerSpawn => {
            let from = editor.player_spawn;
            editor.push_action(EditorAction::PlayerSpawnMoved { from, to: (cx, cy) });
            editor.player_spawn = (cx, cy);
        }
        EditorTool::Erase => {
            erase_at(&mut editor, cx, cy);
        }
    }
}

fn apply_brush(
    editor: &mut EditorState,
    grid: &mut MapGrid,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    cx: u32,
    cy: u32,
) {
    let radius = editor.brush_size.radius();
    let to_ct = cell_type_def_to_ct(&editor.brush_cell_type);

    let size = editor.map_size.value() as i32;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || ny < 0 || nx >= size || ny >= size {
                continue;
            }
            let (nx, ny) = (nx as u32, ny as u32);
            let from_ct = grid.get(nx, ny).unwrap_or(CellType::Open);
            let from_def = ct_to_cell_type_def(from_ct);
            if from_ct != to_ct {
                editor.push_action(EditorAction::CellChanged {
                    x: nx,
                    y: ny,
                    from: from_def,
                    to: editor.brush_cell_type.clone(),
                });
                grid.set(nx, ny, to_ct);
                rebuild_terrain_cell(commands, meshes, materials, grid, nx, ny);
            }
        }
    }
}

fn erase_at(editor: &mut EditorState, cx: u32, cy: u32) {
    // Убираем фабрики
    if let Some(idx) = editor.factories.iter().position(|f| f.x == cx && f.y == cy) {
        let removed = editor.factories.remove(idx);
        editor.push_action(EditorAction::StructureRemoved {
            kind: PlacedStructureKind::Factory(removed),
        });
        return;
    }
    // Убираем варбейсы
    if let Some(idx) = editor.warbases.iter().position(|w| w.x == cx && w.y == cy) {
        let removed = editor.warbases.remove(idx);
        editor.push_action(EditorAction::StructureRemoved {
            kind: PlacedStructureKind::Warbase(removed),
        });
    }
}

fn undo(
    editor: &mut EditorState,
    grid: &mut MapGrid,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let Some(action) = editor.undo_stack.pop() else { return };
    match &action {
        EditorAction::CellChanged { x, y, from, .. } => {
            let ct = cell_type_def_to_ct(from);
            grid.set(*x, *y, ct);
            rebuild_terrain_cell(commands, meshes, materials, grid, *x, *y);
        }
        EditorAction::StructurePlaced { kind, .. } => match kind {
            PlacedStructureKind::Factory(f) => {
                editor.factories.retain(|e| !(e.x == f.x && e.y == f.y));
            }
            PlacedStructureKind::Warbase(w) => {
                editor.warbases.retain(|e| !(e.x == w.x && e.y == w.y));
            }
        },
        EditorAction::StructureRemoved { kind, .. } => match kind.clone() {
            PlacedStructureKind::Factory(f) => editor.factories.push(f),
            PlacedStructureKind::Warbase(w) => editor.warbases.push(w),
        },
        EditorAction::PlayerSpawnMoved { from, .. } => {
            editor.player_spawn = *from;
        }
    }
    editor.redo_stack.push(action);
}

fn redo(
    editor: &mut EditorState,
    grid: &mut MapGrid,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let Some(action) = editor.redo_stack.pop() else { return };
    match &action {
        EditorAction::CellChanged { x, y, to, .. } => {
            let ct = cell_type_def_to_ct(to);
            grid.set(*x, *y, ct);
            rebuild_terrain_cell(commands, meshes, materials, grid, *x, *y);
        }
        EditorAction::StructurePlaced { kind, .. } => match kind.clone() {
            PlacedStructureKind::Factory(f) => editor.factories.push(f),
            PlacedStructureKind::Warbase(w) => editor.warbases.push(w),
        },
        EditorAction::StructureRemoved { kind, .. } => match kind {
            PlacedStructureKind::Factory(f) => {
                editor.factories.retain(|e| !(e.x == f.x && e.y == f.y));
            }
            PlacedStructureKind::Warbase(w) => {
                editor.warbases.retain(|e| !(e.x == w.x && e.y == w.y));
            }
        },
        EditorAction::PlayerSpawnMoved { to, .. } => {
            editor.player_spawn = *to;
        }
    }
    editor.undo_stack.push(action);
}

/// Обновляет предпросмотровую сущность под курсором.
pub fn update_hover_preview(
    editor: Res<EditorState>,
    mut gizmos: Gizmos,
    grid: Res<MapGrid>,
) {
    let Some((cx, cy)) = editor.hovered_cell else { return };
    let center = grid.grid_to_world(cx, cy) + Vec3::Y * 0.05;
    // Жёлтый контур вокруг hover-клетки
    gizmos.rect(
        Isometry3d::from_translation(center),
        Vec2::splat(0.95),
        Color::srgb(1.0, 0.9, 0.1),
    );
    // Подсвечиваем всю область кисти
    let radius = editor.brush_size.radius();
    if editor.current_tool == EditorTool::TerrainBrush && radius > 0 {
        let size = editor.map_size.value();
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || ny < 0 || nx >= size as i32 || ny >= size as i32 {
                    continue;
                }
                if dx == 0 && dy == 0 { continue; }
                let bc = grid.grid_to_world(nx as u32, ny as u32) + Vec3::Y * 0.05;
                gizmos.rect(
                    Isometry3d::from_translation(bc),
                    Vec2::splat(0.95),
                    Color::srgba(1.0, 0.9, 0.1, 0.4),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Вспомогательные конвертеры
// ---------------------------------------------------------------------------

pub fn cell_type_def_to_ct(def: &CellTypeDef) -> CellType {
    match def {
        CellTypeDef::Blocked => CellType::Blocked,
        CellTypeDef::Rock    => CellType::Rock,
        CellTypeDef::Pit     => CellType::Pit,
        CellTypeDef::Sand    => CellType::Sand,
    }
}

pub fn ct_to_cell_type_def(ct: CellType) -> CellTypeDef {
    match ct {
        CellType::Blocked | CellType::Structure(_) => CellTypeDef::Blocked,
        CellType::Rock    => CellTypeDef::Rock,
        CellType::Pit     => CellTypeDef::Pit,
        CellType::Sand    => CellTypeDef::Sand,
        CellType::Open    => CellTypeDef::Blocked, // Open не сериализуется (по умолчанию)
    }
}

use bevy::prelude::*;

use crate::map::{
    grid::{CellType, MapGrid},
    TerrainCellMarker,
};

/// Удаляет существующий terrain-меш для клетки (gx, gy) и спавнит новый.
/// Вызывается редактором при изменении типа клетки.
pub fn rebuild_terrain_cell(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    grid: &MapGrid,
    gx: u32,
    gy: u32,
) {
    // Запоминаем команду на despawn — нельзя делать query здесь (нет World),
    // поэтому используем специальный observer-паттерн через Commands.
    // Фактически despawn старой entity произойдёт через RebuildTerrainCell trigger.
    commands.trigger(RebuildTerrainCell { x: gx, y: gy });
    let _ = (meshes, materials, grid); // используются в observer
}

/// Observer-триггер: пересобрать terrain-меш для ячейки (x, y).
#[derive(Event, Debug)]
pub struct RebuildTerrainCell {
    pub x: u32,
    pub y: u32,
}

/// Система-наблюдатель: реагирует на RebuildTerrainCell.
pub fn on_rebuild_terrain_cell(
    trigger: On<RebuildTerrainCell>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing: Query<(Entity, &TerrainCellMarker)>,
    grid: Res<MapGrid>,
) {
    let ev = trigger.event();
    let (gx, gy) = (ev.x, ev.y);

    // Despawn старого меша для этой клетки
    for (entity, marker) in &existing {
        if marker.x == gx && marker.y == gy {
            commands.entity(entity).despawn();
        }
    }

    let base = grid.grid_to_world(gx, gy);
    let Some(cell) = grid.get(gx, gy) else { return };

    match cell {
        CellType::Rock | CellType::Blocked => {
            commands.spawn((
                Name::new(format!("Rock({gx},{gy})")),
                TerrainCellMarker { x: gx, y: gy },
                Mesh3d(meshes.add(Cuboid::new(0.9, 1.4, 0.9))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.45, 0.42, 0.40),
                    perceptual_roughness: 0.9,
                    ..default()
                })),
                Transform::from_translation(base.with_y(0.7)),
            ));
        }
        CellType::Pit => {
            commands.spawn((
                Name::new(format!("Pit({gx},{gy})")),
                TerrainCellMarker { x: gx, y: gy },
                Mesh3d(meshes.add(Plane3d::default().mesh().size(0.95, 0.95))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.08, 0.06, 0.05),
                    perceptual_roughness: 1.0,
                    ..default()
                })),
                Transform::from_translation(base.with_y(-0.05)),
            ));
        }
        CellType::Sand => {
            commands.spawn((
                Name::new(format!("Sand({gx},{gy})")),
                TerrainCellMarker { x: gx, y: gy },
                Mesh3d(meshes.add(Plane3d::default().mesh().size(1.0, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.82, 0.72, 0.42),
                    perceptual_roughness: 1.0,
                    ..default()
                })),
                Transform::from_translation(base.with_y(0.01)),
            ));
        }
        // Open — меш не нужен, ground plane достаточно
        _ => {}
    }
}

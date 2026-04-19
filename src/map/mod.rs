pub mod collision;
pub mod grid;
pub mod loader;

use bevy::prelude::*;

use collision::scout_collision;
use grid::{CellType, MapGrid};
use loader::{load_map_from_ron, MapSpawnPoints, MapStructures};

use crate::app::state::AppState;


/// Маркер: terrain-меш конкретной ячейки. Используется редактором для замены меша.
#[derive(Component, Debug, Clone, Copy)]
pub struct TerrainCellMarker {
    pub x: u32,
    pub y: u32,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        let (grid, spawn, structures) =
            load_map_from_ron("data/maps/default.ron").unwrap_or_else(|e| {
                warn!("Не удалось загрузить карту: {e}. Использую пустую 64×64.");
                (
                    MapGrid::new(64, 64),
                    MapSpawnPoints {
                        player_spawn: (5, 5),
                    },
                    MapStructures {
                        factories: vec![],
                        warbases: vec![],
                    },
                )
            });

        app.insert_resource(grid)
            .insert_resource(spawn)
            .insert_resource(structures)
            .add_systems(Startup, spawn_ground)
            .add_systems(Startup, spawn_terrain.after(spawn_ground))
            .add_systems(FixedUpdate, scout_collision.run_if(in_state(AppState::Playing)));
    }
}

pub fn spawn_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<MapGrid>,
) {
    let w = map.width as f32;
    let h = map.height as f32;

    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(w, h))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.38, 0.18),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(w * 0.5, 0.0, h * 0.5),
    ));

    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: true,
    });
}

/// Спавн визуальных мешей для Rock, Pit, Sand на основе данных MapGrid.
pub fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<MapGrid>,
) {
    // Общие материалы для каждого типа рельефа
    let rock_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.42, 0.40),
        perceptual_roughness: 0.9,
        ..default()
    });
    let pit_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.06, 0.05),
        perceptual_roughness: 1.0,
        ..default()
    });
    let sand_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.82, 0.72, 0.42),
        perceptual_roughness: 1.0,
        ..default()
    });

    // Меши (переиспользуются для всех ячеек одного типа)
    let rock_mesh = meshes.add(Cuboid::new(0.9, 1.4, 0.9));
    let pit_mesh  = meshes.add(Plane3d::default().mesh().size(0.95, 0.95));
    let sand_mesh = meshes.add(Plane3d::default().mesh().size(1.0, 1.0));

    for ((gx, gy), cell) in map.iter_cells() {
        let base = map.grid_to_world(gx, gy);
        match cell {
            CellType::Rock | CellType::Blocked => {
                commands.spawn((
                    Name::new(format!("Rock({gx},{gy})")),
                    TerrainCellMarker { x: gx, y: gy },
                    Mesh3d(rock_mesh.clone()),
                    MeshMaterial3d(rock_mat.clone()),
                    Transform::from_translation(base.with_y(0.7)),
                ));
            }
            CellType::Pit => {
                // Яма — тёмная плоскость чуть ниже уровня земли
                commands.spawn((
                    Name::new(format!("Pit({gx},{gy})")),
                    TerrainCellMarker { x: gx, y: gy },
                    Mesh3d(pit_mesh.clone()),
                    MeshMaterial3d(pit_mat.clone()),
                    Transform::from_translation(base.with_y(-0.05)),
                ));
            }
            CellType::Sand => {
                // Песок — цветная плоскость на уровне земли поверх ground plane
                commands.spawn((
                    Name::new(format!("Sand({gx},{gy})")),
                    TerrainCellMarker { x: gx, y: gy },
                    Mesh3d(sand_mesh.clone()),
                    MeshMaterial3d(sand_mat.clone()),
                    Transform::from_translation(base.with_y(0.01)),
                ));
            }
            _ => {}
        }
    }
}

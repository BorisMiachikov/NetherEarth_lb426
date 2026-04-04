pub mod collision;
pub mod grid;
pub mod loader;

use bevy::prelude::*;

use collision::scout_collision;
use grid::MapGrid;
use loader::{load_map_from_ron, MapSpawnPoints, MapStructures};

pub use grid::{CellType, MapGrid as Map, CELL_SIZE};

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
            .add_systems(FixedUpdate, scout_collision);
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

pub mod grid;
pub mod loader;

use bevy::prelude::*;

use grid::MapGrid;
use loader::load_map_from_ron;

pub use grid::{CellType, MapGrid as Map, CELL_SIZE};
pub use loader::MapSpawnPoints;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        // Загружаем карту синхронно при старте
        let (grid, spawn) = load_map_from_ron("data/maps/default.ron").unwrap_or_else(|e| {
            warn!("Не удалось загрузить карту: {e}. Использую пустую 64×64.");
            (MapGrid::new(64, 64), MapSpawnPoints {
                player_spawn: (10, 10),
                player_warbase: (10, 10),
                enemy_warbase: (54, 54),
            })
        });

        app.insert_resource(grid)
            .insert_resource(spawn)
            .add_systems(Startup, spawn_ground);
    }
}

fn spawn_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<MapGrid>,
) {
    let w = map.width as f32;
    let h = map.height as f32;

    // Плоскость земли
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(w, h))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.38, 0.18),
            perceptual_roughness: 1.0,
            ..default()
        })),
        // Plane3d центрируется по умолчанию — сдвигаем в начало координат
        Transform::from_xyz(w * 0.5, 0.0, h * 0.5),
    ));

    // Направленный свет
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

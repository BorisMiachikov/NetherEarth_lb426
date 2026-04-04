use bevy::prelude::*;

use crate::map::grid::MapGrid;

use super::components::{PlayerScout, ScoutMoveIntent, ScoutMovement};

/// Перемещает скаута согласно ScoutMoveIntent, зажимает по границам карты и высоте.
pub fn move_scout(
    time: Res<Time>,
    _map: Res<MapGrid>,
    mut query: Query<(&mut Transform, &mut ScoutMovement, &ScoutMoveIntent), With<PlayerScout>>,
) {
    let Ok((mut transform, mut movement, intent)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let speed = movement.speed;

    // Горизонтальное движение (XZ)
    transform.translation.x += intent.horizontal.x * speed * dt;
    transform.translation.z += intent.horizontal.y * speed * dt;

    // Вертикальное движение (Y = высота)
    movement.altitude = (movement.altitude + intent.vertical * speed * dt)
        .clamp(movement.min_alt, movement.max_alt);
    transform.translation.y = movement.altitude;

    // Границы и коллизия со структурами обрабатываются в map::collision::scout_collision
}

/// Спавн скаута (цветной куб-заглушка).
pub fn spawn_scout(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<MapGrid>,
    spawn_points: Res<crate::map::loader::MapSpawnPoints>,
) {
    use super::components::{PlayerScout, ScoutMoveIntent, ScoutMovement};
    use crate::camera::systems::CameraTarget;

    let movement = ScoutMovement::default();
    let (sx, sy) = spawn_points.player_spawn;
    let world_pos = map.grid_to_world(sx, sy);

    commands.spawn((
        Name::new("PlayerScout"),
        PlayerScout,
        CameraTarget,
        movement.clone(),
        ScoutMoveIntent::default(),
        crate::core::Health::new(100.0),
        crate::core::Team::Player,
        Mesh3d(meshes.add(Cuboid::new(0.8, 0.5, 0.8))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 1.0),
            ..default()
        })),
        Transform::from_translation(world_pos.with_y(movement.altitude)),
    ));
}

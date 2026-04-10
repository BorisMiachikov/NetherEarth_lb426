use bevy::prelude::*;

use crate::{camera::systems::IsometricCamera, map::grid::MapGrid};

use super::components::{PlayerScout, ScoutMoveIntent, ScoutMovement};

/// Перемещает скаута согласно ScoutMoveIntent, зажимает по границам карты и высоте.
/// Направление движения WASD вычисляется относительно текущего yaw камеры.
pub fn move_scout(
    time: Res<Time>,
    _map: Res<MapGrid>,
    camera: Query<&IsometricCamera>,
    mut query: Query<(&mut Transform, &mut ScoutMovement, &ScoutMoveIntent), With<PlayerScout>>,
) {
    let Ok((mut transform, mut movement, intent)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let speed = movement.speed;

    // Поворачиваем вектор движения на yaw камеры чтобы WASD был относительным
    let yaw = camera
        .single()
        .map(|c| c.yaw.to_radians())
        .unwrap_or(std::f32::consts::FRAC_PI_4); // 45° по умолчанию
    let (sin_y, cos_y) = yaw.sin_cos();
    let h = intent.horizontal; // h.x = A/D, h.y = W/S (W=-1)
    let world_x = h.y * sin_y + h.x * cos_y;
    let world_z = h.y * cos_y - h.x * sin_y;

    // Горизонтальное движение (XZ)
    transform.translation.x += world_x * speed * dt;
    transform.translation.z += world_z * speed * dt;

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

    // Корпус скаута — плоский гексагональный базис (низкий цилиндр 6-гранник)
    let body_mesh = meshes.add(Cylinder::new(0.55, 0.25).mesh().resolution(6));
    let body_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.6, 1.0),
        metallic: 0.4,
        perceptual_roughness: 0.4,
        ..default()
    });

    // Купол — полу-прозрачная сфера (иллюзия кабины)
    let dome_mesh = meshes.add(Sphere::new(0.3));
    let dome_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.5, 0.85, 1.0, 0.85),
        emissive: LinearRgba::rgb(0.1, 0.25, 0.35),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Светящийся нижний диск (иллюзия антиграва)
    let glow_mesh = meshes.add(Cylinder::new(0.4, 0.04));
    let glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.9, 1.0),
        emissive: LinearRgba::rgb(1.5, 4.5, 5.0),
        ..default()
    });

    commands
        .spawn((
            Name::new("PlayerScout"),
            PlayerScout,
            CameraTarget,
            movement.clone(),
            ScoutMoveIntent::default(),
            crate::core::Health::new(100.0),
            crate::core::Team::Player,
            Mesh3d(body_mesh),
            MeshMaterial3d(body_mat),
            Transform::from_translation(world_pos.with_y(movement.altitude)),
        ))
        .with_children(|parent| {
            // Купол сверху
            parent.spawn((
                Mesh3d(dome_mesh),
                MeshMaterial3d(dome_mat),
                Transform::from_xyz(0.0, 0.15, 0.0),
            ));
            // Светящийся диск снизу
            parent.spawn((
                Mesh3d(glow_mesh),
                MeshMaterial3d(glow_mat),
                Transform::from_xyz(0.0, -0.18, 0.0),
            ));
        });
}

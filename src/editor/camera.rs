use bevy::{
    camera::ScalingMode,
    ecs::message::MessageReader,
    input::mouse::MouseWheel,
    prelude::*,
};

use crate::{
    camera::systems::{ZOOM_DEFAULT, ZOOM_MAX, ZOOM_MIN, ZOOM_SPEED},
    editor::EditorEntity,
};

const CAMERA_PITCH_DEG: f32 = -35.264;
const CAMERA_DISTANCE: f32  = 40.0;
const CAMERA_MOVE_SPEED: f32 = 20.0;

/// Маркер: изометрическая камера редактора.
#[derive(Component)]
pub struct EditorCamera {
    pub viewport_height: f32,
    pub yaw: f32,
}

fn make_rotation(yaw_deg: f32) -> Quat {
    Quat::from_euler(
        EulerRot::YXZ,
        yaw_deg.to_radians(),
        CAMERA_PITCH_DEG.to_radians(),
        0.0,
    )
}

/// Спавн изометрической камеры при входе в Editor.
pub fn spawn_editor_camera(mut commands: Commands) {
    let yaw = 45.0_f32;
    let rotation = make_rotation(yaw);
    let start_pos = Vec3::new(32.0, 0.0, 32.0);

    commands.spawn((
        Name::new("EditorCamera"),
        EditorEntity,
        EditorCamera {
            viewport_height: ZOOM_DEFAULT,
            yaw,
        },
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: ZOOM_DEFAULT,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(start_pos + rotation * Vec3::new(0.0, 0.0, CAMERA_DISTANCE))
            .with_rotation(rotation),
    ));
}

/// Свободное перемещение камеры: WASD + зум колесом.
/// Нет слежения за скаутом, нет ограничения по высоте.
pub fn free_camera_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll: MessageReader<MouseWheel>,
    mut camera: Query<(&mut Transform, &mut Projection, &mut EditorCamera)>,
) {
    let Ok((mut tf, mut proj, mut cam)) = camera.single_mut() else {
        return;
    };

    // Зум
    let scroll_delta: f32 = scroll.read().map(|e| e.y).sum();
    if scroll_delta != 0.0 {
        cam.viewport_height =
            (cam.viewport_height - scroll_delta * ZOOM_SPEED).clamp(ZOOM_MIN, ZOOM_MAX);
        if let Projection::Orthographic(ref mut ortho) = *proj {
            ortho.scaling_mode = ScalingMode::FixedVertical {
                viewport_height: cam.viewport_height,
            };
        }
    }

    // Движение WASD в плоскости карты (camera-relative)
    let dt = time.delta_secs();
    let (sin_y, cos_y) = cam.yaw.to_radians().sin_cos();

    let mut dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { dir.z -= 1.0; }
    if keys.pressed(KeyCode::KeyS) { dir.z += 1.0; }
    if keys.pressed(KeyCode::KeyA) { dir.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { dir.x += 1.0; }

    if dir.length_squared() > 0.0 {
        let world_x = dir.z * sin_y + dir.x * cos_y;
        let world_z = dir.z * cos_y - dir.x * sin_y;
        let move_vec = Vec3::new(world_x, 0.0, world_z).normalize() * CAMERA_MOVE_SPEED * dt;

        let rotation = make_rotation(cam.yaw);
        let desired = tf.translation + move_vec;
        tf.translation = desired;
        // Пересчитываем позицию относительно нового pivot
        let pivot = tf.translation - rotation * Vec3::new(0.0, 0.0, CAMERA_DISTANCE);
        tf.translation = pivot + rotation * Vec3::new(0.0, 0.0, CAMERA_DISTANCE);
        tf.rotation = rotation;
    }

    // Вращение Z/C
    if keys.pressed(KeyCode::KeyZ) {
        cam.yaw += 30.0 * dt;
        tf.rotation = make_rotation(cam.yaw);
    }
    if keys.pressed(KeyCode::KeyC) {
        cam.yaw -= 30.0 * dt;
        tf.rotation = make_rotation(cam.yaw);
    }
}

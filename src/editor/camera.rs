use bevy::{
    camera::ScalingMode,
    ecs::message::MessageReader,
    input::mouse::MouseWheel,
    prelude::*,
};

use crate::camera::systems::{IsometricCamera, ZOOM_DEFAULT, ZOOM_MAX, ZOOM_MIN, ZOOM_SPEED};

const CAMERA_PITCH_DEG: f32 = -35.264;
const CAMERA_DISTANCE: f32  = 40.0;
const CAMERA_MOVE_SPEED: f32 = 20.0;

/// Маркер-компонент, навешиваемый на IsometricCamera при входе в AppState::Editor.
/// Не является отдельной сущностью — только указывает, что IsometricCamera сейчас в режиме редактора.
/// Используется в pick_cell и free_camera_movement для фильтрации запросов.
#[derive(Component)]
pub struct EditorCamera;

fn make_rotation(yaw_deg: f32) -> Quat {
    Quat::from_euler(
        EulerRot::YXZ,
        yaw_deg.to_radians(),
        CAMERA_PITCH_DEG.to_radians(),
        0.0,
    )
}

/// Сбрасывает IsometricCamera в центр карты при входе в редактор.
pub fn reset_camera_for_editor(
    mut camera: Query<(&mut Transform, &mut IsometricCamera)>,
) {
    let Ok((mut tf, mut cam)) = camera.single_mut() else { return };
    cam.yaw = 45.0;
    let rotation = make_rotation(cam.yaw);
    let center = Vec3::new(32.0, 0.0, 32.0);
    tf.translation = center + rotation * Vec3::new(0.0, 0.0, CAMERA_DISTANCE);
    tf.rotation = rotation;
}

/// Свободное перемещение IsometricCamera в режиме редактора: WASD + зум колесом.
/// Запускается только когда IsometricCamera имеет маркер EditorCamera.
pub fn free_camera_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll: MessageReader<MouseWheel>,
    // Запрашиваем IsometricCamera с маркером EditorCamera
    mut camera: Query<(&mut Transform, &mut Projection, &mut IsometricCamera), With<EditorCamera>>,
) {
    let Ok((mut tf, mut proj, mut cam)) = camera.single_mut() else {
        return;
    };

    // Зум колесом
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

    // Движение WASD
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
        let pivot = tf.translation - rotation * Vec3::new(0.0, 0.0, CAMERA_DISTANCE);
        let new_pivot = pivot + move_vec;
        tf.translation = new_pivot + rotation * Vec3::new(0.0, 0.0, CAMERA_DISTANCE);
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

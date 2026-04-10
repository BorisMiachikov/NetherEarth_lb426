use bevy::{
    camera::ScalingMode,
    ecs::message::MessageReader,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

// Угол pitch изометрической камеры (классика: -35.264°)
const CAMERA_PITCH_DEG: f32 = -35.264;
const CAMERA_DISTANCE: f32 = 40.0;

pub const ZOOM_MIN: f32 = 5.0;
pub const ZOOM_MAX: f32 = 80.0;
pub const ZOOM_SPEED: f32 = 2.5;
pub const ZOOM_DEFAULT: f32 = 20.0;

/// Чувствительность вращения (градусов на пиксель при перетаскивании мышью)
const ROTATE_SENSITIVITY: f32 = 0.4;
/// Скорость вращения клавишами Z/C (градусов в секунду)
const ROTATE_KEY_SPEED: f32 = 30.0;

/// Маркер: изометрическая камера.
#[derive(Component)]
pub struct IsometricCamera {
    pub viewport_height: f32,
    /// Текущий угол yaw в градусах (вокруг вертикальной оси).
    pub yaw: f32,
}

/// Маркер на сущности, за которой следит камера.
#[derive(Component)]
pub struct CameraTarget;

fn make_rotation(yaw_deg: f32) -> Quat {
    Quat::from_euler(
        EulerRot::YXZ,
        yaw_deg.to_radians(),
        CAMERA_PITCH_DEG.to_radians(),
        0.0,
    )
}

/// Спавн камеры при старте. Позиционируется в центре карты по умолчанию.
pub fn spawn_camera(mut commands: Commands) {
    let yaw = 45.0_f32;
    let rotation = make_rotation(yaw);
    let start_pos = Vec3::new(32.0, 0.0, 32.0); // центр карты 64×64

    commands.spawn((
        Name::new("IsometricCamera"),
        IsometricCamera {
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

/// Орбитальное вращение камеры:
/// - средняя кнопка мыши + движение → плавное вращение
/// - Z → влево, C → вправо
pub fn rotate_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
    mut camera: Query<&mut IsometricCamera>,
) {
    let Ok(mut cam) = camera.single_mut() else {
        motion.clear();
        return;
    };

    // Вращение мышью (средняя кнопка)
    if mouse_buttons.pressed(MouseButton::Middle) {
        let delta_x: f32 = motion.read().map(|e| e.delta.x).sum();
        cam.yaw -= delta_x * ROTATE_SENSITIVITY;
    } else {
        motion.clear();
    }

    // Вращение клавишами Z / C
    let dt = time.delta_secs();
    if keys.pressed(KeyCode::KeyZ) {
        cam.yaw += ROTATE_KEY_SPEED * dt;
    }
    if keys.pressed(KeyCode::KeyC) {
        cam.yaw -= ROTATE_KEY_SPEED * dt;
    }
}

/// Камера следует за сущностью с `CameraTarget` (PostUpdate).
/// Использует lerp для устранения дрожания при рассинхроне FixedUpdate/PostUpdate.
pub fn follow_target(
    time: Res<Time>,
    target: Query<&Transform, (With<CameraTarget>, Without<IsometricCamera>)>,
    mut camera: Query<(&mut Transform, &IsometricCamera)>,
) {
    let Ok(target_tf) = target.single() else {
        return;
    };
    let Ok((mut cam_tf, cam)) = camera.single_mut() else {
        return;
    };

    let rotation = make_rotation(cam.yaw);
    let desired = target_tf.translation + rotation * Vec3::new(0.0, 0.0, CAMERA_DISTANCE);

    // Высокий коэффициент (~20) — почти мгновенно, но без резких скачков
    let alpha = (20.0 * time.delta_secs()).min(1.0);
    cam_tf.translation = cam_tf.translation.lerp(desired, alpha);
    cam_tf.rotation = rotation;
}

/// Зум колесом мыши — меняет `viewport_height`.
pub fn zoom_camera(
    mut scroll: MessageReader<MouseWheel>,
    mut camera: Query<(&mut Projection, &mut IsometricCamera)>,
) {
    let delta: f32 = scroll.read().map(|e| e.y).sum();
    if delta == 0.0 {
        return;
    }

    let Ok((mut proj, mut cam)) = camera.single_mut() else {
        return;
    };

    cam.viewport_height = (cam.viewport_height - delta * ZOOM_SPEED).clamp(ZOOM_MIN, ZOOM_MAX);

    if let Projection::Orthographic(ref mut ortho) = *proj {
        ortho.scaling_mode = ScalingMode::FixedVertical {
            viewport_height: cam.viewport_height,
        };
    }
}

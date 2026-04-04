use bevy::prelude::*;

use super::components::{PlayerScout, ScoutMoveIntent};

/// Читает WASD+QE и записывает ScoutMoveIntent.
/// W/S — ось Z, A/D — ось X, Q/E — высота.
pub fn read_scout_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut ScoutMoveIntent, With<PlayerScout>>,
) {
    let Ok(mut intent) = query.single_mut() else {
        return;
    };

    let mut h = Vec2::ZERO;
    let mut v = 0.0_f32;

    if keys.pressed(KeyCode::KeyW) {
        h.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        h.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        h.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        h.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyE) {
        v += 1.0;
    }
    if keys.pressed(KeyCode::KeyQ) {
        v -= 1.0;
    }

    intent.horizontal = if h != Vec2::ZERO { h.normalize() } else { Vec2::ZERO };
    intent.vertical = v;
}

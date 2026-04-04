use std::f32::consts::PI;

use bevy::prelude::*;

use crate::map::grid::{MapGrid, CELL_SIZE};

/// Ресурс: переключатель отображения gizmos.
#[derive(Resource, Default)]
pub struct DebugGizmosState {
    pub show_grid: bool,
    pub show_bounds: bool,
}

/// Переключение клавишей F3 (сетка) и F4 (границы).
pub fn toggle_gizmos(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DebugGizmosState>,
) {
    if keys.just_pressed(KeyCode::F3) {
        state.show_grid = !state.show_grid;
    }
    if keys.just_pressed(KeyCode::F4) {
        state.show_bounds = !state.show_bounds;
    }
}

/// Рисует линии сетки и границы карты через Gizmos.
pub fn draw_debug_gizmos(
    mut gizmos: Gizmos,
    map: Res<MapGrid>,
    state: Res<DebugGizmosState>,
) {
    if state.show_grid {
        let w = map.width as f32 * CELL_SIZE;
        let h = map.height as f32 * CELL_SIZE;

        // Рисуем сетку в плоскости XZ (вращение PI/2 вокруг X)
        gizmos.grid(
            Isometry3d::new(
                Vec3::new(w * 0.5, 0.01, h * 0.5),
                Quat::from_rotation_x(PI / 2.0),
            ),
            UVec2::new(map.width, map.height),
            Vec2::splat(CELL_SIZE),
            LinearRgba::new(0.3, 0.6, 0.3, 0.4),
        );
    }

    if state.show_bounds {
        let w = map.width as f32 * CELL_SIZE;
        let h = map.height as f32 * CELL_SIZE;
        let y = 0.05;

        // Периметр карты
        gizmos.line(Vec3::new(0.0, y, 0.0), Vec3::new(w, y, 0.0), Color::srgb(1.0, 0.3, 0.3));
        gizmos.line(Vec3::new(w, y, 0.0), Vec3::new(w, y, h), Color::srgb(1.0, 0.3, 0.3));
        gizmos.line(Vec3::new(w, y, h), Vec3::new(0.0, y, h), Color::srgb(1.0, 0.3, 0.3));
        gizmos.line(Vec3::new(0.0, y, h), Vec3::new(0.0, y, 0.0), Color::srgb(1.0, 0.3, 0.3));
    }
}

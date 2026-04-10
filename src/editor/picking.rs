use bevy::prelude::*;

use crate::{editor::camera::EditorCamera, map::grid::MapGrid};

use super::state::EditorState;

/// Определяет клетку карты под курсором через raycast к плоскости Y=0.
/// Результат сохраняется в EditorState::hovered_cell.
pub fn pick_cell(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    grid: Res<MapGrid>,
    mut editor: ResMut<EditorState>,
) {
    let Ok((camera, cam_tf)) = camera_q.single() else {
        editor.hovered_cell = None;
        return;
    };
    let Ok(window) = windows.single() else {
        editor.hovered_cell = None;
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        editor.hovered_cell = None;
        return;
    };

    // Ray из камеры через экранные координаты
    let Some(ray) = camera.viewport_to_world(cam_tf, cursor_pos).ok() else {
        editor.hovered_cell = None;
        return;
    };

    // Пересечение с плоскостью Y = 0
    let denom = ray.direction.y;
    if denom.abs() < 1e-6 {
        editor.hovered_cell = None;
        return;
    }
    let t = -ray.origin.y / denom;
    if t < 0.0 {
        editor.hovered_cell = None;
        return;
    }
    let hit = ray.origin + ray.direction * t;
    editor.hovered_cell = grid.world_to_grid(hit);
}

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::{
    core::time::GameTime,
    map::grid::MapGrid,
    player::components::{PlayerScout, ScoutMovement},
};

/// Показывает debug-overlay с позицией скаута, временем, FPS, ячейкой под скаутом.
pub fn debug_overlay(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    game_time: Res<GameTime>,
    map: Res<MapGrid>,
    scout: Query<(&Transform, &ScoutMovement), With<PlayerScout>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let (pos, grid_cell, altitude) = if let Ok((tf, mv)) = scout.single() {
        let cell = map
            .world_to_grid(tf.translation)
            .map(|(x, y)| format!("({x}, {y})"))
            .unwrap_or_else(|| "вне карты".to_string());
        (tf.translation, cell, mv.altitude)
    } else {
        (Vec3::ZERO, "—".to_string(), 0.0)
    };

    egui::Window::new("Debug")
        .default_pos([10.0, 10.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(format!("FPS: {fps:.1}"));
            ui.separator();
            ui.label(format!(
                "Позиция: ({:.1}, {:.1}, {:.1})",
                pos.x, pos.y, pos.z
            ));
            ui.label(format!("Высота: {altitude:.2}"));
            ui.label(format!("Ячейка: {grid_cell}"));
            ui.separator();
            ui.label(format!(
                "День: {}  ({:.1}s / {}s)",
                game_time.game_day, game_time.day_elapsed, game_time.seconds_per_day as u32
            ));
        });

    Ok(())
}


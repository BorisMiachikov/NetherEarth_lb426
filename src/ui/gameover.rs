use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::ai::state::{GameOutcome, GameResult};

/// Отображает экран победы / поражения.
pub fn draw_gameover_screen(
    result: Res<GameResult>,
    mut contexts: EguiContexts,
) -> Result {
    let Some(outcome) = result.outcome else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;

    let (title, color) = match outcome {
        GameOutcome::PlayerWin => ("ПОБЕДА", egui::Color32::from_rgb(80, 220, 80)),
        GameOutcome::PlayerLose => ("ПОРАЖЕНИЕ", egui::Color32::from_rgb(220, 60, 60)),
    };

    egui::Window::new("Итог игры")
        .id(egui::Id::new("gameover"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(title)
                        .color(color)
                        .size(36.0)
                        .strong(),
                );
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                egui::Grid::new("stats_grid")
                    .num_columns(2)
                    .spacing([24.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Прошло дней:");
                        ui.label(format!("{}", result.game_days));
                        ui.end_row();

                        ui.label("Фабрики игрока:");
                        ui.label(format!("{}", result.player_factories));
                        ui.end_row();

                        ui.label("Фабрики врага:");
                        ui.label(format!("{}", result.enemy_factories));
                        ui.end_row();
                    });

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("Закройте окно или перезапустите игру.")
                        .color(egui::Color32::GRAY),
                );
                ui.add_space(8.0);
            });
        });

    Ok(())
}

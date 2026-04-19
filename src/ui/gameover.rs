use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    ai::state::{GameOutcome, GameResult},
    app::state::AppState,
    localization::Localization,
};

/// Отображает экран победы / поражения.
pub fn draw_gameover_screen(
    result: Res<GameResult>,
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut time: ResMut<Time<Virtual>>,
    mut exit: MessageWriter<AppExit>,
    loc: Res<Localization>,
) -> Result {
    let Some(outcome) = result.outcome else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;

    // Затемнение
    egui::Area::new(egui::Id::new("gameover_overlay"))
        .fixed_pos(egui::Pos2::ZERO)
        .order(egui::Order::Background)
        .interactable(false)
        .show(ctx, |ui| {
            let screen = ui.ctx().viewport_rect();
            ui.painter()
                .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(160));
        });

    let (title_key, title_color) = match outcome {
        GameOutcome::PlayerWin => (
            "gameover.victory",
            egui::Color32::from_rgb(80, 220, 80),
        ),
        GameOutcome::PlayerLose => (
            "gameover.defeat",
            egui::Color32::from_rgb(220, 60, 60),
        ),
    };

    egui::Window::new("##gameover_win")
        .id(egui::Id::new("gameover"))
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgb(12, 15, 22))
                .stroke(egui::Stroke::new(2.0, title_color)),
        )
        .show(ctx, |ui| {
            ui.set_min_width(260.0);
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(loc.t(title_key))
                        .color(title_color)
                        .size(38.0)
                        .strong(),
                );
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(10.0);

                egui::Grid::new("stats_grid")
                    .num_columns(2)
                    .spacing([32.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(loc.t("gameover.days"));
                        ui.label(
                            egui::RichText::new(format!("{}", result.game_days)).strong(),
                        );
                        ui.end_row();

                        ui.label(loc.t("gameover.your_factories"));
                        ui.label(
                            egui::RichText::new(format!("{}", result.player_factories))
                                .color(egui::Color32::from_rgb(60, 140, 255)),
                        );
                        ui.end_row();

                        ui.label(loc.t("gameover.enemy_factories"));
                        ui.label(
                            egui::RichText::new(format!("{}", result.enemy_factories))
                                .color(egui::Color32::from_rgb(220, 60, 60)),
                        );
                        ui.end_row();
                    });

                ui.add_space(20.0);

                if ui
                    .add_sized([200.0, 36.0], egui::Button::new(loc.t("gameover.main_menu")))
                    .clicked()
                {
                    time.unpause();
                    next_state.set(AppState::MainMenu);
                }
                ui.add_space(8.0);
                if ui
                    .add_sized([200.0, 36.0], egui::Button::new(loc.t("gameover.quit")))
                    .clicked()
                {
                    exit.write(AppExit::Success);
                }
                ui.add_space(12.0);
            });
        });

    Ok(())
}

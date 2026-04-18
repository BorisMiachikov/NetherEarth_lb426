use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    app::state::AppState,
    localization::{ChangeLanguage, Language, Localization},
};

use super::save_slots::{draw_load_panel, draw_save_panel};

/// Какая вкладка открыта в меню паузы.
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
pub enum PauseSubPanel {
    #[default]
    None,
    Save,
    Load,
}

/// Экран паузы (ESC во время игры).
pub fn draw_pause_menu(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut contexts: EguiContexts,
    mut time: ResMut<Time<Virtual>>,
    mut exit: MessageWriter<AppExit>,
    loc: Res<Localization>,
    mut sub_panel: Local<PauseSubPanel>,
    mut commands: Commands,
) -> Result {
    if *state.get() != AppState::Paused {
        *sub_panel = PauseSubPanel::None;
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Area::new(egui::Id::new("pause_overlay"))
        .fixed_pos(egui::Pos2::ZERO)
        .order(egui::Order::Background)
        .interactable(false)
        .show(ctx, |ui| {
            let screen = ui.ctx().viewport_rect();
            ui.painter().rect_filled(screen, 0.0, egui::Color32::from_black_alpha(160));
        });

    egui::Window::new("##pause_win")
        .id(egui::Id::new("pause_menu"))
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgb(15, 20, 30))
                .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(60, 120, 200))),
        )
        .show(ctx, |ui| {
            ui.set_min_width(240.0);
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(loc.t("pause.title"))
                        .size(22.0)
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 200)),
                );
                ui.add_space(12.0);
            });

            match *sub_panel {
                PauseSubPanel::None => draw_pause_buttons(ui, &loc, &mut sub_panel, &mut time, &mut next_state, &mut exit, &mut commands),
                PauseSubPanel::Save => draw_save_panel(ui, &loc, &mut sub_panel, &mut commands),
                PauseSubPanel::Load => draw_load_panel(ui, &loc, &mut sub_panel, &mut commands, &mut time, &mut next_state),
            }
        });

    Ok(())
}

fn draw_pause_buttons(
    ui: &mut egui::Ui,
    loc: &Localization,
    sub_panel: &mut PauseSubPanel,
    time: &mut Time<Virtual>,
    next_state: &mut NextState<AppState>,
    exit: &mut MessageWriter<AppExit>,
    commands: &mut Commands,
) {
    ui.vertical_centered(|ui| {
        if ui.add_sized([190.0, 34.0], egui::Button::new(loc.t("pause.continue"))).clicked() {
            time.unpause();
            next_state.set(AppState::Playing);
        }
        ui.add_space(6.0);
        if ui.add_sized([190.0, 34.0], egui::Button::new(loc.t("pause.save"))).clicked() {
            *sub_panel = PauseSubPanel::Save;
        }
        ui.add_space(4.0);
        if ui.add_sized([190.0, 34.0], egui::Button::new(loc.t("pause.load"))).clicked() {
            *sub_panel = PauseSubPanel::Load;
        }
        ui.add_space(6.0);
        if ui.add_sized([190.0, 34.0], egui::Button::new(loc.t("pause.main_menu"))).clicked() {
            time.unpause();
            *sub_panel = PauseSubPanel::None;
            next_state.set(AppState::MainMenu);
        }
        ui.add_space(4.0);
        if ui.add_sized([190.0, 34.0], egui::Button::new(loc.t("pause.quit"))).clicked() {
            exit.write(AppExit::Success);
        }
        ui.add_space(10.0);

        ui.separator();
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(loc.t("settings.language")).color(egui::Color32::GRAY).size(12.0));
            ui.add_space(6.0);
            for lang in [Language::Russian, Language::English] {
                let active = loc.language == lang;
                let btn = egui::Button::new(
                    egui::RichText::new(lang.label())
                        .color(if active { egui::Color32::from_rgb(80, 190, 255) } else { egui::Color32::GRAY }),
                );
                if ui.add_enabled(!active, btn).clicked() {
                    commands.trigger(ChangeLanguage(lang));
                }
            }
        });
        ui.add_space(6.0);

        ui.separator();
        ui.label(egui::RichText::new(loc.t("pause.hint")).size(11.0).color(egui::Color32::DARK_GRAY));
        ui.add_space(6.0);
    });
}

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    app::state::AppState,
    localization::Localization,
    save::{io::autosave_info, systems::{TriggerLoadAutosave, TriggerNewGame}},
};

use super::scenario_picker::ScenarioList;

/// Система запуска: Loading → MainMenu.
pub fn init_to_main_menu(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::MainMenu);
}

/// Переключение паузы по ESC.
pub fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut time: ResMut<Time<Virtual>>,
    playtest: Option<Res<crate::editor::EditorPlaytest>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    if playtest.is_some() {
        return;
    }
    match state.get() {
        AppState::Playing => {
            time.pause();
            next_state.set(AppState::Paused);
        }
        AppState::Paused => {
            time.unpause();
            next_state.set(AppState::Playing);
        }
        _ => {}
    }
}

/// Главное меню (показывается поверх мира при AppState::MainMenu).
pub fn draw_main_menu(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut contexts: EguiContexts,
    mut scenarios: ResMut<ScenarioList>,
    mut exit: MessageWriter<AppExit>,
    loc: Res<Localization>,
    mut commands: Commands,
) -> Result {
    if *state.get() != AppState::MainMenu {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Area::new(egui::Id::new("menu_overlay"))
        .fixed_pos(egui::Pos2::ZERO)
        .order(egui::Order::Background)
        .interactable(false)
        .show(ctx, |ui| {
            let screen = ui.ctx().viewport_rect();
            ui.painter().rect_filled(screen, 0.0, egui::Color32::from_black_alpha(210));
        });

    egui::Window::new("##main_menu_win")
        .id(egui::Id::new("main_menu"))
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
            ui.set_width(260.0);
            draw_title(ui, &loc);
            ui.separator();
            ui.add_space(6.0);
            draw_scenario_picker(ui, &loc, &mut scenarios);
            ui.add_space(14.0);
            draw_menu_buttons(ui, &loc, &mut scenarios, &mut next_state, &mut exit, &mut commands);
        });

    Ok(())
}

fn draw_title(ui: &mut egui::Ui, loc: &Localization) {
    ui.vertical_centered(|ui| {
        ui.add_space(16.0);
        ui.label(
            egui::RichText::new(loc.t("menu.title"))
                .size(30.0)
                .strong()
                .color(egui::Color32::from_rgb(80, 190, 255)),
        );
        ui.label(
            egui::RichText::new(loc.t("menu.subtitle"))
                .size(13.0)
                .color(egui::Color32::GRAY),
        );
        ui.add_space(16.0);
    });
}

fn draw_scenario_picker(ui: &mut egui::Ui, loc: &Localization, scenarios: &mut ScenarioList) {
    ui.label(egui::RichText::new(loc.t("menu.scenario")).small().color(egui::Color32::DARK_GRAY));
    ui.add_space(2.0);

    let scenario_count = scenarios.scenarios.len();
    if scenario_count > 1 {
        ui.horizontal(|ui| {
            let prev_ok = scenarios.selected > 0;
            if ui.add_enabled(prev_ok, egui::Button::new("◀")).clicked() {
                scenarios.selected -= 1;
            }
            let name_w = 200.0 - 28.0 * 2.0;
            ui.add_sized(
                [name_w, 18.0],
                egui::Label::new(egui::RichText::new(&scenarios.current().name).strong().color(egui::Color32::WHITE)),
            );
            let next_ok = scenarios.selected + 1 < scenario_count;
            if ui.add_enabled(next_ok, egui::Button::new("▶")).clicked() {
                scenarios.selected += 1;
            }
        });
        ui.add(egui::Label::new(
            egui::RichText::new(&scenarios.current().description).small().color(egui::Color32::GRAY),
        ).wrap());
    } else if let Some(s) = scenarios.scenarios.first() {
        ui.label(egui::RichText::new(&s.name).color(egui::Color32::WHITE));
    }
}

fn draw_menu_buttons(
    ui: &mut egui::Ui,
    loc: &Localization,
    scenarios: &mut ScenarioList,
    next_state: &mut NextState<AppState>,
    exit: &mut MessageWriter<AppExit>,
    commands: &mut Commands,
) {
    ui.vertical_centered(|ui| {
        if let Some(autosave_day) = autosave_info() {
            let label = format!("{} ({}{})", loc.t("menu.continue"), loc.t("save.day"), autosave_day);
            if ui.add_sized([200.0, 38.0], egui::Button::new(label)).clicked() {
                commands.trigger(TriggerLoadAutosave);
                next_state.set(AppState::Playing);
            }
            ui.add_space(6.0);
        }

        if ui.add_sized([200.0, 38.0], egui::Button::new(loc.t("menu.new_game"))).clicked() {
            commands.trigger(TriggerNewGame);
            if let Some(ir) = scenarios.current().initial_resources.clone() {
                commands.insert_resource(crate::economy::resource::PlayerResources::from_scenario(&ir));
            }
            next_state.set(AppState::Playing);
        }
        ui.add_space(8.0);
        if ui.add_sized([200.0, 38.0], egui::Button::new(loc.t("menu.editor"))).clicked() {
            next_state.set(AppState::Editor);
        }
        ui.add_space(8.0);
        if ui.add_sized([200.0, 38.0], egui::Button::new(loc.t("menu.quit"))).clicked() {
            exit.write(AppExit::Success);
        }
        ui.add_space(14.0);
    });
}

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    localization::Localization,
    save::{
        io::slot_info,
        systems::{TriggerLoad, TriggerSave},
        SAVE_SLOT_COUNT,
    },
    app::state::AppState,
};

use super::pause::PauseSubPanel;

/// Панель сохранения: список слотов с кнопками записи.
pub fn draw_save_panel(
    ui: &mut egui::Ui,
    loc: &Localization,
    sub_panel: &mut PauseSubPanel,
    commands: &mut Commands,
) {
    ui.label(
        egui::RichText::new(loc.t("pause.save"))
            .color(egui::Color32::from_rgb(80, 190, 255))
            .size(14.0),
    );
    ui.add_space(6.0);

    for slot in 0..SAVE_SLOT_COUNT {
        let label = if let Some((day, _ts)) = slot_info(slot) {
            format!("{} {} — {} {}", loc.t("save.slot"), slot + 1, loc.t("save.day"), day)
        } else {
            format!("{} {} — {}", loc.t("save.slot"), slot + 1, loc.t("save.empty"))
        };
        if ui.add_sized([190.0, 28.0], egui::Button::new(label)).clicked() {
            commands.trigger(TriggerSave { slot });
        }
        ui.add_space(2.0);
    }

    ui.add_space(8.0);
    if ui.button(loc.t("menu.btn.back")).clicked() {
        *sub_panel = PauseSubPanel::None;
    }
    ui.add_space(6.0);
}

/// Панель загрузки: список слотов с кнопками чтения.
pub fn draw_load_panel(
    ui: &mut egui::Ui,
    loc: &Localization,
    sub_panel: &mut PauseSubPanel,
    commands: &mut Commands,
    time: &mut Time<Virtual>,
    next_state: &mut NextState<AppState>,
) {
    ui.label(
        egui::RichText::new(loc.t("pause.load"))
            .color(egui::Color32::from_rgb(80, 190, 255))
            .size(14.0),
    );
    ui.add_space(6.0);

    for slot in 0..SAVE_SLOT_COUNT {
        let (label, has_save) = if let Some((day, _ts)) = slot_info(slot) {
            (format!("{} {} — {} {}", loc.t("save.slot"), slot + 1, loc.t("save.day"), day), true)
        } else {
            (format!("{} {} — {}", loc.t("save.slot"), slot + 1, loc.t("save.empty")), false)
        };
        if ui
            .add_enabled(has_save, egui::Button::new(label).min_size([190.0, 28.0].into()))
            .clicked()
        {
            commands.trigger(TriggerLoad { slot });
            time.unpause();
            *sub_panel = PauseSubPanel::None;
            next_state.set(AppState::Playing);
        }
        ui.add_space(2.0);
    }

    ui.add_space(8.0);
    if ui.button(loc.t("menu.btn.back")).clicked() {
        *sub_panel = PauseSubPanel::None;
    }
    ui.add_space(6.0);
}

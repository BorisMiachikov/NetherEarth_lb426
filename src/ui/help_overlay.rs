use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// Ресурс: показывать ли справку по клавишам.
#[derive(Resource, Default)]
pub struct HelpOverlayState {
    pub visible: bool,
}

/// Переключение справки по F1.
pub fn toggle_help_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<HelpOverlayState>,
) {
    if keys.just_pressed(KeyCode::F1) {
        state.visible = !state.visible;
    }
}

/// Отрисовка overlay с таблицей клавиш.
pub fn draw_help_overlay(
    mut state: ResMut<HelpOverlayState>,
    mut contexts: EguiContexts,
) -> Result {
    if !state.visible {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    let mut close = false;

    egui::Window::new("Управление  [F1]")
        .id(egui::Id::new("help_overlay"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            let header = egui::Color32::from_rgb(200, 200, 120);
            let key    = egui::Color32::from_rgb(120, 220, 255);
            let desc   = egui::Color32::from_rgb(200, 200, 200);

            macro_rules! section {
                ($title:expr) => {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new($title).strong().color(header));
                    ui.separator();
                };
            }

            macro_rules! row {
                ($k:expr, $d:expr) => {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new($k).color(key).monospace().strong());
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new($d).color(desc));
                    });
                };
            }

            section!("Скаут");
            row!("W / A / S / D",  "Движение скаута");
            row!("Q / E",          "Снизить / поднять высоту");
            row!("Scroll",         "Зум камеры");

            section!("Выбор роботов");
            row!("ЛКМ",            "Выбрать робота");
            row!("Shift + ЛКМ",   "Добавить к выбору");
            row!("Ctrl + 1-9",    "Сохранить группу");
            row!("1-9",            "Выбрать группу");

            section!("Команды роботов");
            row!("ПКМ",            "Двигаться к точке");
            row!("P + ПКМ",       "Патруль (2 точки)");

            section!("Строительство");
            row!("B",              "Открыть Builder (рядом с варбейсом)");

            section!("Прочее");
            row!("Esc",            "Пауза / Продолжить");
            row!("F1",             "Показать / скрыть справку");

            ui.add_space(8.0);
            if ui.button("Закрыть [F1]").clicked() {
                close = true;
            }
        });

    if close {
        state.visible = false;
    }

    Ok(())
}

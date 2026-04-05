use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use serde::Deserialize;

use crate::app::state::AppState;

// ---------------------------------------------------------------------------
// Ресурс: выбранный сценарий
// ---------------------------------------------------------------------------

/// Описание одного сценария (десериализуется из data/scenarios/*.ron).
#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioDef {
    pub name: String,
    pub description: String,
    pub map_path: String,
}

/// Список доступных сценариев и индекс выбранного.
#[derive(Resource)]
pub struct ScenarioList {
    pub scenarios: Vec<ScenarioDef>,
    pub selected: usize,
}

impl ScenarioList {
    /// Сканирует `data/scenarios/` и загружает все .ron файлы.
    pub fn load_from_dir() -> Self {
        let mut scenarios: Vec<ScenarioDef> = Vec::new();

        if let Ok(entries) = std::fs::read_dir("data/scenarios") {
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map_or(false, |ext| ext == "ron"))
                .collect();
            paths.sort();

            for path in paths {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match ron::from_str::<ScenarioDef>(&content) {
                        Ok(def) => scenarios.push(def),
                        Err(e) => warn!("Ошибка парсинга {:?}: {e}", path),
                    },
                    Err(e) => warn!("Не удалось прочитать {:?}: {e}", path),
                }
            }
        }

        if scenarios.is_empty() {
            // Фоллбэк: всегда есть дефолтная карта
            scenarios.push(ScenarioDef {
                name: "Стандартная схватка".into(),
                description: "8 нейтральных фабрик.".into(),
                map_path: "data/maps/default.ron".into(),
            });
        }

        Self {
            scenarios,
            selected: 0,
        }
    }

    pub fn current(&self) -> &ScenarioDef {
        &self.scenarios[self.selected]
    }
}

/// Ресурс: путь к карте активной игры (используется MapPlugin при Startup).
#[derive(Resource, Default)]
pub struct SelectedMapPath(pub String);

// ---------------------------------------------------------------------------
// Системы меню
// ---------------------------------------------------------------------------

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
) {
    if !keys.just_pressed(KeyCode::Escape) {
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
) -> Result {
    if *state.get() != AppState::MainMenu {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    // Полупрозрачный фон
    egui::Area::new(egui::Id::new("menu_overlay"))
        .fixed_pos(egui::Pos2::ZERO)
        .order(egui::Order::Background)
        .interactable(false)
        .show(ctx, |ui| {
            let screen = ui.ctx().viewport_rect();
            ui.painter()
                .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(210));
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
            ui.set_min_width(280.0);
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("NETHER EARTH")
                        .size(30.0)
                        .strong()
                        .color(egui::Color32::from_rgb(80, 190, 255)),
                );
                ui.label(
                    egui::RichText::new("LB426")
                        .size(13.0)
                        .color(egui::Color32::GRAY),
                );
                ui.add_space(20.0);
            });

            ui.separator();

            // --- Выбор сценария ---
            ui.label(
                egui::RichText::new("СЦЕНАРИЙ")
                    .small()
                    .color(egui::Color32::DARK_GRAY),
            );

            let scenario_count = scenarios.scenarios.len();
            if scenario_count > 1 {
                ui.horizontal(|ui| {
                    if ui.button("◀").clicked() && scenarios.selected > 0 {
                        scenarios.selected -= 1;
                    }
                    ui.vertical_centered(|ui| {
                        ui.set_min_width(180.0);
                        ui.label(
                            egui::RichText::new(&scenarios.current().name)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );
                    });
                    if ui.button("▶").clicked() && scenarios.selected + 1 < scenario_count {
                        scenarios.selected += 1;
                    }
                });
                ui.label(
                    egui::RichText::new(&scenarios.current().description)
                        .small()
                        .color(egui::Color32::GRAY),
                );
            } else {
                ui.label(
                    egui::RichText::new(
                        scenarios.scenarios.first().map_or("—", |s| &s.name),
                    )
                    .color(egui::Color32::WHITE),
                );
            }

            ui.add_space(16.0);

            ui.vertical_centered(|ui| {
                if ui
                    .add_sized([200.0, 38.0], egui::Button::new("▶  Новая игра"))
                    .clicked()
                {
                    next_state.set(AppState::Playing);
                }
                ui.add_space(8.0);
                if ui
                    .add_sized([200.0, 38.0], egui::Button::new("✕  Выход"))
                    .clicked()
                {
                    exit.write(AppExit::Success);
                }
                ui.add_space(14.0);
            });
        });

    Ok(())
}

/// Экран паузы (ESC во время игры).
pub fn draw_pause_menu(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut contexts: EguiContexts,
    mut time: ResMut<Time<Virtual>>,
    mut exit: MessageWriter<AppExit>,
) -> Result {
    if *state.get() != AppState::Paused {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Area::new(egui::Id::new("pause_overlay"))
        .fixed_pos(egui::Pos2::ZERO)
        .order(egui::Order::Background)
        .interactable(false)
        .show(ctx, |ui| {
            let screen = ui.ctx().viewport_rect();
            ui.painter()
                .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(160));
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
            ui.set_min_width(210.0);
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("ПАУЗА")
                        .size(22.0)
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 200)),
                );
                ui.add_space(20.0);

                if ui
                    .add_sized([170.0, 34.0], egui::Button::new("▶  Продолжить"))
                    .clicked()
                {
                    time.unpause();
                    next_state.set(AppState::Playing);
                }
                ui.add_space(8.0);
                if ui
                    .add_sized([170.0, 34.0], egui::Button::new("⌂  Главное меню"))
                    .clicked()
                {
                    time.unpause();
                    next_state.set(AppState::MainMenu);
                }
                ui.add_space(8.0);
                if ui
                    .add_sized([170.0, 34.0], egui::Button::new("✕  Выход"))
                    .clicked()
                {
                    exit.write(AppExit::Success);
                }
                ui.add_space(12.0);

                ui.separator();
                ui.label(
                    egui::RichText::new("ESC — продолжить")
                        .size(11.0)
                        .color(egui::Color32::DARK_GRAY),
                );
                ui.add_space(6.0);
            });
        });

    Ok(())
}

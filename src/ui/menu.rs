use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use serde::Deserialize;

use crate::{
    app::state::AppState,
    localization::{ChangeLanguage, Language, Localization},
    save::{
        io::{autosave_info, slot_info},
        systems::{TriggerLoad, TriggerLoadAutosave, TriggerNewGame, TriggerSave},
        SAVE_SLOT_COUNT,
    },
};

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

/// Какая вкладка открыта в меню паузы.
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
pub enum PauseSubPanel {
    #[default]
    None,
    Save,
    Load,
}

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
            ui.set_width(260.0);

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

            ui.separator();
            ui.add_space(6.0);

            ui.label(
                egui::RichText::new(loc.t("menu.scenario"))
                    .small()
                    .color(egui::Color32::DARK_GRAY),
            );
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
                        egui::Label::new(
                            egui::RichText::new(&scenarios.current().name)
                                .strong()
                                .color(egui::Color32::WHITE),
                        ),
                    );
                    let next_ok = scenarios.selected + 1 < scenario_count;
                    if ui.add_enabled(next_ok, egui::Button::new("▶")).clicked() {
                        scenarios.selected += 1;
                    }
                });
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(&scenarios.current().description)
                            .small()
                            .color(egui::Color32::GRAY),
                    )
                    .wrap(),
                );
            } else if let Some(s) = scenarios.scenarios.first() {
                ui.label(egui::RichText::new(&s.name).color(egui::Color32::WHITE));
            }

            ui.add_space(14.0);

            ui.vertical_centered(|ui| {
                // Кнопка "Продолжить" — только если есть автосохранение
                if let Some(autosave_day) = autosave_info() {
                    let label = format!("{} ({}{})", loc.t("menu.continue"), loc.t("save.day"), autosave_day);
                    if ui.add_sized([200.0, 38.0], egui::Button::new(label)).clicked() {
                        commands.trigger(TriggerLoadAutosave);
                        next_state.set(AppState::Playing);
                    }
                    ui.add_space(6.0);
                }

                if ui
                    .add_sized([200.0, 38.0], egui::Button::new(loc.t("menu.new_game")))
                    .clicked()
                {
                    commands.trigger(TriggerNewGame);
                    next_state.set(AppState::Playing);
                }
                ui.add_space(8.0);
                if ui
                    .add_sized([200.0, 38.0], egui::Button::new(loc.t("menu.quit")))
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
    loc: Res<Localization>,
    mut sub_panel: Local<PauseSubPanel>,
    mut commands: Commands,
) -> Result {
    if *state.get() != AppState::Paused {
        // Сброс вкладки при выходе из паузы
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
                PauseSubPanel::None => {
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

                        // Смена языка
                        ui.separator();
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(loc.t("settings.language")).color(egui::Color32::GRAY).size(12.0));
                            ui.add_space(6.0);
                            for lang in [Language::Russian, Language::English] {
                                let active = loc.language == lang;
                                let btn = egui::Button::new(
                                    egui::RichText::new(lang.label())
                                        .color(if active { egui::Color32::from_rgb(80, 190, 255) } else { egui::Color32::GRAY })
                                );
                                if ui.add_enabled(!active, btn).clicked() {
                                    commands.trigger(ChangeLanguage(lang));
                                }
                            }
                        });
                        ui.add_space(6.0);

                        ui.separator();
                        ui.label(
                            egui::RichText::new(loc.t("pause.hint"))
                                .size(11.0)
                                .color(egui::Color32::DARK_GRAY),
                        );
                        ui.add_space(6.0);
                    });
                }

                PauseSubPanel::Save => {
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
                    if ui.button("← Назад").clicked() {
                        *sub_panel = PauseSubPanel::None;
                    }
                    ui.add_space(6.0);
                }

                PauseSubPanel::Load => {
                    ui.label(
                        egui::RichText::new(loc.t("pause.load"))
                            .color(egui::Color32::from_rgb(80, 190, 255))
                            .size(14.0),
                    );
                    ui.add_space(6.0);

                    for slot in 0..SAVE_SLOT_COUNT {
                        let (label, has_save) = if let Some((day, _ts)) = slot_info(slot) {
                            (
                                format!("{} {} — {} {}", loc.t("save.slot"), slot + 1, loc.t("save.day"), day),
                                true,
                            )
                        } else {
                            (
                                format!("{} {} — {}", loc.t("save.slot"), slot + 1, loc.t("save.empty")),
                                false,
                            )
                        };
                        if ui.add_enabled(has_save, egui::Button::new(label).min_size([190.0, 28.0].into())).clicked() {
                            commands.trigger(TriggerLoad { slot });
                            time.unpause();
                            *sub_panel = PauseSubPanel::None;
                            next_state.set(AppState::Playing);
                        }
                        ui.add_space(2.0);
                    }

                    ui.add_space(8.0);
                    if ui.button("← Назад").clicked() {
                        *sub_panel = PauseSubPanel::None;
                    }
                    ui.add_space(6.0);
                }
            }
        });

    Ok(())
}

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    app::state::AppState,
    map::{grid::MapGrid, loader::{CellTypeDef, FactoryTypeDef, TeamDef}},
};

use super::{
    save::{load_map_into_editor, save_map},
    state::{BrushSize, EditorState, EditorTool, MapSize},
    terrain::RebuildTerrainCell,
};

/// Левая панель: инструменты.
pub fn draw_editor_toolbox(
    mut contexts: EguiContexts,
    mut editor: ResMut<EditorState>,
    mut grid: ResMut<MapGrid>,
    mut commands: Commands,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::SidePanel::left("editor_toolbox")
        .default_width(180.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Редактор карты");
            ui.separator();

            // --- Инструменты ---
            ui.label(egui::RichText::new("Инструмент").small().color(egui::Color32::GRAY));
            for (tool, label) in [
                (EditorTool::TerrainBrush,    "🖌 Рельеф"),
                (EditorTool::PlaceFactory,    "🏭 Фабрика"),
                (EditorTool::PlaceWarbase,    "🏰 Варбейс"),
                (EditorTool::PlacePlayerSpawn,"🚀 Спавн"),
                (EditorTool::Erase,           "✕ Стереть"),
            ] {
                ui.radio_value(&mut editor.current_tool, tool, label);
            }
            ui.add_space(6.0);

            // --- Настройки кисти рельефа ---
            if editor.current_tool == EditorTool::TerrainBrush {
                ui.separator();
                ui.label(egui::RichText::new("Тип клетки").small().color(egui::Color32::GRAY));
                for (ct, label) in [
                    (CellTypeDef::Rock,    "⬛ Скала"),
                    (CellTypeDef::Pit,     "🕳 Яма"),
                    (CellTypeDef::Sand,    "🟨 Песок"),
                    (CellTypeDef::Blocked, "🚫 Заблокировано"),
                ] {
                    let selected = std::mem::discriminant(&editor.brush_cell_type)
                        == std::mem::discriminant(&ct);
                    if ui.selectable_label(selected, label).clicked() {
                        editor.brush_cell_type = ct;
                    }
                }
                // Кнопка очистки: установить Open
                if ui.button("⬜ Очистить (Open)").clicked() {
                    // Используем специальный маркер — Open не является CellTypeDef,
                    // поэтому передаём сигнал через инструмент Erase
                    editor.current_tool = EditorTool::Erase;
                }
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Размер кисти").small().color(egui::Color32::GRAY));
                for sz in [BrushSize::One, BrushSize::Three, BrushSize::Five] {
                    ui.radio_value(&mut editor.brush_size, sz, sz.label());
                }
            }

            // --- Настройки структур ---
            if matches!(editor.current_tool, EditorTool::PlaceFactory | EditorTool::PlaceWarbase) {
                ui.separator();
                ui.label(egui::RichText::new("Команда").small().color(egui::Color32::GRAY));
                for (team, label) in [
                    (TeamDef::Player,  "🔵 Игрок"),
                    (TeamDef::Enemy,   "🔴 Враг"),
                    (TeamDef::Neutral, "⚪ Нейтральный"),
                ] {
                    let selected = std::mem::discriminant(&editor.place_team)
                        == std::mem::discriminant(&team);
                    if ui.selectable_label(selected, label).clicked() {
                        editor.place_team = team;
                    }
                }

                if editor.current_tool == EditorTool::PlaceFactory {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Тип фабрики").small().color(egui::Color32::GRAY));
                    for (ft, label) in [
                        (FactoryTypeDef::Chassis,     "⚙ Шасси"),
                        (FactoryTypeDef::Cannon,      "💣 Пушка"),
                        (FactoryTypeDef::Missile,     "🚀 Ракета"),
                        (FactoryTypeDef::Phasers,     "⚡ Фазеры"),
                        (FactoryTypeDef::Electronics, "📡 Электроника"),
                        (FactoryTypeDef::Nuclear,     "☢ Ядерное"),
                    ] {
                        let selected = std::mem::discriminant(&editor.factory_type)
                            == std::mem::discriminant(&ft);
                        if ui.selectable_label(selected, label).clicked() {
                            editor.factory_type = ft;
                        }
                    }
                }
            }

            ui.add_space(8.0);
            ui.separator();

            // --- Undo/Redo ---
            ui.horizontal(|ui| {
                let can_undo = !editor.undo_stack.is_empty();
                let can_redo = !editor.redo_stack.is_empty();
                if ui.add_enabled(can_undo, egui::Button::new("↩ Отмена")).clicked() {
                    editor.undo_requested = true;
                }
                if ui.add_enabled(can_redo, egui::Button::new("↪ Повтор")).clicked() {
                    editor.redo_requested = true;
                }
            });
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Ctrl+Z / Ctrl+Shift+Z")
                    .small()
                    .color(egui::Color32::DARK_GRAY),
            );

            ui.add_space(8.0);
            ui.separator();

            // --- Файловые операции ---
            ui.label(egui::RichText::new("Карта").small().color(egui::Color32::GRAY));
            if ui.button("📄 Новая карта").clicked() {
                editor.show_new_map_dialog = true;
            }
            if ui.button("📂 Открыть").clicked() {
                editor.refresh_map_list();
                editor.show_open_dialog = true;
            }
            let dirty_mark = if editor.dirty { " *" } else { "" };
            if ui.button(format!("💾 Сохранить{dirty_mark}")).clicked() {
                if let Some(err) = editor.validate() {
                    editor.show_validation_error = Some(err);
                } else {
                    match save_map(&editor, &grid) {
                        Ok(path) => {
                            info!("Карта сохранена: {path}");
                            editor.dirty = false;
                        }
                        Err(e) => {
                            editor.show_validation_error = Some(e);
                        }
                    }
                }
            }

            ui.add_space(8.0);
            ui.separator();

            // --- Тестовый запуск ---
            let can_play = editor.validate().is_none();
            if ui
                .add_enabled(can_play, egui::Button::new("▶ Тест карты").fill(egui::Color32::from_rgb(30, 100, 30)))
                .clicked()
            {
                editor.play_test_requested = true;
            }
            if !can_play {
                ui.label(
                    egui::RichText::new("Нужен варбейс игрока и врага")
                        .small()
                        .color(egui::Color32::DARK_RED),
                );
            }

            ui.add_space(8.0);
            ui.separator();
            ui.label(
                egui::RichText::new("ESC — выход в меню")
                    .small()
                    .color(egui::Color32::DARK_GRAY),
            );

            // --- Диалог новой карты ---
            if editor.show_new_map_dialog {
                egui::Window::new("Новая карта")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.label("Размер карты:");
                        for sz in [MapSize::Small, MapSize::Normal, MapSize::Large] {
                            ui.radio_value(&mut editor.new_map_size, sz, sz.label());
                        }
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("✅ Создать").clicked() {
                                let size = editor.new_map_size;
                                *grid = MapGrid::new(size.value(), size.value());
                                editor.reset_to_empty(size);
                                // Пересобрать все terrain-меши (очистить)
                                for y in 0..size.value() {
                                    for x in 0..size.value() {
                                        commands.trigger(RebuildTerrainCell { x, y });
                                    }
                                }
                            }
                            if ui.button("✕ Отмена").clicked() {
                                editor.show_new_map_dialog = false;
                            }
                        });
                    });
            }

            // --- Диалог открытия ---
            if editor.show_open_dialog {
                let maps = editor.available_maps.clone();
                egui::Window::new("Открыть карту")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        if maps.is_empty() {
                            ui.label("Карты не найдены в data/maps/");
                        }
                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            for map_name in &maps {
                                if ui.button(map_name).clicked() {
                                    match load_map_into_editor(map_name, &mut editor, &mut grid) {
                                        Ok(()) => {
                                            editor.show_open_dialog = false;
                                            // Пересобрать terrain
                                            let size = editor.map_size.value();
                                            for y in 0..size {
                                                for x in 0..size {
                                                    commands.trigger(RebuildTerrainCell { x, y });
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            editor.show_validation_error = Some(e);
                                            editor.show_open_dialog = false;
                                        }
                                    }
                                }
                            }
                        });
                        if ui.button("✕ Отмена").clicked() {
                            editor.show_open_dialog = false;
                        }
                    });
            }

            // --- Диалог ошибки ---
            if let Some(err) = editor.show_validation_error.clone() {
                egui::Window::new("Ошибка")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.colored_label(egui::Color32::RED, &err);
                        if ui.button("OK").clicked() {
                            editor.show_validation_error = None;
                        }
                    });
            }
        });

    Ok(())
}

/// Правая панель: свойства карты + статистика.
pub fn draw_editor_map_props(
    mut contexts: EguiContexts,
    mut editor: ResMut<EditorState>,
    grid: Res<MapGrid>,
    mut next_state: ResMut<NextState<AppState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::SidePanel::right("editor_map_props")
        .default_width(200.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Свойства");
            ui.separator();

            ui.label("Имя файла:");
            ui.text_edit_singleline(&mut editor.file_name);
            ui.add_space(4.0);

            ui.label("Название карты:");
            ui.text_edit_singleline(&mut editor.map_name);
            ui.add_space(4.0);

            ui.label("Описание:");
            ui.text_edit_multiline(&mut editor.map_description);
            ui.add_space(8.0);

            ui.separator();
            ui.label(egui::RichText::new("Статистика").small().color(egui::Color32::GRAY));

            let size = editor.map_size.value();
            ui.label(format!("Размер: {}×{}", grid.width, grid.height));
            ui.label(format!("Фабрик: {}", editor.factories.len()));
            ui.label(format!("Варбейсов: {}", editor.warbases.len()));

            let player_wb = editor.warbases.iter().filter(|w| matches!(w.team, TeamDef::Player)).count();
            let enemy_wb  = editor.warbases.iter().filter(|w| matches!(w.team, TeamDef::Enemy)).count();
            let neutral_f = editor.factories.iter().filter(|f| matches!(f.team, TeamDef::Neutral)).count();

            ui.label(format!("  Игрок: {player_wb} ВБ"));
            ui.label(format!("  Враг:  {enemy_wb} ВБ"));
            ui.label(format!("  Нейтр: {neutral_f} фабрик"));

            let spawn = editor.player_spawn;
            ui.label(format!("Спавн: ({},{})", spawn.0, spawn.1));

            if let Some(hov) = editor.hovered_cell {
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!("Курсор: ({},{})", hov.0, hov.1))
                        .color(egui::Color32::YELLOW)
                        .small(),
                );
                if let Some(ct) = grid.get(hov.0, hov.1) {
                    ui.label(
                        egui::RichText::new(format!("{ct:?}"))
                            .color(egui::Color32::LIGHT_GRAY)
                            .small(),
                    );
                }
            }

            ui.add_space(8.0);
            ui.separator();

            // --- Валидация в реальном времени ---
            if let Some(err) = editor.validate() {
                ui.colored_label(egui::Color32::RED, format!("⚠ {err}"));
            } else {
                ui.colored_label(egui::Color32::GREEN, "✓ Карта валидна");
            }

            ui.add_space(8.0);
            ui.separator();

            // --- Тестовый запуск ---
            let can_play = editor.validate().is_none();
            if ui.add_enabled(can_play, egui::Button::new("▶ Тест")).clicked() {
                // Сохраняем временный сценарий для плейтеста
                let _ = std::fs::create_dir_all("saves");
                let scenario_ron = format!(
                    "(\n    name: \"[PLAYTEST]\",\n    description: \"Editor playtest\",\n    map_path: \"data/maps/{}.ron\",\n)\n",
                    editor.file_name
                );
                if std::fs::write("saves/editor_playtest.ron", &scenario_ron).is_ok() {
                    // TODO: 11.16 — AppState::Playing с флагом возврата в Editor
                    next_state.set(AppState::Playing);
                }
            }
            ui.label(egui::RichText::new("(ESC в игре вернёт в меню)").small().color(egui::Color32::DARK_GRAY));

            // Заглушка для размера карты (смена требует пересоздания grid)
            ui.add_space(6.0);
            ui.separator();
            ui.label(egui::RichText::new("Размер карты").small().color(egui::Color32::GRAY));
            ui.label(egui::RichText::new("(Смена — через 'Новая карта')").small().color(egui::Color32::DARK_GRAY));
            let _ = size;
        });

    Ok(())
}

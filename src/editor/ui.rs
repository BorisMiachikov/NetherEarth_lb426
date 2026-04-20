use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    localization::Localization,
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
    loc: Res<Localization>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::SidePanel::left("editor_toolbox")
        .default_width(180.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading(loc.t("editor.title"));
            ui.separator();

            // --- Инструменты ---
            ui.label(egui::RichText::new(loc.t("editor.section.tool")).small().color(egui::Color32::GRAY));
            for (tool, label) in [
                (EditorTool::TerrainBrush,    loc.t("editor.tool.terrain")),
                (EditorTool::PlaceFactory,    loc.t("editor.tool.factory")),
                (EditorTool::PlaceWarbase,    loc.t("editor.tool.warbase")),
                (EditorTool::PlacePlayerSpawn,loc.t("editor.tool.spawn")),
                (EditorTool::Erase,           loc.t("editor.tool.erase")),
            ] {
                ui.radio_value(&mut editor.current_tool, tool, label);
            }
            ui.add_space(6.0);

            // --- Настройки кисти рельефа ---
            if editor.current_tool == EditorTool::TerrainBrush {
                ui.separator();
                ui.label(egui::RichText::new(loc.t("editor.section.cell_type")).small().color(egui::Color32::GRAY));
                for (ct, label) in [
                    (CellTypeDef::Rock,    loc.t("editor.cell.rock")),
                    (CellTypeDef::Pit,     loc.t("editor.cell.pit")),
                    (CellTypeDef::Sand,    loc.t("editor.cell.sand")),
                    (CellTypeDef::Blocked, loc.t("editor.cell.blocked")),
                ] {
                    let selected = std::mem::discriminant(&editor.brush_cell_type)
                        == std::mem::discriminant(&ct);
                    if ui.selectable_label(selected, label).clicked() {
                        editor.brush_cell_type = ct;
                    }
                }
                // Кнопка очистки: установить Open
                if ui.button(loc.t("editor.cell.clear")).clicked() {
                    // Используем специальный маркер — Open не является CellTypeDef,
                    // поэтому передаём сигнал через инструмент Erase
                    editor.current_tool = EditorTool::Erase;
                }
                ui.add_space(4.0);
                ui.label(egui::RichText::new(loc.t("editor.section.brush_size")).small().color(egui::Color32::GRAY));
                for sz in [BrushSize::One, BrushSize::Three, BrushSize::Five] {
                    ui.radio_value(&mut editor.brush_size, sz, sz.label());
                }
            }

            // --- Настройки структур ---
            if matches!(editor.current_tool, EditorTool::PlaceFactory | EditorTool::PlaceWarbase) {
                ui.separator();
                ui.label(egui::RichText::new(loc.t("editor.section.team")).small().color(egui::Color32::GRAY));
                for (team, label) in [
                    (TeamDef::Player,  loc.t("editor.team.player")),
                    (TeamDef::Enemy,   loc.t("editor.team.enemy")),
                    (TeamDef::Neutral, loc.t("editor.team.neutral")),
                ] {
                    let selected = std::mem::discriminant(&editor.place_team)
                        == std::mem::discriminant(&team);
                    if ui.selectable_label(selected, label).clicked() {
                        editor.place_team = team;
                    }
                }

                if editor.current_tool == EditorTool::PlaceFactory {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(loc.t("editor.section.factory_type")).small().color(egui::Color32::GRAY));
                    for (ft, emoji, key) in [
                        (FactoryTypeDef::Chassis,     "⚙",  "ui.resource.chassis"),
                        (FactoryTypeDef::Cannon,      "💣", "ui.resource.cannon"),
                        (FactoryTypeDef::Missile,     "🚀", "ui.resource.missile"),
                        (FactoryTypeDef::Phasers,     "⚡", "ui.resource.phasers"),
                        (FactoryTypeDef::Electronics, "📡", "ui.resource.electronics"),
                        (FactoryTypeDef::Nuclear,     "☢",  "ui.resource.nuclear"),
                    ] {
                        let label = format!("{emoji} {}", loc.t(key));
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
                if ui.add_enabled(can_undo, egui::Button::new(loc.t("editor.btn.undo"))).clicked() {
                    editor.undo_requested = true;
                }
                if ui.add_enabled(can_redo, egui::Button::new(loc.t("editor.btn.redo"))).clicked() {
                    editor.redo_requested = true;
                }
            });
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(loc.t("editor.hint.undo_keys"))
                    .small()
                    .color(egui::Color32::DARK_GRAY),
            );

            ui.add_space(8.0);
            ui.separator();

            // --- Файловые операции ---
            ui.label(egui::RichText::new(loc.t("editor.section.map")).small().color(egui::Color32::GRAY));
            if ui.button(loc.t("editor.btn.new_map")).clicked() {
                editor.show_new_map_dialog = true;
            }
            if ui.button(loc.t("editor.btn.open")).clicked() {
                editor.refresh_map_list();
                editor.show_open_dialog = true;
            }
            let dirty_mark = if editor.dirty { " *" } else { "" };
            if ui.button(format!("{}{dirty_mark}", loc.t("editor.btn.save"))).clicked() {
                if let Some(err) = editor.validate(&loc) {
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
            let can_play = editor.validate(&loc).is_none();
            if ui
                .add_enabled(can_play, egui::Button::new(loc.t("editor.btn.test")).fill(egui::Color32::from_rgb(30, 100, 30)))
                .clicked()
            {
                editor.play_test_requested = true;
            }
            if !can_play {
                ui.label(
                    egui::RichText::new(loc.t("editor.hint.test_invalid"))
                        .small()
                        .color(egui::Color32::DARK_RED),
                );
            }

            ui.add_space(8.0);
            ui.separator();
            ui.label(
                egui::RichText::new(loc.t("editor.hint.esc_menu"))
                    .small()
                    .color(egui::Color32::DARK_GRAY),
            );

            // --- Диалог новой карты ---
            if editor.show_new_map_dialog {
                egui::Window::new(loc.t("editor.dialog.new_map"))
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.label(loc.t("editor.dialog.map_size"));
                        for sz in [MapSize::Small, MapSize::Normal, MapSize::Large] {
                            ui.radio_value(&mut editor.new_map_size, sz, sz.label());
                        }
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button(loc.t("editor.dialog.create")).clicked() {
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
                            if ui.button(loc.t("editor.dialog.cancel")).clicked() {
                                editor.show_new_map_dialog = false;
                            }
                        });
                    });
            }

            // --- Диалог открытия ---
            if editor.show_open_dialog {
                let maps = editor.available_maps.clone();
                egui::Window::new(loc.t("editor.dialog.open_map"))
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        if maps.is_empty() {
                            ui.label(loc.t("editor.dialog.no_maps"));
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
                        if ui.button(loc.t("editor.dialog.cancel")).clicked() {
                            editor.show_open_dialog = false;
                        }
                    });
            }

            // --- Диалог ошибки ---
            if let Some(err) = editor.show_validation_error.clone() {
                egui::Window::new(loc.t("editor.dialog.error"))
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
    loc: Res<Localization>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::SidePanel::right("editor_map_props")
        .default_width(200.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading(loc.t("editor.props.title"));
            ui.separator();

            ui.label(loc.t("editor.props.file_name"));
            ui.text_edit_singleline(&mut editor.file_name);
            ui.add_space(4.0);

            ui.label(loc.t("editor.props.map_name"));
            ui.text_edit_singleline(&mut editor.map_name);
            ui.add_space(4.0);

            ui.label(loc.t("editor.props.description"));
            ui.text_edit_multiline(&mut editor.map_description);
            ui.add_space(8.0);

            ui.separator();
            // --- Начальные ресурсы сценария ---
            let ir_before = editor.initial_resources_enabled;
            ui.checkbox(
                &mut editor.initial_resources_enabled,
                loc.t("editor.props.initial_resources"),
            );
            if editor.initial_resources_enabled != ir_before {
                editor.dirty = true;
            }
            if editor.initial_resources_enabled {
                ui.label(
                    egui::RichText::new(loc.t("editor.props.initial_resources_hint"))
                        .small()
                        .color(egui::Color32::DARK_GRAY),
                );
                let r = &mut editor.initial_resources;
                let mut changed = false;
                for (value, key) in [
                    (&mut r.general,     "ui.resource.general"),
                    (&mut r.chassis,     "ui.resource.chassis"),
                    (&mut r.cannon,      "ui.resource.cannon"),
                    (&mut r.missile,     "ui.resource.missile"),
                    (&mut r.phasers,     "ui.resource.phasers"),
                    (&mut r.electronics, "ui.resource.electronics"),
                    (&mut r.nuclear,     "ui.resource.nuclear"),
                ] {
                    ui.horizontal(|ui| {
                        ui.label(loc.t(key));
                        let resp = ui.add(
                            egui::DragValue::new(value)
                                .range(0..=99999)
                                .speed(1.0),
                        );
                        if resp.changed() { changed = true; }
                    });
                }
                if changed {
                    editor.dirty = true;
                }
            }
            ui.add_space(8.0);

            ui.separator();
            ui.label(egui::RichText::new(loc.t("editor.props.stats")).small().color(egui::Color32::GRAY));

            let size = editor.map_size.value();
            ui.label(format!("{} {}×{}", loc.t("editor.props.size"), grid.width, grid.height));
            ui.label(format!("{} {}", loc.t("editor.props.factories"), editor.factories.len()));
            ui.label(format!("{} {}", loc.t("editor.props.warbases"), editor.warbases.len()));

            let player_wb = editor.warbases.iter().filter(|w| matches!(w.team, TeamDef::Player)).count();
            let enemy_wb  = editor.warbases.iter().filter(|w| matches!(w.team, TeamDef::Enemy)).count();
            let neutral_f = editor.factories.iter().filter(|f| matches!(f.team, TeamDef::Neutral)).count();

            ui.label(format!("{} {player_wb}", loc.t("editor.props.player_wb")));
            ui.label(format!("{} {enemy_wb}", loc.t("editor.props.enemy_wb")));
            ui.label(format!("{} {neutral_f}", loc.t("editor.props.neutral_f")));

            let spawn = editor.player_spawn;
            ui.label(format!("{} ({},{})", loc.t("editor.props.spawn"), spawn.0, spawn.1));

            if let Some(hov) = editor.hovered_cell {
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!("{} ({},{})", loc.t("editor.props.cursor"), hov.0, hov.1))
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
            if let Some(err) = editor.validate(&loc) {
                ui.colored_label(egui::Color32::RED, format!("⚠ {err}"));
            } else {
                ui.colored_label(egui::Color32::GREEN, loc.t("editor.props.valid"));
            }

            ui.add_space(8.0);
            ui.separator();

            // Информация о размере карты (смена — через «Новая карта»)
            ui.add_space(6.0);
            ui.separator();
            ui.label(egui::RichText::new(loc.t("editor.props.map_size_label")).small().color(egui::Color32::GRAY));
            ui.label(egui::RichText::new(loc.t("editor.props.size_hint")).small().color(egui::Color32::DARK_GRAY));
            let _ = size;
        });

    Ok(())
}

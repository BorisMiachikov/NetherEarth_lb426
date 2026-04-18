pub mod camera;
pub mod picking;
pub mod save;
pub mod state;
pub mod terrain;
pub mod tools;
pub mod ui;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    app::state::AppState,
    camera::systems::IsometricCamera,
    map::{
        grid::{CellType, MapGrid},
        loader::{MapSpawnPoints, MapStructures, TeamDef},
    },
    save::TriggerNewGame,
    structure::{factory::Factory, spawn_structures_system, warbase::Warbase},
};

use camera::{free_camera_movement, reset_camera_for_editor, EditorCamera};
use picking::pick_cell;
use state::EditorState;
use terrain::{on_rebuild_terrain_cell, RebuildTerrainCell};
use tools::{apply_tool, update_hover_preview};
use ui::{draw_editor_toolbox, draw_editor_map_props};

/// Маркер: игра запущена из редактора (Play-тест). ESC возвращает в редактор.
#[derive(Resource)]
pub struct EditorPlaytest;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>()
            // Observer для пересборки terrain-мешей
            .add_observer(on_rebuild_terrain_cell)
            // Вход в Editor:
            //   IsometricCamera остаётся активной (egui привязан к ней с самого старта).
            //   Добавляем маркер EditorCamera на неё — включает editor-режим движения/пикинга.
            //   Сбрасываем позицию камеры в центр карты.
            .add_systems(
                OnEnter(AppState::Editor),
                (on_enter_editor, reset_camera_for_editor).chain(),
            )
            // Выход из Editor: убираем маркер, возвращаем видимость
            .add_systems(OnExit(AppState::Editor), (on_exit_editor, cleanup_editor).chain())
            // Логика (Update)
            .add_systems(
                Update,
                (
                    editor_exit_to_menu,
                    free_camera_movement,
                    pick_cell,
                    apply_tool,
                    update_hover_preview,
                    draw_editor_grid,
                    draw_editor_structures,
                )
                    .run_if(in_state(AppState::Editor)),
            )
            // UI (EguiPrimaryContextPass)
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                (draw_editor_toolbox, draw_editor_map_props, draw_exit_dialog)
                    .run_if(in_state(AppState::Editor)),
            )
            // Спавн структур при входе в Playing из редактора
            .add_systems(
                OnEnter(AppState::Playing),
                spawn_structures_system.run_if(resource_exists::<EditorPlaytest>),
            )
            // Play-тест: запуск из редактора
            .add_systems(
                Update,
                (prepare_playtest.run_if(in_state(AppState::Editor)),
                 playtest_esc_handler.run_if(in_state(AppState::Playing)).run_if(resource_exists::<EditorPlaytest>)),
            )
            // Play-тест: оверлей поверх игры
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                playtest_overlay
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<EditorPlaytest>),
            );
    }
}

/// Тонкий grid-оверлей поверх карты в режиме редактора.
fn draw_editor_grid(mut gizmos: Gizmos, grid: Res<MapGrid>) {
    let color = Color::srgba(1.0, 1.0, 1.0, 0.08);
    let w = grid.width as f32;
    let h = grid.height as f32;
    for x in 0..=grid.width {
        let xf = x as f32;
        gizmos.line(Vec3::new(xf, 0.02, 0.0), Vec3::new(xf, 0.02, h), color);
    }
    for y in 0..=grid.height {
        let yf = y as f32;
        gizmos.line(Vec3::new(0.0, 0.02, yf), Vec3::new(w, 0.02, yf), color);
    }
}

/// Gizmos-предпросмотр фабрик, варбейсов и спавна игрока из EditorState.
fn draw_editor_structures(mut gizmos: Gizmos, editor: Res<EditorState>, grid: Res<MapGrid>) {
    // Фабрики: квадрат на полу + вертикальная линия-флажок
    for f in &editor.factories {
        let color = team_gizmo_color(&f.team);
        let base = grid.grid_to_world(f.x, f.y);
        let floor = base + Vec3::Y * 0.03;
        gizmos.rect(Isometry3d::from_translation(floor), Vec2::splat(0.8), color);
        gizmos.line(base + Vec3::Y * 0.03, base + Vec3::Y * 0.9, color);
        gizmos.line(base + Vec3::Y * 0.9, base + Vec3::new(0.25, 0.9, 0.0), color);
        gizmos.line(base + Vec3::new(0.25, 0.9, 0.0), base + Vec3::new(0.25, 0.65, 0.0), color);
    }

    // Варбейсы: двойной квадрат + крест
    for w in &editor.warbases {
        let color = team_gizmo_color(&w.team);
        let base = grid.grid_to_world(w.x, w.y);
        let floor = base + Vec3::Y * 0.03;
        gizmos.rect(Isometry3d::from_translation(floor), Vec2::splat(0.95), color);
        gizmos.rect(Isometry3d::from_translation(floor), Vec2::splat(0.75), color);
        gizmos.line(base + Vec3::new(-0.35, 0.03, 0.0), base + Vec3::new(0.35, 0.03, 0.0), color);
        gizmos.line(base + Vec3::new(0.0, 0.03, -0.35), base + Vec3::new(0.0, 0.03, 0.35), color);
    }

    // Точка спавна игрока: жёлтый ромб + крест
    let (sx, sy) = editor.player_spawn;
    let base = grid.grid_to_world(sx, sy);
    let y = base + Vec3::Y * 0.04;
    let sc = Color::srgb(1.0, 0.9, 0.0);
    let r = 0.4_f32;
    gizmos.line(y + Vec3::new(-r, 0.0, 0.0), y + Vec3::new(0.0, 0.0, -r), sc);
    gizmos.line(y + Vec3::new(0.0, 0.0, -r), y + Vec3::new(r, 0.0, 0.0), sc);
    gizmos.line(y + Vec3::new(r, 0.0, 0.0), y + Vec3::new(0.0, 0.0, r), sc);
    gizmos.line(y + Vec3::new(0.0, 0.0, r), y + Vec3::new(-r, 0.0, 0.0), sc);
    gizmos.line(y + Vec3::new(-0.25, 0.0, 0.0), y + Vec3::new(0.25, 0.0, 0.0), sc);
    gizmos.line(y + Vec3::new(0.0, 0.0, -0.25), y + Vec3::new(0.0, 0.0, 0.25), sc);
}

fn team_gizmo_color(team: &TeamDef) -> Color {
    match team {
        TeamDef::Player  => Color::srgb(0.15, 0.85, 0.25),
        TeamDef::Enemy   => Color::srgb(0.95, 0.15, 0.15),
        TeamDef::Neutral => Color::srgb(0.75, 0.75, 0.75),
    }
}

/// Вход в Editor: добавляем маркер EditorCamera на IsometricCamera + скрываем игровые сущности.
/// IsometricCamera остаётся активной — egui продолжает работать.
fn on_enter_editor(
    mut commands: Commands,
    iso_cam: Query<Entity, With<IsometricCamera>>,
    mut game_vis: Query<&mut Visibility, With<GameWorldEntity>>,
) {
    if let Ok(entity) = iso_cam.single() {
        commands.entity(entity).insert(EditorCamera);
    }
    for mut vis in &mut game_vis {
        *vis = Visibility::Hidden;
    }
}

/// Выход из Editor: снимаем маркер EditorCamera + возвращаем видимость игровым сущностям.
fn on_exit_editor(
    mut commands: Commands,
    iso_cam: Query<Entity, With<IsometricCamera>>,
    mut game_vis: Query<&mut Visibility, With<GameWorldEntity>>,
) {
    if let Ok(entity) = iso_cam.single() {
        commands.entity(entity).remove::<EditorCamera>();
    }
    for mut vis in &mut game_vis {
        *vis = Visibility::Inherited;
    }
}

/// ESC в редакторе → возврат в главное меню.
fn editor_exit_to_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut state: ResMut<EditorState>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    // Если диалог выхода открыт — ESC закрывает его (не выходит)
    if state.show_exit_dialog {
        state.show_exit_dialog = false;
        return;
    }
    if state.dirty {
        state.show_exit_dialog = true;
    } else {
        next_state.set(AppState::MainMenu);
    }
}

/// Убираем сущности редактора (EditorEntity) при выходе из состояния.
fn cleanup_editor(mut commands: Commands, query: Query<Entity, With<EditorEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Маркер: сущность принадлежит редактору и должна быть удалена при выходе.
#[derive(Component)]
pub struct EditorEntity;

/// Маркер: игровая сущность (скаут, структуры, роботы), которую нужно скрывать в редакторе.
#[derive(Component)]
pub struct GameWorldEntity;

// ---------------------------------------------------------------------------
// Диалог подтверждения выхода
// ---------------------------------------------------------------------------

fn draw_exit_dialog(
    mut contexts: bevy_egui::EguiContexts,
    mut editor: ResMut<EditorState>,
    mut next_state: ResMut<NextState<AppState>>,
    grid: Res<MapGrid>,
) -> Result {
    if !editor.show_exit_dialog {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;

    // Затемнение фона
    egui::Area::new(egui::Id::new("exit_dialog_bg"))
        .fixed_pos(egui::Pos2::ZERO)
        .order(egui::Order::Background)
        .interactable(false)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            ui.painter().rect_filled(screen, 0.0, egui::Color32::from_black_alpha(160));
        });

    egui::Window::new("Несохранённые изменения")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label("На карте есть несохранённые изменения.");
            ui.label("Сохранить перед выходом?");
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("💾 Сохранить и выйти").clicked() {
                    if let Some(err) = editor.validate() {
                        editor.show_validation_error = Some(err);
                        editor.show_exit_dialog = false;
                    } else {
                        match crate::editor::save::save_map(&editor, &grid) {
                            Ok(_) => {
                                editor.dirty = false;
                                editor.show_exit_dialog = false;
                                next_state.set(AppState::MainMenu);
                            }
                            Err(e) => {
                                editor.show_validation_error = Some(e);
                                editor.show_exit_dialog = false;
                            }
                        }
                    }
                }
                if ui.button("🚪 Выйти без сохранения").clicked() {
                    editor.dirty = false;
                    editor.show_exit_dialog = false;
                    next_state.set(AppState::MainMenu);
                }
                if ui.button("✕ Отмена").clicked() {
                    editor.show_exit_dialog = false;
                }
            });
        });
    Ok(())
}

// ---------------------------------------------------------------------------
// Play-тест
// ---------------------------------------------------------------------------

/// Готовит мир к тестовому запуску карты из редактора.
#[allow(clippy::too_many_arguments)]
fn prepare_playtest(
    mut editor: ResMut<EditorState>,
    mut map: ResMut<MapGrid>,
    mut map_structures: ResMut<MapStructures>,
    mut spawn_points: ResMut<MapSpawnPoints>,
    structures_q: Query<Entity, Or<(With<Factory>, With<Warbase>)>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !std::mem::take(&mut editor.play_test_requested) {
        return;
    }
    if let Some(err) = editor.validate() {
        editor.show_validation_error = Some(err);
        return;
    }

    // Деспавним все структуры игрового мира
    for entity in &structures_q {
        commands.entity(entity).despawn();
    }

    // Очищаем Structure()-ячейки в MapGrid немедленно
    let (w, h) = (map.width, map.height);
    for y in 0..h {
        for x in 0..w {
            if matches!(map.get(x, y), Some(CellType::Structure(_))) {
                map.set(x, y, CellType::Open);
            }
        }
    }

    // Обновляем ресурсы карты из состояния редактора
    spawn_points.player_spawn = editor.player_spawn;
    map_structures.factories  = editor.factories.clone();
    map_structures.warbases   = editor.warbases.clone();

    // Сбрасываем игровой мир (роботы, ресурсы, скаут, AI, время)
    // Цикл сброса структур в on_trigger_new_game не найдёт ничего в MapGrid → безопасно
    commands.trigger(TriggerNewGame);

    commands.insert_resource(EditorPlaytest);
    next_state.set(AppState::Playing);
}

/// Обрабатывает ESC во время Play-теста: возврат в редактор.
fn playtest_esc_handler(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.remove_resource::<EditorPlaytest>();
        next_state.set(AppState::Editor);
    }
}

/// Небольшой оверлей поверх игры во время Play-теста.
fn playtest_overlay(mut contexts: bevy_egui::EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Area::new(egui::Id::new("playtest_banner"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 8.0))
        .interactable(false)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("▶ ТЕСТ КАРТЫ  |  ESC — вернуться в редактор")
                    .color(egui::Color32::YELLOW)
                    .background_color(egui::Color32::from_black_alpha(200))
                    .size(14.0),
            );
        });
    Ok(())
}

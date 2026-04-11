pub mod camera;
pub mod picking;
pub mod save;
pub mod state;
pub mod terrain;
pub mod tools;
pub mod ui;

use bevy::prelude::*;

use crate::{
    app::state::AppState,
    camera::systems::IsometricCamera,
    map::{
        grid::MapGrid,
        loader::TeamDef,
    },
};

use camera::{free_camera_movement, reset_camera_for_editor, EditorCamera};
use picking::pick_cell;
use state::EditorState;
use terrain::{on_rebuild_terrain_cell, RebuildTerrainCell};
use tools::{apply_tool, update_hover_preview};
use ui::{draw_editor_toolbox, draw_editor_map_props};

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
                (draw_editor_toolbox, draw_editor_map_props)
                    .run_if(in_state(AppState::Editor)),
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
    state: ResMut<EditorState>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    if state.dirty {
        // TODO: диалог подтверждения (задача 11.18)
    }
    next_state.set(AppState::MainMenu);
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

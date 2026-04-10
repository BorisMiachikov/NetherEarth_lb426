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
    map::grid::MapGrid,
};

use camera::{spawn_editor_camera, free_camera_movement};
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
            // Вход в режим редактора: выключаем игровую камеру, прячем игровые сущности
            .add_systems(OnEnter(AppState::Editor), (spawn_editor_camera, on_enter_editor))
            // Выход из режима редактора: включаем игровую камеру обратно
            .add_systems(OnExit(AppState::Editor), (cleanup_editor, on_exit_editor))
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
        gizmos.line(
            Vec3::new(xf, 0.02, 0.0),
            Vec3::new(xf, 0.02, h),
            color,
        );
    }
    for y in 0..=grid.height {
        let yf = y as f32;
        gizmos.line(
            Vec3::new(0.0, 0.02, yf),
            Vec3::new(w,   0.02, yf),
            color,
        );
    }
}

/// При входе в редактор: отключаем игровую IsometricCamera и прячем игровые сущности.
fn on_enter_editor(
    mut game_cameras: Query<&mut Camera, With<IsometricCamera>>,
    mut game_visibility: Query<&mut Visibility, With<GameWorldEntity>>,
) {
    for mut cam in &mut game_cameras {
        cam.is_active = false;
    }
    for mut vis in &mut game_visibility {
        *vis = Visibility::Hidden;
    }
}

/// При выходе из редактора: включаем игровую IsometricCamera и показываем игровые сущности.
fn on_exit_editor(
    mut game_cameras: Query<&mut Camera, With<IsometricCamera>>,
    mut game_visibility: Query<&mut Visibility, With<GameWorldEntity>>,
) {
    for mut cam in &mut game_cameras {
        cam.is_active = true;
    }
    for mut vis in &mut game_visibility {
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
    // TODO: если dirty — показать диалог (задача 11.18)
    if state.dirty {
        // пока просто выходим; диалог будет в 11.18
    }
    next_state.set(AppState::MainMenu);
}

/// Убираем сущности редактора при выходе из состояния.
fn cleanup_editor(mut commands: Commands, query: Query<Entity, With<EditorEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Маркер: сущность принадлежит редактору и должна быть удалена при выходе.
#[derive(Component)]
pub struct EditorEntity;

/// Маркер: игровая сущность (скаут, структуры, роботы), которую нужно скрывать в редакторе.
/// Навешивается на сущности при их спавне через PlayerPlugin / StructurePlugin / RobotPlugin.
#[derive(Component)]
pub struct GameWorldEntity;

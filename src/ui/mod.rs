pub mod builder_ui;
pub mod gameover;
pub mod help_overlay;
pub mod hud;
pub mod menu;
pub mod minimap;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use builder_ui::{draw_builder_ui, on_warbase_click, open_builder_input, BuilderUiState};
use gameover::draw_gameover_screen;
use help_overlay::{draw_help_overlay, toggle_help_overlay, HelpOverlayState};
use hud::draw_resource_hud;
use menu::{draw_main_menu, draw_pause_menu, init_to_main_menu, toggle_pause, ScenarioList};
use minimap::draw_minimap;

use crate::app::state::AppState;

/// Состояния, при которых отображается игровой интерфейс (HUD, builder).
fn is_in_game(state: Res<State<AppState>>) -> bool {
    matches!(
        state.get(),
        AppState::Playing | AppState::Paused | AppState::GameOver
    )
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScenarioList::load_from_dir())
            .init_resource::<BuilderUiState>()
            .init_resource::<HelpOverlayState>()
            .add_observer(on_warbase_click)
            // Начальный переход: Loading → MainMenu
            .add_systems(OnEnter(AppState::Loading), init_to_main_menu)
            // Системы без egui — в Update
            .add_systems(Update, (open_builder_input, toggle_pause, toggle_help_overlay))
            // Оверлеи и меню — всегда активны
            .add_systems(
                EguiPrimaryContextPass,
                (
                    draw_main_menu,
                    draw_pause_menu,
                    draw_gameover_screen,
                    draw_minimap,
                    draw_help_overlay,
                ),
            )
            // Игровой HUD — только во время игры/паузы/game over
            .add_systems(
                EguiPrimaryContextPass,
                (draw_resource_hud, draw_builder_ui).run_if(is_in_game),
            );
    }
}

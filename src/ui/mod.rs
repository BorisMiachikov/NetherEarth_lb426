pub mod builder_ui;
pub mod gameover;
pub mod hud;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use builder_ui::{draw_builder_ui, open_builder_input, BuilderUiState};
use gameover::draw_gameover_screen;
use hud::draw_resource_hud;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuilderUiState>()
            // Системы без egui — в Update
            .add_systems(Update, open_builder_input)
            // Системы с egui — в EguiPrimaryContextPass
            .add_systems(
                EguiPrimaryContextPass,
                (draw_resource_hud, draw_builder_ui, draw_gameover_screen),
            );
    }
}

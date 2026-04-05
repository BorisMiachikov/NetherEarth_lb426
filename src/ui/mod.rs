pub mod builder_ui;
pub mod hud;

use bevy::prelude::*;

use builder_ui::{draw_builder_ui, open_builder_input, BuilderUiState};
use hud::draw_resource_hud;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuilderUiState>()
            .add_systems(Update, (draw_resource_hud, draw_builder_ui, open_builder_input));
    }
}

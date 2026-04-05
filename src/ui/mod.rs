pub mod hud;

use bevy::prelude::*;

use hud::draw_resource_hud;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_resource_hud);
    }
}

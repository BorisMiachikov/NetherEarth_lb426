pub mod resource;

use bevy::prelude::*;

use resource::PlayerResources;

pub use resource::{PlayerResources as Resources, ResourceType};

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerResources::with_starting_values());
    }
}

pub mod production;
pub mod resource;

use bevy::prelude::*;

use production::{tick_production, LastProductionDay};
use resource::PlayerResources;

pub use resource::{PlayerResources as Resources, ResourceType};

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerResources::with_starting_values())
            .init_resource::<LastProductionDay>()
            .add_systems(FixedUpdate, tick_production);
    }
}

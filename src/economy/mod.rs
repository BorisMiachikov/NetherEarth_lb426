pub mod production;
pub mod resource;

use bevy::prelude::*;

use production::{tick_production, LastProductionDay};

pub use resource::{EnemyResources, PlayerResources};

use crate::app::state::AppState;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerResources::with_starting_values())
            .insert_resource(EnemyResources(PlayerResources::with_starting_values()))
            .init_resource::<LastProductionDay>()
            .add_systems(FixedUpdate, tick_production.run_if(in_state(AppState::Playing)));
    }
}

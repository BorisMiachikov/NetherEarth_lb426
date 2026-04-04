pub mod components;
pub mod input;
pub mod systems;

use bevy::prelude::*;

use input::read_scout_input;
use systems::{move_scout, spawn_scout};

pub use components::{PlayerScout, ScoutMoveIntent, ScoutMovement};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scout)
            .add_systems(Update, read_scout_input)
            .add_systems(FixedUpdate, move_scout);
    }
}

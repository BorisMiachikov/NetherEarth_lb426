pub mod scoring;
pub mod state;
pub mod systems;

use bevy::prelude::*;

use state::{load_ai_config, AICommander, GameResult};
use systems::{
    ai_assign_commands, ai_build_robots, arm_nuclear_on_arrival, check_victory_defeat,
    seek_destroy_base, update_seek_destroy,
};

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        let config = load_ai_config();
        app.insert_resource(AICommander::new(config))
            .init_resource::<GameResult>()
            .add_systems(
                FixedUpdate,
                (
                    check_victory_defeat,
                    ai_build_robots,
                    ai_assign_commands,
                    update_seek_destroy,
                    seek_destroy_base,
                    arm_nuclear_on_arrival,
                ),
            );
    }
}

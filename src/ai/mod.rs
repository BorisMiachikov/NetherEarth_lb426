pub mod build;
pub mod command;
pub mod scoring;
pub mod state;
pub mod victory;

use bevy::prelude::*;

use build::ai_build_robots;
use command::{ai_assign_commands, arm_nuclear_on_arrival, seek_destroy_base, update_retreat, update_seek_destroy};
use state::{load_ai_config, AICommander, GameResult};
use victory::check_victory_defeat;

use crate::app::state::AppState;

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
                    update_retreat,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

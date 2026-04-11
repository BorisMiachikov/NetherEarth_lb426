pub mod command;
pub mod queue;
pub mod systems;

use bevy::prelude::*;

use systems::{process_commands, update_patrol};

pub use command::RobotCommand;
pub use queue::CommandQueue;

use crate::app::state::AppState;

pub struct CommandPlugin;

impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (process_commands, update_patrol.after(process_commands))
                .run_if(in_state(AppState::Playing)),
        );
    }
}

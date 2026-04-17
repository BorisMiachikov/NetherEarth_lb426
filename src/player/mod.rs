pub mod commands_ui;
pub mod components;
pub mod input;
pub mod selection;
pub mod systems;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::camera::systems::rotate_camera;
use commands_ui::{draw_command_indicators, right_click_move, robot_info_panel, CommandUiState};
use input::read_scout_input;
use selection::{
    draw_selection_indicators, handle_selection_groups, on_robot_click, SelectionState,
};
use systems::{move_manual_control_robot, move_scout, spawn_scout, sync_manual_control_camera};

use crate::app::state::AppState;

pub use components::{ManualControl, PlayerScout, ScoutMoveIntent, ScoutMovement};
pub use selection::Selected;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionState>()
            .init_resource::<CommandUiState>()
            .add_observer(on_robot_click)
            .add_systems(Startup, spawn_scout)
            .add_systems(
                Update,
                (
                    read_scout_input,
                    right_click_move,
                    handle_selection_groups,
                    draw_selection_indicators,
                    draw_command_indicators,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    move_scout.after(rotate_camera),
                    move_manual_control_robot.after(rotate_camera),
                    sync_manual_control_camera,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                robot_info_panel.run_if(in_state(AppState::Playing)),
            );
    }
}

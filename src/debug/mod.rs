pub mod gizmos;
pub mod overlay;
pub mod robot_spawn;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_egui::EguiPrimaryContextPass;

use gizmos::{draw_debug_gizmos, toggle_gizmos, DebugGizmosState};
use overlay::debug_overlay;
use robot_spawn::{robot_spawn_panel, RobotSpawnUi};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<DebugGizmosState>()
            .init_resource::<RobotSpawnUi>()
            .add_systems(
                EguiPrimaryContextPass,
                (debug_overlay, robot_spawn_panel),
            )
            .add_systems(Update, (toggle_gizmos, draw_debug_gizmos));
    }
}

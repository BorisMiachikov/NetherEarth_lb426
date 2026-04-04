pub mod gizmos;
pub mod overlay;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_egui::EguiPrimaryContextPass;

use gizmos::{draw_debug_gizmos, toggle_gizmos, DebugGizmosState};
use overlay::debug_overlay;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<DebugGizmosState>()
            .add_systems(EguiPrimaryContextPass, debug_overlay)
            .add_systems(Update, (toggle_gizmos, draw_debug_gizmos));
    }
}

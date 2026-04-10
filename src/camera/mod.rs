pub mod systems;

use bevy::prelude::*;

use systems::{follow_target, rotate_camera, spawn_camera, zoom_camera};

pub use systems::{CameraTarget, IsometricCamera};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (zoom_camera, rotate_camera))
            .add_systems(PostUpdate, follow_target);
    }
}

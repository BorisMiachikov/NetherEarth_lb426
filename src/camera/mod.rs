pub mod systems;

use bevy::prelude::*;

use systems::{follow_target, rotate_camera, spawn_camera, zoom_camera};

pub use systems::{CameraTarget, IsometricCamera};

use crate::app::state::AppState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            // Зум и вращение — только вне редактора (free_camera_movement обрабатывает их сам)
            .add_systems(
                Update,
                (zoom_camera, rotate_camera).run_if(not(in_state(AppState::Editor))),
            )
            // Следование за скаутом — только в игровых состояниях
            .add_systems(
                PostUpdate,
                follow_target.run_if(
                    in_state(AppState::Playing).or(in_state(AppState::Paused)),
                ),
            );
    }
}

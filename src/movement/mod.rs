pub mod pathfinding;
pub mod steering;
pub mod velocity;

use bevy::prelude::*;

use steering::{compute_path, follow_path};

pub use steering::CurrentPath;
pub use velocity::{MovementTarget, Velocity};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (compute_path, follow_path.after(compute_path)));
    }
}

pub mod pathfinding;
pub mod steering;
pub mod velocity;

use bevy::prelude::*;

use steering::{compute_path, detect_stuck_robots, follow_path, separate_robots};

pub use steering::{CurrentPath, StuckDetector};
pub use velocity::{MovementTarget, Velocity};

/// Выбирает точку исследования в квадранте, противоположном текущей позиции.
/// Детерминировано по entity id + текущей позиции.
pub fn exploration_target(entity: Entity, pos: Vec3, map_w: u32, map_h: u32) -> Vec3 {
    use crate::map::grid::CELL_SIZE;

    let half_w = (map_w / 2).max(1);
    let half_h = (map_h / 2).max(1);

    let x_base = if pos.x < half_w as f32 { half_w } else { 0 };
    let z_base = if pos.z < half_h as f32 { half_h } else { 0 };

    let seed = (entity.to_bits() as u32)
        .wrapping_mul(2654435761)
        .wrapping_add(pos.x as u32)
        .wrapping_add(pos.z as u32);
    let dx = seed % half_w;
    let dz = seed.wrapping_mul(2246822519) % half_h;

    Vec3::new(
        (x_base + dx) as f32 * CELL_SIZE,
        0.3,
        (z_base + dz) as f32 * CELL_SIZE,
    )
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                detect_stuck_robots,
                compute_path.after(detect_stuck_robots),
                follow_path.after(compute_path),
                separate_robots.after(follow_path),
            ),
        );
    }
}

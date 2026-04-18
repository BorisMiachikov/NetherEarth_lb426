pub mod death;
pub mod fire;
pub mod projectile;
pub mod targeting;
pub mod visuals;
pub mod weapon;

use bevy::prelude::*;

use death::on_entity_destroyed;
use fire::fire_weapons;
use projectile::move_projectiles;
use targeting::acquire_targets;
use visuals::{draw_muzzle_flashes, draw_projectiles};

pub use weapon::WeaponCooldowns;

use crate::{app::state::AppState, spatial::SpatialSet};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_entity_destroyed)
            .add_systems(
                FixedUpdate,
                (
                    acquire_targets.after(SpatialSet),
                    fire_weapons.after(acquire_targets),
                    move_projectiles.after(fire_weapons),
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (draw_muzzle_flashes, draw_projectiles)
                    .run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
            );
    }
}

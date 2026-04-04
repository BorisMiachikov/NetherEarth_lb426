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

pub use projectile::Projectile;
pub use weapon::{CombatTarget, MuzzleFlash, WeaponCooldowns};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_entity_destroyed)
            .add_systems(
                FixedUpdate,
                (
                    acquire_targets,
                    fire_weapons.after(acquire_targets),
                    move_projectiles.after(fire_weapons),
                ),
            )
            .add_systems(Update, (draw_muzzle_flashes, draw_projectiles));
    }
}

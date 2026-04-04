use bevy::prelude::*;

use super::{projectile::Projectile, weapon::MuzzleFlash};

/// Рисует линии выстрелов (хитскан) и уменьшает таймер вспышки.
pub fn draw_muzzle_flashes(
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut flashes: Query<(Entity, &Transform, &mut MuzzleFlash)>,
    time: Res<Time>,
) {
    for (entity, tf, mut flash) in &mut flashes {
        gizmos.line(
            tf.translation + Vec3::Y * 0.3,
            flash.target_pos,
            Color::srgb(1.0, 1.0, 0.0),
        );
        flash.timer -= time.delta_secs();
        if flash.timer <= 0.0 {
            commands.entity(entity).remove::<MuzzleFlash>();
        }
    }
}

/// Рисует оранжевые сферы на месте летящих ракет.
pub fn draw_projectiles(
    mut gizmos: Gizmos,
    projectiles: Query<&Transform, With<Projectile>>,
) {
    for tf in &projectiles {
        gizmos.sphere(tf.translation, 0.2, Color::srgb(1.0, 0.4, 0.0));
    }
}

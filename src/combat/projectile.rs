use bevy::prelude::*;

use crate::core::events::EntityDamaged;

/// Ракета, летящая к цели с самонаведением.
#[derive(Component, Debug)]
pub struct Projectile {
    pub target: Entity,
    pub damage: f32,
    pub speed: f32,
}

/// Двигает снаряды к целям. При достижении наносит урон и уничтожает снаряд.
pub fn move_projectiles(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &Projectile)>,
    targets: Query<&Transform, Without<Projectile>>,
    time: Res<Time>,
) {
    for (proj_entity, mut proj_tf, proj) in &mut projectiles {
        let Ok(target_tf) = targets.get(proj.target) else {
            // Цель уничтожена раньше попадания — убираем снаряд
            commands.entity(proj_entity).despawn();
            continue;
        };

        let step = proj.speed * time.delta_secs();
        let dist = proj_tf.translation.distance(target_tf.translation);

        if dist <= step {
            // Попадание
            commands.trigger(EntityDamaged {
                entity: proj.target,
                amount: proj.damage,
                attacker: None,
            });
            commands.entity(proj_entity).despawn();
        } else {
            let dir = (target_tf.translation - proj_tf.translation).normalize();
            proj_tf.translation += dir * step;
        }
    }
}

use bevy::prelude::*;

use crate::{
    core::events::{EntityDamaged, EntityDestroyed},
    robot::components::{Nuclear, RobotMarker},
};

/// Observer: обрабатывает EntityDestroyed — ядерный взрыв (если armed) и деспавн.
pub fn on_entity_destroyed(
    trigger: On<EntityDestroyed>,
    mut commands: Commands,
    robots: Query<(Entity, &Transform, Option<&Nuclear>), With<RobotMarker>>,
) {
    let entity = trigger.event().entity;

    // Ядерный взрыв при условии, что заряд был взведён
    if let Ok((_, tf, Some(nuc))) = robots.get(entity) {
        if nuc.armed {
            let blast_pos = tf.translation;
            let blast_radius = nuc.blast_radius;

            let victims: Vec<Entity> = robots
                .iter()
                .filter(|(e, t, _)| *e != entity && t.translation.distance(blast_pos) <= blast_radius)
                .map(|(e, _, _)| e)
                .collect();

            for victim in victims {
                commands.trigger(EntityDamaged {
                    entity: victim,
                    amount: 9999.0,
                    attacker: Some(entity),
                });
            }
        }
    }

    // Деспавн уничтоженной сущности
    commands.entity(entity).despawn();
}

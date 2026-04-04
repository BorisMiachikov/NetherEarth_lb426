use bevy::prelude::*;

use crate::{
    core::Team,
    robot::components::{RobotMarker, WeaponSlots},
};

use super::weapon::CombatTarget;

/// Назначает / снимает CombatTarget: ближайший враг в радиусе хотя бы одного оружия.
pub fn acquire_targets(
    mut commands: Commands,
    attackers: Query<(Entity, &Transform, &WeaponSlots, &Team), With<RobotMarker>>,
    all_robots: Query<(Entity, &Transform, &Team), With<RobotMarker>>,
) {
    for (entity, tf, slots, team) in &attackers {
        let max_range = slots
            .slots
            .iter()
            .flatten()
            .map(|w| w.range)
            .fold(0.0_f32, f32::max);

        if max_range <= 0.0 {
            continue;
        }

        let nearest_enemy = all_robots
            .iter()
            .filter(|(e, _, t)| *e != entity && *t != team)
            .min_by_key(|(_, t, _)| (tf.translation.distance(t.translation) * 1000.0) as u32);

        match nearest_enemy {
            Some((target_e, target_tf, _))
                if tf.translation.distance(target_tf.translation) <= max_range =>
            {
                commands.entity(entity).insert(CombatTarget(target_e));
            }
            _ => {
                commands.entity(entity).remove::<CombatTarget>();
            }
        }
    }
}

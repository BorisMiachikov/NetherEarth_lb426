use bevy::prelude::*;

use crate::{
    core::Team,
    robot::components::{RobotMarker, WeaponSlots},
    spatial::SpatialIndex,
};

use super::weapon::CombatTarget;

/// Назначает / снимает CombatTarget через SpatialIndex.
/// Текущая цель сохраняется, пока она жива и в радиусе — A* не перезапускается.
pub fn acquire_targets(
    mut commands: Commands,
    index: Res<SpatialIndex>,
    attackers: Query<(Entity, &Transform, &WeaponSlots, &Team, Option<&CombatTarget>), With<RobotMarker>>,
    alive: Query<&Transform, With<RobotMarker>>,
) {
    for (entity, tf, slots, team, cur_target) in &attackers {
        let max_range = slots
            .slots
            .iter()
            .flatten()
            .map(|w| w.range)
            .fold(0.0_f32, f32::max);

        if max_range <= 0.0 {
            continue;
        }

        // Текущая цель жива и в радиусе — не пересчитываем (12.16).
        if let Some(&CombatTarget(target_e)) = cur_target {
            if let Ok(target_tf) = alive.get(target_e) {
                if tf.translation.distance(target_tf.translation) <= max_range {
                    continue;
                }
            }
        }

        // Ищем ближайшего врага через SpatialIndex.
        let pos = tf.translation;
        let mut nearest: Option<(Entity, f32)> = None;
        index.query_radius(pos, max_range, |e, enemy_pos, t| {
            if e == entity || t == *team {
                return;
            }
            let dist = pos.distance(enemy_pos);
            if nearest.map_or(true, |(_, d)| dist < d) {
                nearest = Some((e, dist));
            }
        });

        match nearest {
            Some((target_e, _)) => {
                if cur_target.map_or(true, |ct| ct.0 != target_e) {
                    debug!("combat: {:?} → цель {:?}", entity, target_e);
                }
                commands.entity(entity).insert(CombatTarget(target_e));
            }
            None => {
                if cur_target.is_some() {
                    debug!("combat: {:?} потерял цель", entity);
                }
                commands.entity(entity).remove::<CombatTarget>();
            }
        }
    }
}

use bevy::prelude::*;

use super::components::{Chassis, Electronics, RobotMarker, RobotStats, WeaponSlots};

/// Пересчитывает RobotStats при изменении компонентов модулей.
pub fn recalc_stats(
    mut query: Query<
        (&Chassis, &WeaponSlots, Option<&Electronics>, &mut RobotStats),
        (
            With<RobotMarker>,
            Or<(Changed<Chassis>, Changed<WeaponSlots>, Changed<Electronics>)>,
        ),
    >,
) {
    for (chassis, slots, elec, mut stats) in &mut query {
        // max_hp = base_hp + sum(module_weight) * 2
        stats.max_hp = chassis.base_hp + slots.total_weight() * 2.0;
        stats.speed = chassis.speed;

        // Время захвата: базовое минус бонус электроники
        stats.capture_time = if let Some(e) = elec {
            crate::structure::capture::BASE_CAPTURE_TIME * (1.0 - e.capture_time_reduction)
        } else {
            crate::structure::capture::BASE_CAPTURE_TIME
        };
    }
}

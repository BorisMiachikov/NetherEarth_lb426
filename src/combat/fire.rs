use bevy::prelude::*;

use crate::{
    core::{events::EntityDamaged, Team},
    robot::components::{RobotMarker, WeaponSlots, WeaponType},
};

use super::{
    projectile::Projectile,
    weapon::{CombatTarget, MuzzleFlash, WeaponCooldowns},
};

/// Тикает перезарядку и производит выстрел по CombatTarget.
pub fn fire_weapons(
    mut commands: Commands,
    mut query: Query<
        (Entity, &Transform, &WeaponSlots, &mut WeaponCooldowns, &CombatTarget, &Team),
        With<RobotMarker>,
    >,
    targets: Query<&Transform, With<RobotMarker>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, tf, slots, mut cooldowns, combat_target, team) in &mut query {
        let Ok(target_tf) = targets.get(combat_target.0) else {
            continue;
        };
        let dist = tf.translation.distance(target_tf.translation);

        for (i, slot) in slots.slots.iter().enumerate() {
            let Some(weapon) = slot else { continue };

            // Уменьшаем таймер перезарядки
            cooldowns.cooldowns[i] = (cooldowns.cooldowns[i] - dt).max(0.0);

            if cooldowns.cooldowns[i] > 0.0 || dist > weapon.range {
                continue;
            }

            match weapon.weapon_type {
                WeaponType::Cannon | WeaponType::Phasers => {
                    // Хитскан — мгновенный урон
                    commands.trigger(EntityDamaged {
                        entity: combat_target.0,
                        amount: weapon.damage,
                        attacker: Some(entity),
                    });
                    // Визуальная вспышка выстрела
                    commands.entity(entity).insert(MuzzleFlash {
                        target_pos: target_tf.translation,
                        timer: 0.1,
                    });
                }
                WeaponType::Missile => {
                    // Спавним ракету
                    commands.spawn((
                        Name::new("Missile"),
                        Projectile {
                            target: combat_target.0,
                            damage: weapon.damage,
                            speed: 8.0,
                            owner_team: *team,
                        },
                        Transform::from_translation(tf.translation + Vec3::Y * 0.5),
                    ));
                }
            }

            cooldowns.cooldowns[i] = weapon.reload_time;
        }
    }
}

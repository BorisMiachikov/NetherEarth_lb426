use bevy::prelude::*;

use crate::{
    core::{events::EntityDamaged, Team},
    robot::components::{Electronics, RobotMarker, WeaponSlots, WeaponType},
};

use super::{
    projectile::Projectile,
    weapon::{CombatTarget, MuzzleFlash, WeaponCooldowns},
};

/// Тикает перезарядку и производит выстрел по CombatTarget.
pub fn fire_weapons(
    mut commands: Commands,
    mut query: Query<
        (Entity, &Transform, &WeaponSlots, &mut WeaponCooldowns, &CombatTarget, &Team, Option<&Electronics>),
        With<RobotMarker>,
    >,
    targets: Query<&Transform, With<RobotMarker>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, tf, slots, mut cooldowns, combat_target, _team, electronics) in &mut query {
        let reload_mult = match electronics {
            Some(e) => 1.0 - e.fire_rate_bonus,
            None => 1.0,
        };
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
                    // Визуальная вспышка выстрела (try_insert: робот мог умереть в той же цепочке)
                    commands.entity(entity).try_insert(MuzzleFlash {
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
                        },
                        Transform::from_translation(tf.translation + Vec3::Y * 0.5),
                    ));
                }
            }

            cooldowns.cooldowns[i] = weapon.reload_time * reload_mult;
        }
    }
}

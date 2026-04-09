use bevy::prelude::*;

use crate::{
    core::events::{EntityDamaged, EntityDestroyed},
    map::grid::{CellType, MapGrid},
    robot::components::{Nuclear, RobotMarker},
    structure::{factory::Factory, warbase::Warbase},
};

/// Observer: обрабатывает EntityDestroyed — ядерный взрыв (если armed), очистка карты и деспавн.
pub fn on_entity_destroyed(
    trigger: On<EntityDestroyed>,
    mut commands: Commands,
    mut map: ResMut<MapGrid>,
    robots: Query<(Entity, &Transform, Option<&Nuclear>), With<RobotMarker>>,
    structures: Query<(Entity, &Transform), Or<(With<Factory>, With<Warbase>)>>,
) {
    let entity = trigger.event().entity;

    // Ядерный взрыв при условии, что заряд был взведён
    if let Ok((_, tf, Some(nuc))) = robots.get(entity) {
        if nuc.armed {
            let blast_pos = tf.translation;
            let blast_radius = nuc.blast_radius;

            // Роботы в радиусе
            let robot_victims: Vec<Entity> = robots
                .iter()
                .filter(|(e, t, _)| *e != entity && t.translation.distance(blast_pos) <= blast_radius)
                .map(|(e, _, _)| e)
                .collect();

            // Структуры в радиусе (варбейсы, фабрики)
            let structure_victims: Vec<Entity> = structures
                .iter()
                .filter(|(e, t)| *e != entity && t.translation.distance(blast_pos) <= blast_radius)
                .map(|(e, _)| e)
                .collect();

            for victim in robot_victims.into_iter().chain(structure_victims) {
                commands.trigger(EntityDamaged {
                    entity: victim,
                    amount: 9999.0,
                    attacker: Some(entity),
                });
            }
        }
    }

    // Если уничтожена структура — освобождаем её ячейку в MapGrid.
    if structures.contains(entity) {
        if let Ok((_, tf)) = structures.get(entity) {
            if let Some((gx, gy)) = map.world_to_grid(tf.translation) {
                // Освобождаем только если ячейка принадлежит именно этой entity.
                if matches!(map.get(gx, gy), Some(CellType::Structure(e)) if e == entity) {
                    map.set(gx, gy, CellType::Open);
                }
            }
        }
    }

    // Деспавн уничтоженной сущности
    commands.entity(entity).despawn();
}

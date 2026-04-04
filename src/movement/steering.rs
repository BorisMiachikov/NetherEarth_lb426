use bevy::prelude::*;

use crate::map::grid::{MapGrid, CELL_SIZE};
use crate::robot::components::{Chassis, ChassisType, RobotMarker, RobotStats};

use super::{
    pathfinding::{find_path, GridCell},
    velocity::MovementTarget,
};

/// Текущий путь робота.
#[derive(Component, Default)]
pub struct CurrentPath {
    pub waypoints: Vec<GridCell>,
    pub index: usize,
}

/// Обновляет CurrentPath при смене MovementTarget (вычисляет A*).
pub fn compute_path(
    map: Res<MapGrid>,
    mut query: Query<
        (&Transform, &Chassis, &mut CurrentPath),
        (With<RobotMarker>, Changed<MovementTarget>),
    >,
    targets: Query<&MovementTarget>,
    entities: Query<Entity, With<RobotMarker>>,
) {
    for entity in &entities {
        let Ok((tf, chassis, mut path)) = query.get_mut(entity) else {
            continue;
        };
        let Ok(target) = targets.get(entity) else {
            continue;
        };

        let Some(start) = map.world_to_grid(tf.translation) else {
            continue;
        };
        let Some(goal) = map.world_to_grid(target.0) else {
            continue;
        };

        let can_fly = chassis.chassis_type.can_fly();
        let start_cell = GridCell::new(start.0, start.1);
        let goal_cell = GridCell::new(goal.0, goal.1);

        if let Some(new_path) = find_path(&map, start_cell, goal_cell, can_fly) {
            path.waypoints = new_path;
            path.index = 0;
        } else {
            path.waypoints.clear();
            path.index = 0;
        }
    }
}

/// Движение по пути (FixedUpdate).
pub fn follow_path(
    time: Res<Time>,
    map: Res<MapGrid>,
    mut query: Query<
        (&mut Transform, &RobotStats, &mut CurrentPath),
        With<RobotMarker>,
    >,
) {
    let dt = time.delta_secs();

    for (mut tf, stats, mut path) in &mut query {
        if path.index >= path.waypoints.len() {
            continue;
        }

        let target_cell = path.waypoints[path.index];
        let target_world = map.grid_to_world(target_cell.x, target_cell.y);
        let target_xz = Vec3::new(target_world.x, tf.translation.y, target_world.z);

        let dir = target_xz - tf.translation;
        let dist = dir.length();
        let step = stats.speed * CELL_SIZE * dt;

        if dist <= step {
            tf.translation = target_xz;
            path.index += 1;
        } else {
            let dir_norm = dir / dist;
            tf.translation += dir_norm * step;

            // Поворот в направлении движения
            if dir_norm.length_squared() > 0.001 {
                let target_rot = Transform::from_translation(Vec3::ZERO)
                    .looking_at(-dir_norm, Vec3::Y)
                    .rotation;
                tf.rotation = tf.rotation.slerp(target_rot, (dt * 8.0).min(1.0));
            }
        }
    }
}

/// Простое расталкивание роботов при наложении.
pub fn separate_robots(
    mut query: Query<&mut Transform, With<RobotMarker>>,
) {
    const MIN_DIST: f32 = 1.0;
    const PUSH: f32 = 0.05;

    let positions: Vec<(Entity, Vec3)> = query
        .iter()
        .map(|tf| (Entity::PLACEHOLDER, tf.translation))
        .collect();

    // Простой O(n²) для небольшого числа роботов
    let entities: Vec<Entity> = query.iter().map(|_| Entity::PLACEHOLDER).collect();
    drop(positions);
    drop(entities);
    // TODO: реализовать когда будет выбор роботов (Фаза 4)
}

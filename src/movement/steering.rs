use bevy::prelude::*;

use crate::map::grid::{CellType, MapGrid, CELL_SIZE};
use crate::robot::components::{Chassis, ChassisType, RobotMarker, RobotStats};

use super::{
    pathfinding::{find_path, GridCell},
    velocity::MovementTarget,
};

/// Детектор застревания: отслеживает, что робот двигается.
/// Если за `threshold` секунд позиция не изменилась значимо — сбрасывает путь.
#[derive(Component)]
pub struct StuckDetector {
    pub last_pos: Vec3,
    pub timer: f32,
    /// Порог (секунды) без движения → пересчёт пути.
    pub threshold: f32,
}

impl Default for StuckDetector {
    fn default() -> Self {
        Self {
            last_pos: Vec3::ZERO,
            timer: 0.0,
            threshold: 3.0,
        }
    }
}

/// Текущий путь робота.
#[derive(Component, Default)]
pub struct CurrentPath {
    pub waypoints: Vec<GridCell>,
    pub index: usize,
    /// Последняя цель, для которой был вычислен путь.
    /// Путь пересчитывается только когда цель меняется.
    pub last_target: Option<Vec3>,
}

/// Обновляет CurrentPath при смене MovementTarget (вычисляет A*).
pub fn compute_path(
    map: Res<MapGrid>,
    mut query: Query<
        (&Transform, &Chassis, &mut CurrentPath, &MovementTarget),
        With<RobotMarker>,
    >,
) {
    for (tf, chassis, mut path, target) in &mut query {
        if path.last_target == Some(target.0) {
            continue;
        }
        path.last_target = Some(target.0);

        let Some(start) = map.world_to_grid(tf.translation) else {
            continue;
        };
        let Some(goal) = map.world_to_grid(target.0) else {
            continue;
        };

        let start_cell = GridCell::new(start.0, start.1);
        let goal_cell = GridCell::new(goal.0, goal.1);

        if let Some(new_path) = find_path(&map, start_cell, goal_cell, chassis.chassis_type) {
            path.waypoints = new_path;
            path.index = 0;
        } else {
            path.waypoints.clear();
            path.index = 0;
        }
    }
}

/// Коэффициент замедления на песке для Wheels и Bipod.
const SAND_SPEED_MULT: f32 = 0.5;

/// Движение по пути (FixedUpdate).
pub fn follow_path(
    time: Res<Time>,
    map: Res<MapGrid>,
    mut query: Query<
        (&mut Transform, &RobotStats, &Chassis, &mut CurrentPath),
        With<RobotMarker>,
    >,
) {
    let dt = time.delta_secs();

    for (mut tf, stats, chassis, mut path) in &mut query {
        if path.index >= path.waypoints.len() {
            continue;
        }

        let target_cell = path.waypoints[path.index];
        let target_world = map.grid_to_world(target_cell.x, target_cell.y);
        let target_xz = Vec3::new(target_world.x, tf.translation.y, target_world.z);

        let dir = target_xz - tf.translation;
        let dist = dir.length();

        // Применяем замедление на песке для Wheels и Bipod
        let sand_mult = if matches!(
            map.world_to_grid(tf.translation).and_then(|(x, y)| map.get(x, y)),
            Some(CellType::Sand)
        ) && matches!(chassis.chassis_type, ChassisType::Wheels | ChassisType::Bipod)
        {
            SAND_SPEED_MULT
        } else {
            1.0
        };

        let step = stats.speed * CELL_SIZE * dt * sand_mult;

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

/// Расталкивание роботов при наложении через SpatialIndex (O(n·k), k — соседи в ячейке).
/// После толчка позиция зажимается по границам карты и не входит в непроходимые ячейки.
pub fn separate_robots(
    time: Res<Time>,
    map: Res<MapGrid>,
    index: Res<crate::spatial::SpatialIndex>,
    mut query: Query<(Entity, &mut Transform, &Chassis), With<RobotMarker>>,
    mut neighbors_buf: Local<Vec<(Entity, Vec3)>>,
) {
    const MIN_DIST: f32 = 1.0;
    const PUSH_SPEED: f32 = 2.5;

    let dt = time.delta_secs();

    debug_assert!(
        query.iter().count() < 200,
        "separate_robots: слишком много роботов ({}), проверь производительность",
        query.iter().count()
    );

    for (entity, mut tf, chassis) in &mut query {
        // Собираем соседей из SpatialIndex в переиспользуемый буфер.
        neighbors_buf.clear();
        let pos = tf.translation;
        index.query_radius(pos, MIN_DIST, |other_e, other_pos, _| {
            if other_e != entity {
                neighbors_buf.push((other_e, other_pos));
            }
        });

        let mut push = Vec3::ZERO;
        for &(_, other_pos) in neighbors_buf.iter() {
            let diff = Vec2::new(pos.x - other_pos.x, pos.z - other_pos.z);
            let dist = diff.length();
            if dist < MIN_DIST && dist > 0.001 {
                let strength = PUSH_SPEED * dt * (1.0 - dist / MIN_DIST);
                let push_xz = diff.normalize() * strength;
                push += Vec3::new(push_xz.x, 0.0, push_xz.y);
            }
        }

        if push == Vec3::ZERO {
            continue;
        }

        let new_pos = tf.translation + push;
        let map_max_x = map.width as f32 * CELL_SIZE - CELL_SIZE * 0.5;
        let map_max_z = map.height as f32 * CELL_SIZE - CELL_SIZE * 0.5;
        let clamped = Vec3::new(
            new_pos.x.clamp(CELL_SIZE * 0.5, map_max_x),
            tf.translation.y,
            new_pos.z.clamp(CELL_SIZE * 0.5, map_max_z),
        );

        let passable = map
            .world_to_grid(clamped)
            .and_then(|(gx, gy)| map.get(gx, gy))
            .map(|cell| cell.is_passable_for(chassis.chassis_type))
            .unwrap_or(false);

        if passable {
            tf.translation = clamped;
        }
    }
}

/// Если робот не двигается дольше порога — принудительно пересчитывает путь.
pub fn detect_stuck_robots(
    time: Res<Time>,
    mut query: Query<
        (&Transform, &mut CurrentPath, &mut StuckDetector),
        (With<RobotMarker>, With<MovementTarget>),
    >,
) {
    let dt = time.delta_secs();

    for (tf, mut path, mut stuck) in &mut query {
        // Если waypoints закончились — робот дошёл, сбрасываем таймер
        if path.index >= path.waypoints.len() {
            stuck.timer = 0.0;
            stuck.last_pos = tf.translation;
            continue;
        }

        let moved = tf.translation.distance(stuck.last_pos);
        if moved > 0.1 {
            // Нормально движется
            stuck.timer = 0.0;
            stuck.last_pos = tf.translation;
        } else {
            stuck.timer += dt;
            if stuck.timer >= stuck.threshold {
                // Застрял — сбрасываем last_target, compute_path пересчитает маршрут
                path.last_target = None;
                stuck.timer = 0.0;
            }
        }
    }
}

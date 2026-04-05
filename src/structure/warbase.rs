use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    core::Team,
    robot::{builder::RobotBlueprint, bundle::spawn_robot, registry::ModuleRegistry},
};

/// Маркер: эта сущность — Warbase (главная база).
/// Уничтожается только ядерным зарядом.
#[derive(Component)]
pub struct Warbase;

/// Задание на постройку робота в очереди.
#[derive(Clone, Debug)]
pub struct QueuedRobot {
    pub blueprint: RobotBlueprint,
    pub build_time: f32,
}

/// Очередь постройки роботов на варбейсе.
#[derive(Component, Default)]
pub struct ProductionQueue {
    pub queue: VecDeque<QueuedRobot>,
    /// Прошедшее время постройки текущего робота.
    pub build_timer: f32,
}

impl ProductionQueue {
    pub fn enqueue(&mut self, blueprint: RobotBlueprint, build_time: f32) {
        self.queue.push_back(QueuedRobot {
            blueprint,
            build_time,
        });
    }

    /// Дробь прогресса текущей постройки [0..1].
    pub fn progress_fraction(&self) -> f32 {
        match self.queue.front() {
            Some(q) => (self.build_timer / q.build_time).clamp(0.0, 1.0),
            None => 0.0,
        }
    }

    pub fn is_building(&self) -> bool {
        !self.queue.is_empty()
    }
}

/// Тик очереди постройки: продвигает таймер, спавнит готовых роботов.
pub fn tick_production_queue(
    time: Res<Time>,
    mut warbases: Query<(&mut ProductionQueue, &Transform, &Team), With<Warbase>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    registry: Res<ModuleRegistry>,
) {
    for (mut queue, tf, team) in &mut warbases {
        let Some(current_build_time) = queue.queue.front().map(|q| q.build_time) else {
            continue;
        };

        queue.build_timer += time.delta_secs();

        if queue.build_timer >= current_build_time {
            let finished = queue.queue.pop_front().unwrap();
            queue.build_timer = 0.0;

            // Спавн у входа варбейса (смещение по X)
            let spawn_pos = tf.translation + Vec3::new(3.0, 0.0, 0.0);
            spawn_robot(
                &mut commands,
                &mut meshes,
                &mut materials,
                &finished.blueprint,
                &registry,
                *team,
                spawn_pos,
            );

            info!("Робот построен у {:?} варбейса", team);
        }
    }
}

/// Gizmo прогресс-бар постройки над варбейсом.
pub fn draw_production_progress(
    warbases: Query<(&Transform, &ProductionQueue), With<Warbase>>,
    mut gizmos: Gizmos,
) {
    for (tf, queue) in &warbases {
        if !queue.is_building() {
            continue;
        }
        let frac = queue.progress_fraction();
        let base = tf.translation + Vec3::Y * 2.5;
        let half = 1.5_f32;

        gizmos.line(
            base - Vec3::X * half,
            base + Vec3::X * half,
            Color::srgb(0.2, 0.2, 0.2),
        );
        gizmos.line(
            base - Vec3::X * half,
            base + Vec3::X * (half * 2.0 * frac - half),
            Color::srgb(0.2, 0.8, 1.0),
        );

        // Подсчёт роботов в очереди
        let count = queue.queue.len();
        if count > 1 {
            // Маленькие точки — оставшиеся роботы
            for i in 1..count.min(5) {
                let dot_pos = base + Vec3::Z * 0.5 + Vec3::X * (-half + (i as f32) * 0.5);
                gizmos.sphere(Isometry3d::from_translation(dot_pos), 0.1, Color::srgb(0.2, 0.8, 1.0));
            }
        }
    }
}

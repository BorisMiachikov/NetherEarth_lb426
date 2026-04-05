use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    command::command::RobotCommand,
    core::{events::StructureCaptured, Team},
    movement::velocity::MovementTarget,
    robot::components::{RobotMarker, RobotStats},
};

/// Маркер: структура может быть захвачена.
#[derive(Component)]
pub struct Capturable;

/// Прогресс захвата структуры.
#[derive(Component, Debug, Clone)]
pub struct CaptureProgress {
    /// Текущий прогресс [0..required].
    pub progress: f32,
    /// Время захвата в секундах (зависит от типа шасси).
    pub required: f32,
}

impl CaptureProgress {
    pub fn new(required: f32) -> Self {
        Self {
            progress: 0.0,
            required,
        }
    }

    pub fn is_captured(&self) -> bool {
        self.progress >= self.required
    }

    pub fn fraction(&self) -> f32 {
        (self.progress / self.required).clamp(0.0, 1.0)
    }
}

/// Дистанция для начала захвата (game units).
pub const CAPTURE_RANGE: f32 = 2.5;

/// Цвет команды для перекраски структур.
pub fn team_color_core(team: Team) -> Color {
    match team {
        Team::Player => Color::srgb(0.15, 0.75, 0.2),
        Team::Enemy => Color::srgb(0.8, 0.15, 0.15),
        Team::Neutral => Color::srgb(0.55, 0.55, 0.55),
    }
}

/// Направляет роботов с приказом SeekAndCapture к ближайшей вражеской структуре.
/// Останавливает робота когда он в радиусе захвата.
pub fn seek_capture_navigation(
    mut commands: Commands,
    mut robots: Query<(Entity, &mut RobotCommand, &Transform, &Team), With<RobotMarker>>,
    capturable: Query<(Entity, &Transform, &Team), (With<Capturable>, With<CaptureProgress>)>,
) {
    for (robot_entity, mut cmd, robot_tf, robot_team) in &mut robots {
        let RobotCommand::SeekAndCapture(ref mut target_opt) = *cmd else {
            continue;
        };

        // Проверить валидность текущей цели
        let current_valid = target_opt
            .and_then(|t| capturable.get(t).ok())
            .filter(|(_, _, t)| *t != robot_team)
            .map(|(e, tf, _)| (e, tf.translation));

        // Если текущая цель невалидна — найти ближайшую
        let target = current_valid.or_else(|| {
            capturable
                .iter()
                .filter(|(_, _, t)| *t != robot_team)
                .min_by_key(|(_, tf, _)| {
                    (tf.translation.distance(robot_tf.translation) * 100.0) as u32
                })
                .map(|(e, tf, _)| (e, tf.translation))
        });

        if let Some((target_entity, target_pos)) = target {
            // Обновить цель только если изменилась
            if *target_opt != Some(target_entity) {
                *target_opt = Some(target_entity);
            }

            let dist = robot_tf.translation.xz().distance(target_pos.xz());
            if dist <= CAPTURE_RANGE {
                // В радиусе захвата — остановиться
                commands.entity(robot_entity).remove::<MovementTarget>();
            } else {
                commands
                    .entity(robot_entity)
                    .insert(MovementTarget(target_pos));
            }
        } else {
            // Нет целей — перейти в Idle
            *cmd = RobotCommand::Idle;
        }
    }
}

/// Накапливает прогресс захвата для структур (FixedUpdate).
/// Если роботы двух фракций в радиусе — захват оспорен, прогресс не идёт.
pub fn update_capture_progress(
    time: Res<Time>,
    mut structures: Query<(Entity, &mut CaptureProgress, &Transform, &Team), With<Capturable>>,
    robots: Query<(&Transform, &Team, &RobotStats, &RobotCommand), With<RobotMarker>>,
    mut commands: Commands,
) {
    let mut to_capture: Vec<(Entity, Team, Team)> = Vec::new();

    for (struct_entity, mut progress, struct_tf, struct_team) in &mut structures {
        // Собрать роботов в радиусе с приказом SeekAndCapture, не своей команды
        let mut rates: HashMap<Team, f32> = HashMap::new();

        for (robot_tf, robot_team, stats, cmd) in &robots {
            if !matches!(cmd, RobotCommand::SeekAndCapture(_)) {
                continue;
            }
            if *robot_team == *struct_team {
                continue; // Нельзя захватить свою структуру
            }
            let dist = robot_tf.translation.xz().distance(struct_tf.translation.xz());
            if dist > CAPTURE_RANGE {
                continue;
            }
            // Скорость захвата = 1 / capture_time (сек⁻¹)
            *rates.entry(*robot_team).or_insert(0.0) += 1.0 / stats.capture_time.max(0.1);
        }

        match rates.len() {
            0 => {} // Никто не захватывает
            1 => {
                let (&captor_team, &rate) = rates.iter().next().unwrap();
                progress.progress += rate * time.delta_secs();
                if progress.is_captured() {
                    to_capture.push((struct_entity, captor_team, *struct_team));
                }
            }
            _ => {} // Оспорено — прогресс заморожен
        }
    }

    for (entity, new_owner, old_owner) in to_capture {
        commands.trigger(StructureCaptured {
            structure: entity,
            new_owner,
            old_owner,
        });
    }
}

/// Observer: смена владельца и сброс прогресса при захвате.
pub fn on_structure_captured(
    trigger: On<StructureCaptured>,
    mut structures: Query<(
        &mut Team,
        &mut CaptureProgress,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ev = trigger.event();
    let Ok((mut team, mut progress, mat_handle)) = structures.get_mut(ev.structure) else {
        return;
    };
    *team = ev.new_owner;
    progress.progress = 0.0;
    if let Some(mat) = materials.get_mut(mat_handle.id()) {
        mat.base_color = team_color_core(ev.new_owner);
    }
    info!(
        "Структура захвачена: {:?} → {:?}",
        ev.old_owner, ev.new_owner
    );
}

/// Рисует прогресс-бар захвата над структурами (gizmos).
pub fn draw_capture_progress(
    structures: Query<(&Transform, &CaptureProgress), With<Capturable>>,
    mut gizmos: Gizmos,
) {
    for (tf, progress) in &structures {
        if progress.progress <= 0.0 {
            continue;
        }
        let frac = progress.fraction();
        let base = tf.translation + Vec3::Y * 1.8;
        let half = 1.0_f32;

        // Фоновая полоса
        gizmos.line(
            base - Vec3::X * half,
            base + Vec3::X * half,
            Color::srgb(0.3, 0.3, 0.3),
        );
        // Прогрессивная полоса: красный → зелёный
        let bar_color = Color::srgb(1.0 - frac, frac, 0.0);
        gizmos.line(
            base - Vec3::X * half,
            base + Vec3::X * (half * 2.0 * frac - half),
            bar_color,
        );
    }
}

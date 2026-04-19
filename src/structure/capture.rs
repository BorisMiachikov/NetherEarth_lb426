use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    command::command::RobotCommand,
    core::{events::StructureCaptured, Team},
    movement::{exploration_target, velocity::MovementTarget},
    robot::components::{RobotMarker, RobotStats, VisionRange},
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

/// Базовое время захвата (сек) — одинаково для всех шасси.
pub const BASE_CAPTURE_TIME: f32 = 10.0;

/// Цвет команды для перекраски структур.
pub fn team_color_core(team: Team) -> Color {
    match team {
        Team::Player => Color::srgb(0.15, 0.75, 0.2),
        Team::Enemy => Color::srgb(0.8, 0.15, 0.15),
        Team::Neutral => Color::srgb(0.55, 0.55, 0.55),
    }
}

/// Направляет роботов с приказом SeekAndCapture к ближайшей видимой чужой структуре.
/// Если структур в радиусе видимости нет — исследует карту.
pub fn seek_capture_navigation(
    mut commands: Commands,
    mut robots: Query<
        (Entity, &mut RobotCommand, &Transform, &Team, &VisionRange, Option<&MovementTarget>),
        With<RobotMarker>,
    >,
    capturable: Query<(Entity, &Transform, &Team), (With<Capturable>, With<CaptureProgress>)>,
    map: Res<crate::map::grid::MapGrid>,
) {
    for (robot_entity, mut cmd, robot_tf, robot_team, vision, cur_target) in &mut robots {
        let RobotCommand::SeekAndCapture(ref mut target_opt) = *cmd else {
            continue;
        };

        let robot_pos = robot_tf.translation;

        // Ближайшая видимая чужая структура (дистанция + LOS)
        let from_cell = map.world_to_grid(robot_pos);
        let visible_target = capturable
            .iter()
            .filter(|(_, _, t)| *t != robot_team)
            .filter(|(_, tf, _)| robot_pos.distance(tf.translation) <= vision.0)
            .filter(|(_, tf, _)| {
                let to_cell = map.world_to_grid(tf.translation);
                match (from_cell, to_cell) {
                    (Some(f), Some(t)) => map.has_line_of_sight(f, t),
                    _ => false,
                }
            })
            .min_by_key(|(_, tf, _)| (robot_pos.distance(tf.translation) * 100.0) as u32)
            .map(|(e, tf, _)| (e, tf.translation));

        if let Some((target_entity, target_pos)) = visible_target {
            if *target_opt != Some(target_entity) {
                *target_opt = Some(target_entity);
            }
            let dist = robot_pos.xz().distance(target_pos.xz());
            if dist <= CAPTURE_RANGE {
                commands.entity(robot_entity).remove::<MovementTarget>();
            } else {
                commands.entity(robot_entity).try_insert(MovementTarget(target_pos));
            }
        } else {
            // Ничего не видно — исследовать карту
            let near_target = cur_target
                .map_or(true, |t| robot_pos.xz().distance(t.0.xz()) < 2.0);
            if near_target {
                let explore = exploration_target(robot_entity, robot_pos, map.width, map.height);
                commands.entity(robot_entity).try_insert(MovementTarget(explore));
            }
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
                debug_assert!(
                    progress.progress >= 0.0,
                    "CaptureProgress.progress отрицательный: {}",
                    progress.progress
                );
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

/// Observer: смена владельца при захвате.
/// Если старый владелец — не нейтральный (вражеский/свой), завод сначала становится
/// нейтральным (удвоенное время захвата). Если старый владелец нейтральный — переходит
/// к захватчику сразу.
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

    let new_team = if ev.old_owner == Team::Neutral {
        // Нейтральный завод → сразу к захватчику
        ev.new_owner
    } else {
        // Вражеский (или свой перехваченный) → сначала нейтральный
        Team::Neutral
    };

    *team = new_team;
    progress.progress = 0.0;

    if let Some(mat) = materials.get_mut(mat_handle.id()) {
        mat.base_color = team_color_core(new_team);
    }

    info!(
        "Структура: {:?} → {:?}{}",
        ev.old_owner,
        new_team,
        if new_team == Team::Neutral { " (нейтральная, требует повторного захвата)" } else { "" }
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

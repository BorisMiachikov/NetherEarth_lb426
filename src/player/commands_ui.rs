use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{
    camera::systems::IsometricCamera,
    command::command::RobotCommand,
    core::Team,
    robot::components::RobotMarker,
};

use super::selection::{Selected, SelectionState};

/// Ресурс: состояние UI команд.
#[derive(Resource, Default)]
pub struct CommandUiState {
    pub patrol_points: Vec<Vec3>,
    pub show_patrol_hint: bool,
}

/// ПКМ на земле → MoveTo для выбранных роботов.
pub fn right_click_move(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<IsometricCamera>>,
    selection: Res<SelectionState>,
    mut robot_cmds: Query<&mut RobotCommand, (With<RobotMarker>, With<Selected>)>,
    mut cmd_ui: ResMut<CommandUiState>,
) {
    if !mouse.just_pressed(MouseButton::Right) || selection.selected.is_empty() {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, cam_tf)) = camera_q.single() else { return };
    let Ok(ray) = camera.viewport_to_world(cam_tf, cursor) else { return };

    let Some(ground_hit) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };
    let target = ray.origin + ray.direction * ground_hit;

    let is_patrol = keys.pressed(KeyCode::KeyP);

    if is_patrol {
        cmd_ui.patrol_points.push(target);
        cmd_ui.show_patrol_hint = true;
        if cmd_ui.patrol_points.len() >= 2 {
            let points = cmd_ui.patrol_points.clone();
            for mut cmd in &mut robot_cmds {
                *cmd = RobotCommand::Patrol(points.clone());
            }
            cmd_ui.patrol_points.clear();
            cmd_ui.show_patrol_hint = false;
        }
    } else {
        for mut cmd in &mut robot_cmds {
            *cmd = RobotCommand::MoveTo(target);
        }
    }
}

/// Панель информации о выбранном роботе + кнопки команд.
pub fn robot_info_panel(
    mut contexts: EguiContexts,
    selection: Res<SelectionState>,
    robots: Query<(&RobotCommand, &crate::core::Health, &crate::robot::components::Chassis,
                   &crate::robot::components::WeaponSlots, &Team), With<Selected>>,
    enemies: Query<(Entity, &Transform, &Team), With<RobotMarker>>,
    mut robot_cmds: Query<&mut RobotCommand, (With<RobotMarker>, With<Selected>)>,
    cmd_ui: Res<CommandUiState>,
) -> Result {
    if selection.selected.is_empty() {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Window::new("Выбранный робот")
        .default_pos([10.0, 350.0])
        .resizable(false)
        .show(ctx, |ui| {
            // Показываем только первого выбранного
            if let Ok((cmd, health, chassis, weapons, team)) = robots.single() {
                ui.label(format!("Шасси: {:?}", chassis.chassis_type));
                ui.label(format!("Команда: {team:?}"));
                ui.label(format!("HP: {:.0} / {:.0}", health.current, health.max));
                ui.label(format!("Оружия: {}", weapons.count()));
                ui.separator();
                ui.label(format!("Приказ: {}", cmd_label(cmd)));
                ui.separator();

                // Кнопки команд
                if ui.button("MoveTo (ПКМ)").clicked() {
                    ui.close_menu();
                }
                if ui.button("SeekAndDestroy").clicked() {
                    // Найти ближайшего врага
                    if let Ok((mut robot_cmd, _, _, _, my_team)) = robots.single() {
                        // найдём через отдельный запрос ниже
                    }
                    for mut cmd in &mut robot_cmds {
                        *cmd = RobotCommand::SeekAndDestroy(None);
                    }
                }
                if ui.button("SeekAndCapture").clicked() {
                    for mut cmd in &mut robot_cmds {
                        *cmd = RobotCommand::SeekAndCapture(None);
                    }
                }
                if ui.button("Defend (ПКМ = позиция)").clicked() {
                    // Позиция задаётся правым кликом — здесь просто Defend(Vec3::ZERO)
                    for mut cmd in &mut robot_cmds {
                        *cmd = RobotCommand::Defend(Vec3::ZERO);
                    }
                }
                if ui.button("Idle").clicked() {
                    for mut cmd in &mut robot_cmds {
                        *cmd = RobotCommand::Idle;
                    }
                }

                if cmd_ui.show_patrol_hint {
                    ui.separator();
                    ui.colored_label(egui::Color32::YELLOW,
                        format!("Patrol: {} точек (P+ПКМ для добавления)", cmd_ui.patrol_points.len()));
                }
            } else {
                ui.label(format!("Выбрано роботов: {}", selection.selected.len()));
            }
        });

    Ok(())
}

fn cmd_label(cmd: &RobotCommand) -> &'static str {
    match cmd {
        RobotCommand::Idle => "Idle",
        RobotCommand::MoveTo(_) => "MoveTo",
        RobotCommand::SeekAndDestroy(_) => "SeekAndDestroy",
        RobotCommand::SeekAndCapture(_) => "SeekAndCapture",
        RobotCommand::Defend(_) => "Defend",
        RobotCommand::Patrol(_) => "Patrol",
    }
}

/// Рисует линии к целям выбранных роботов + индикаторы Patrol.
pub fn draw_command_indicators(
    mut gizmos: Gizmos,
    robots: Query<(&Transform, &RobotCommand), With<Selected>>,
    cmd_ui: Res<CommandUiState>,
) {
    for (tf, cmd) in &robots {
        match cmd {
            RobotCommand::MoveTo(target) => {
                gizmos.line(tf.translation, *target, Color::srgb(0.0, 1.0, 1.0));
                gizmos.sphere(*target, 0.3, Color::srgb(0.0, 1.0, 1.0));
            }
            RobotCommand::Patrol(points) if points.len() >= 2 => {
                for i in 0..points.len() {
                    let a = points[i];
                    let b = points[(i + 1) % points.len()];
                    gizmos.line(a, b, Color::srgb(1.0, 0.5, 0.0));
                }
            }
            RobotCommand::Defend(pos) => {
                gizmos.circle(
                    Isometry3d::new(*pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                    3.0,
                    Color::srgb(0.0, 1.0, 0.0),
                );
            }
            _ => {}
        }
    }

    // Промежуточные точки Patrol в процессе набора
    for (i, p) in cmd_ui.patrol_points.iter().enumerate() {
        gizmos.sphere(*p, 0.4, Color::srgb(1.0, 0.5, 0.0));
        if i > 0 {
            gizmos.line(cmd_ui.patrol_points[i - 1], *p, Color::srgb(1.0, 0.5, 0.0));
        }
    }
}

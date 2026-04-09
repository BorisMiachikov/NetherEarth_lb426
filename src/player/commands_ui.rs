use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    camera::systems::IsometricCamera,
    command::command::RobotCommand,
    core::{Health, Team},
    robot::components::{Chassis, ChassisType, Nuclear, RobotMarker, WeaponSlots},
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

/// Сводная информация по одному роботу.
struct SingleInfo {
    chassis: ChassisType,
    team: Team,
    hp: f32,
    hp_max: f32,
    weapons: usize,
    has_nuclear: bool,
    cmd: &'static str,
    pos: Vec3,
}

/// Сводная информация по нескольким роботам.
struct MultiInfo {
    count: usize,
    avg_hp_pct: f32,
    wheels: usize,
    bipod: usize,
    tracks: usize,
    antigrav: usize,
    has_nuclear: bool,
    cmd_counts: [(&'static str, usize); 7],
}

/// Панель информации о выбранных роботах + кнопки команд.
pub fn robot_info_panel(
    mut contexts: EguiContexts,
    mut selection: ResMut<SelectionState>,
    mut robots: Query<
        (
            &mut RobotCommand,
            &Transform,
            &Health,
            &Chassis,
            &WeaponSlots,
            &Team,
            Option<&Nuclear>,
        ),
        With<Selected>,
    >,
    selected_entities: Query<Entity, With<Selected>>,
    cmd_ui: Res<CommandUiState>,
    mut commands: Commands,
) -> Result {
    if selection.selected.is_empty() {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    let count = selection.selected.len();

    // Собираем данные (иммутабельный проход)
    let single: Option<SingleInfo>;
    let multi: Option<MultiInfo>;

    if count == 1 {
        single = robots.single().ok().map(|(cmd, tf, hp, chassis, weapons, team, nuc)| SingleInfo {
            chassis: chassis.chassis_type,
            team: *team,
            hp: hp.current,
            hp_max: hp.max,
            weapons: weapons.count(),
            has_nuclear: nuc.is_some(),
            cmd: cmd_label(&cmd),
            pos: tf.translation,
        });
        multi = None;
    } else {
        single = None;

        let mut total_hp_pct = 0.0f32;
        let mut wheels = 0usize;
        let mut bipod = 0usize;
        let mut tracks = 0usize;
        let mut antigrav = 0usize;
        let mut has_nuclear = false;
        let cmds_order = [
            "Idle", "MoveTo", "SeekAndDestroy", "SeekAndCapture", "DestroyEnemyBase", "Defend", "Patrol",
        ];
        let mut cmd_counts = [0usize; 7];

        for (cmd, _, hp, chassis, _, _, nuc) in robots.iter() {
            total_hp_pct += hp.current / hp.max.max(1.0);
            if nuc.is_some() { has_nuclear = true; }
            match chassis.chassis_type {
                ChassisType::Wheels   => wheels += 1,
                ChassisType::Bipod    => bipod += 1,
                ChassisType::Tracks   => tracks += 1,
                ChassisType::AntiGrav => antigrav += 1,
            }
            let label = cmd_label(&cmd);
            if let Some(idx) = cmds_order.iter().position(|&c| c == label) {
                cmd_counts[idx] += 1;
            }
        }

        multi = Some(MultiInfo {
            count,
            avg_hp_pct: total_hp_pct / count as f32,
            wheels,
            bipod,
            tracks,
            antigrav,
            has_nuclear,
            cmd_counts: cmds_order
                .iter()
                .zip(cmd_counts.iter())
                .map(|(&n, &c)| (n, c))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        });
    }

    let mut new_cmd: Option<RobotCommand> = None;
    let mut deselect = false;

    // Есть ли ядерный заряд среди выбранных
    let any_nuclear = single.as_ref().map_or(false, |s| s.has_nuclear)
        || multi.as_ref().map_or(false, |m| m.has_nuclear);

    egui::Window::new("Юниты")
        .id(egui::Id::new("robot_panel"))
        .default_pos([10.0, 310.0])
        .resizable(false)
        .collapsible(true)
        .show(ctx, |ui| {
            if let Some(ref s) = single {
                // === Один робот ===
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{:?}", s.chassis))
                            .strong()
                            .color(team_color_egui(s.team)),
                    );
                    ui.label(
                        egui::RichText::new(format!("{:?}", s.team))
                            .color(team_color_egui(s.team))
                            .small(),
                    );
                });

                // HP-бар
                let hp_pct = (s.hp / s.hp_max.max(1.0)).clamp(0.0, 1.0);
                let hp_color = if hp_pct > 0.6 {
                    egui::Color32::from_rgb(60, 180, 60)
                } else if hp_pct > 0.3 {
                    egui::Color32::from_rgb(220, 180, 40)
                } else {
                    egui::Color32::from_rgb(220, 60, 60)
                };
                let bar_w = ui.available_width().min(160.0);
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(bar_w, 10.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 2.0, egui::Color32::from_rgb(40, 40, 40));
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * hp_pct, rect.height())),
                    2.0,
                    hp_color,
                );
                ui.label(
                    egui::RichText::new(format!("HP {:.0}/{:.0}  ⚙{}", s.hp, s.hp_max, s.weapons))
                        .small()
                        .color(egui::Color32::GRAY),
                );

                ui.separator();
                ui.label(
                    egui::RichText::new(format!("▸ {}", s.cmd))
                        .color(egui::Color32::from_rgb(180, 220, 255)),
                );
            } else if let Some(ref m) = multi {
                // === Несколько роботов ===
                ui.label(
                    egui::RichText::new(format!("Выбрано: {} роботов", m.count))
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 200)),
                );

                // Состав по шасси
                ui.horizontal_wrapped(|ui| {
                    if m.wheels   > 0 { ui.label(egui::RichText::new(format!("Кол.{}", m.wheels)).small()); }
                    if m.bipod    > 0 { ui.label(egui::RichText::new(format!("Бип.{}", m.bipod)).small()); }
                    if m.tracks   > 0 { ui.label(egui::RichText::new(format!("Гус.{}", m.tracks)).small()); }
                    if m.antigrav > 0 { ui.label(egui::RichText::new(format!("АГр.{}", m.antigrav)).small()); }
                });

                // Средний HP
                let bar_w = ui.available_width().min(160.0);
                let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_w, 8.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, egui::Color32::from_rgb(40, 40, 40));
                let hp_color = if m.avg_hp_pct > 0.6 {
                    egui::Color32::from_rgb(60, 180, 60)
                } else if m.avg_hp_pct > 0.3 {
                    egui::Color32::from_rgb(220, 180, 40)
                } else {
                    egui::Color32::from_rgb(220, 60, 60)
                };
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * m.avg_hp_pct, rect.height())),
                    2.0,
                    hp_color,
                );
                ui.label(
                    egui::RichText::new(format!("Средний HP: {:.0}%", m.avg_hp_pct * 100.0))
                        .small()
                        .color(egui::Color32::GRAY),
                );

                // Разброс приказов
                let active: Vec<&str> = m.cmd_counts.iter()
                    .filter(|(_, c)| *c > 0)
                    .map(|(n, _)| *n)
                    .collect();
                if !active.is_empty() {
                    ui.label(
                        egui::RichText::new(format!("▸ {}", active.join(", ")))
                            .small()
                            .color(egui::Color32::from_rgb(180, 220, 255)),
                    );
                }
            }

            ui.separator();

            // === Кнопки команд (работают для всех выбранных) ===
            ui.label(
                egui::RichText::new("КОМАНДЫ").small().color(egui::Color32::DARK_GRAY),
            );

            ui.columns(2, |cols| {
                if cols[0].button("⚔ Атаковать").clicked() {
                    new_cmd = Some(RobotCommand::SeekAndDestroy(None));
                }
                if cols[0].button("⚑ Захватить").clicked() {
                    new_cmd = Some(RobotCommand::SeekAndCapture(None));
                }
                if cols[1].button("⬡ Держать").clicked() {
                    new_cmd = Some(RobotCommand::Defend(Vec3::ZERO));
                }
                if cols[1].button("◻ Стоп").clicked() {
                    new_cmd = Some(RobotCommand::Idle);
                }
            });
            if any_nuclear {
                if ui.button("☢ Уничтожить базу").clicked() {
                    new_cmd = Some(RobotCommand::DestroyEnemyBase(None));
                }
            }

            ui.label(
                egui::RichText::new("ПКМ = Двигаться  |  P+ПКМ = Патруль")
                    .small()
                    .color(egui::Color32::DARK_GRAY),
            );

            if cmd_ui.show_patrol_hint {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("Patrol: {} точек", cmd_ui.patrol_points.len()),
                );
            }

            ui.separator();
            if ui
                .add_sized(
                    [ui.available_width(), 22.0],
                    egui::Button::new(
                        egui::RichText::new("✕ Снять выбор")
                            .small()
                            .color(egui::Color32::GRAY),
                    ),
                )
                .clicked()
            {
                deselect = true;
            }
        });

    // Снять выбор (кнопка в панели)
    if deselect {
        for e in &selected_entities {
            commands.entity(e).remove::<Selected>();
        }
        selection.selected.clear();
        return Ok(());
    }

    // Применяем команду ко всем выбранным роботам
    if let Some(cmd) = new_cmd {
        for (mut robot_cmd, tf, ..) in &mut robots {
            *robot_cmd = if matches!(cmd, RobotCommand::Defend(_)) {
                RobotCommand::Defend(tf.translation)
            } else {
                cmd.clone()
            };
        }
    }

    Ok(())
}

fn cmd_label(cmd: &RobotCommand) -> &'static str {
    match cmd {
        RobotCommand::Idle                => "Idle",
        RobotCommand::MoveTo(_)           => "MoveTo",
        RobotCommand::SeekAndDestroy(_)   => "SeekAndDestroy",
        RobotCommand::SeekAndCapture(_)   => "SeekAndCapture",
        RobotCommand::DestroyEnemyBase(_) => "DestroyEnemyBase",
        RobotCommand::Defend(_)           => "Defend",
        RobotCommand::Patrol(_)           => "Patrol",
    }
}

fn team_color_egui(team: Team) -> egui::Color32 {
    match team {
        Team::Player  => egui::Color32::from_rgb(60, 200, 100),
        Team::Enemy   => egui::Color32::from_rgb(220, 80, 80),
        Team::Neutral => egui::Color32::GRAY,
    }
}

/// Рисует gizmo-линии к целям выбранных роботов + индикаторы Patrol.
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

    for (i, p) in cmd_ui.patrol_points.iter().enumerate() {
        gizmos.sphere(*p, 0.4, Color::srgb(1.0, 0.5, 0.0));
        if i > 0 {
            gizmos.line(cmd_ui.patrol_points[i - 1], *p, Color::srgb(1.0, 0.5, 0.0));
        }
    }
}

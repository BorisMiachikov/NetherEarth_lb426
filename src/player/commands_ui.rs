use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    camera::systems::{CameraTarget, IsometricCamera},
    command::command::RobotCommand,
    core::{Health, Team},
    robot::components::{Chassis, ChassisType, Nuclear, RobotMarker, WeaponSlots},
};

use super::{
    components::{ManualControl, PlayerScout},
    selection::{Selected, SelectionState},
};

/// Ресурс: состояние UI команд.
#[derive(Resource, Default)]
pub struct CommandUiState {
    pub patrol_points: Vec<Vec3>,
    pub show_patrol_hint: bool,
}

// ──── типы данных ────────────────────────────────────────────────────────────

struct SingleInfo {
    chassis: ChassisType,
    team: Team,
    hp: f32,
    hp_max: f32,
    weapons: usize,
    has_nuclear: bool,
    is_manual: bool,
    cmd: &'static str,
}

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

enum SelectionInfo {
    Single(SingleInfo),
    Multi(MultiInfo),
}

impl SelectionInfo {
    fn has_nuclear(&self) -> bool {
        match self {
            SelectionInfo::Single(s) => s.has_nuclear,
            SelectionInfo::Multi(m) => m.has_nuclear,
        }
    }
}

#[derive(Default)]
struct UiActions {
    new_cmd: Option<RobotCommand>,
    deselect: bool,
    toggle_manual: Option<Entity>,
}

// ──── right_click_move ───────────────────────────────────────────────────────

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

// ──── collect_selection_info ─────────────────────────────────────────────────

fn collect_selection_info(
    count: usize,
    robots: &Query<
        (
            &mut RobotCommand,
            &Transform,
            &Health,
            &Chassis,
            &WeaponSlots,
            &Team,
            Option<&Nuclear>,
            Option<&ManualControl>,
        ),
        With<Selected>,
    >,
) -> SelectionInfo {
    if count == 1 {
        let info = robots
            .single()
            .ok()
            .map(|(cmd, _tf, hp, chassis, weapons, team, nuc, manual)| SingleInfo {
                chassis: chassis.chassis_type,
                team: *team,
                hp: hp.current,
                hp_max: hp.max,
                weapons: weapons.count(),
                has_nuclear: nuc.is_some(),
                is_manual: manual.is_some(),
                cmd: cmd_label(&cmd),
            });
        return SelectionInfo::Single(info.unwrap_or(SingleInfo {
            chassis: ChassisType::Wheels,
            team: Team::Player,
            hp: 0.0,
            hp_max: 1.0,
            weapons: 0,
            has_nuclear: false,
            is_manual: false,
            cmd: "Idle",
        }));
    }

    const CMDS: [&str; 7] = [
        "Idle", "MoveTo", "SeekAndDestroy", "SeekAndCapture",
        "DestroyEnemyBase", "Defend", "Patrol",
    ];
    let mut total_hp_pct = 0.0f32;
    let mut wheels = 0usize;
    let mut bipod = 0usize;
    let mut tracks = 0usize;
    let mut antigrav = 0usize;
    let mut has_nuclear = false;
    let mut cmd_counts = [0usize; 7];

    for (cmd, _, hp, chassis, _, _, nuc, _) in robots.iter() {
        total_hp_pct += hp.current / hp.max.max(1.0);
        if nuc.is_some() { has_nuclear = true; }
        match chassis.chassis_type {
            ChassisType::Wheels   => wheels += 1,
            ChassisType::Bipod    => bipod += 1,
            ChassisType::Tracks   => tracks += 1,
            ChassisType::AntiGrav => antigrav += 1,
        }
        let label = cmd_label(&cmd);
        if let Some(idx) = CMDS.iter().position(|&c| c == label) {
            cmd_counts[idx] += 1;
        }
    }

    SelectionInfo::Multi(MultiInfo {
        count,
        avg_hp_pct: total_hp_pct / count as f32,
        wheels,
        bipod,
        tracks,
        antigrav,
        has_nuclear,
        cmd_counts: CMDS
            .iter()
            .zip(cmd_counts.iter())
            .map(|(&n, &c)| (n, c))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    })
}

// ──── draw_single_robot_panel ────────────────────────────────────────────────

fn draw_single_robot_panel(
    ui: &mut egui::Ui,
    info: &SingleInfo,
    single_entity: Option<Entity>,
    actions: &mut UiActions,
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{:?}", info.chassis))
                .strong()
                .color(team_color_egui(info.team)),
        );
        ui.label(
            egui::RichText::new(format!("{:?}", info.team))
                .color(team_color_egui(info.team))
                .small(),
        );
    });

    let hp_pct = (info.hp / info.hp_max.max(1.0)).clamp(0.0, 1.0);
    let hp_color = hp_bar_color(hp_pct);
    let bar_w = ui.available_width().min(160.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_w, 10.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 2.0, egui::Color32::from_rgb(40, 40, 40));
    ui.painter().rect_filled(
        egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * hp_pct, rect.height())),
        2.0,
        hp_color,
    );
    ui.label(
        egui::RichText::new(format!("HP {:.0}/{:.0}  ⚙{}", info.hp, info.hp_max, info.weapons))
            .small()
            .color(egui::Color32::GRAY),
    );

    if info.is_manual {
        ui.colored_label(egui::Color32::from_rgb(0, 220, 255), "⊕ РУЧНОЕ УПРАВЛЕНИЕ");
    }

    ui.separator();
    ui.label(
        egui::RichText::new(format!("▸ {}", info.cmd))
            .color(egui::Color32::from_rgb(180, 220, 255)),
    );

    let label = if info.is_manual { "✋ Выйти из управления" } else { "✋ Ручное управление" };
    if ui.add_sized([ui.available_width(), 22.0], egui::Button::new(label)).clicked() {
        actions.toggle_manual = single_entity;
    }

    if info.is_manual {
        ui.label(
            egui::RichText::new("WASD = движение  |  Ctrl+LMB = выход")
                .small()
                .color(egui::Color32::from_rgb(0, 180, 220)),
        );
    }
}

// ──── draw_multi_robot_panel ─────────────────────────────────────────────────

fn draw_multi_robot_panel(ui: &mut egui::Ui, info: &MultiInfo) {
    ui.label(
        egui::RichText::new(format!("Выбрано: {} роботов", info.count))
            .strong()
            .color(egui::Color32::from_rgb(200, 200, 200)),
    );

    ui.horizontal_wrapped(|ui| {
        if info.wheels   > 0 { ui.label(egui::RichText::new(format!("Кол.{}", info.wheels)).small()); }
        if info.bipod    > 0 { ui.label(egui::RichText::new(format!("Бип.{}", info.bipod)).small()); }
        if info.tracks   > 0 { ui.label(egui::RichText::new(format!("Гус.{}", info.tracks)).small()); }
        if info.antigrav > 0 { ui.label(egui::RichText::new(format!("АГр.{}", info.antigrav)).small()); }
    });

    let bar_w = ui.available_width().min(160.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_w, 8.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 2.0, egui::Color32::from_rgb(40, 40, 40));
    let hp_color = hp_bar_color(info.avg_hp_pct);
    ui.painter().rect_filled(
        egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * info.avg_hp_pct, rect.height())),
        2.0,
        hp_color,
    );
    ui.label(
        egui::RichText::new(format!("Средний HP: {:.0}%", info.avg_hp_pct * 100.0))
            .small()
            .color(egui::Color32::GRAY),
    );

    let active: Vec<&str> = info.cmd_counts.iter()
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

// ──── draw_command_buttons ───────────────────────────────────────────────────

fn draw_command_buttons(
    ui: &mut egui::Ui,
    info: &SelectionInfo,
    cmd_ui: &CommandUiState,
    actions: &mut UiActions,
) {
    ui.separator();
    ui.label(egui::RichText::new("КОМАНДЫ").small().color(egui::Color32::DARK_GRAY));

    ui.columns(2, |cols| {
        if cols[0].button("⚔ Атаковать").clicked() {
            actions.new_cmd = Some(RobotCommand::SeekAndDestroy(None));
        }
        if cols[0].button("⚑ Захватить").clicked() {
            actions.new_cmd = Some(RobotCommand::SeekAndCapture(None));
        }
        if cols[1].button("⬡ Держать").clicked() {
            actions.new_cmd = Some(RobotCommand::Defend(Vec3::ZERO));
        }
        if cols[1].button("◻ Стоп").clicked() {
            actions.new_cmd = Some(RobotCommand::Idle);
        }
    });

    if info.has_nuclear() && ui.button("☢ Уничтожить базу").clicked() {
        actions.new_cmd = Some(RobotCommand::DestroyEnemyBase(None));
    }

    let is_manual = matches!(info, SelectionInfo::Single(s) if s.is_manual);
    if !is_manual {
        ui.label(
            egui::RichText::new("ПКМ = Двигаться  |  P+ПКМ = Патруль  |  Ctrl+LMB = ручное")
                .small()
                .color(egui::Color32::DARK_GRAY),
        );
    }

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
                egui::RichText::new("✕ Снять выбор").small().color(egui::Color32::GRAY),
            ),
        )
        .clicked()
    {
        actions.deselect = true;
    }
}

// ──── handle_selection_actions ───────────────────────────────────────────────

fn handle_selection_actions(
    actions: UiActions,
    selection: &mut SelectionState,
    robots: &mut Query<
        (
            &mut RobotCommand,
            &Transform,
            &Health,
            &Chassis,
            &WeaponSlots,
            &Team,
            Option<&Nuclear>,
            Option<&ManualControl>,
        ),
        With<Selected>,
    >,
    selected_entities: &Query<Entity, With<Selected>>,
    manual_query: &Query<Entity, With<ManualControl>>,
    scout_query: &Query<Entity, With<PlayerScout>>,
    commands: &mut Commands,
) {
    if let Some(target) = actions.toggle_manual {
        let already_manual = manual_query.get(target).is_ok();
        for e in manual_query.iter() {
            commands.entity(e).remove::<ManualControl>();
        }
        if let Ok(scout) = scout_query.single() {
            commands.entity(scout).try_insert(CameraTarget);
        }
        if !already_manual {
            commands.entity(target).insert(ManualControl).insert(RobotCommand::Idle);
            if let Ok(scout) = scout_query.single() {
                commands.entity(scout).remove::<CameraTarget>();
            }
            commands.entity(target).insert(CameraTarget);
        }
        return;
    }

    if actions.deselect {
        for e in selected_entities.iter() {
            commands.entity(e).remove::<Selected>();
        }
        selection.selected.clear();
        for e in manual_query.iter() {
            commands.entity(e).remove::<ManualControl>();
        }
        if let Ok(scout) = scout_query.single() {
            commands.entity(scout).try_insert(CameraTarget);
        }
        return;
    }

    if let Some(cmd) = actions.new_cmd {
        for (mut robot_cmd, tf, _, _, _, _, _, _) in robots.iter_mut() {
            *robot_cmd = if matches!(cmd, RobotCommand::Defend(_)) {
                RobotCommand::Defend(tf.translation)
            } else {
                cmd.clone()
            };
        }
    }
}

// ──── robot_info_panel (оркестратор) ────────────────────────────────────────

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
            Option<&ManualControl>,
        ),
        With<Selected>,
    >,
    selected_entities: Query<Entity, With<Selected>>,
    manual_query: Query<Entity, With<ManualControl>>,
    scout_query: Query<Entity, With<PlayerScout>>,
    cmd_ui: Res<CommandUiState>,
    mut commands: Commands,
) -> Result {
    if selection.selected.is_empty() {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    let info = collect_selection_info(selection.selected.len(), &robots);
    let single_entity = selection.selected.first().copied();
    let mut actions = UiActions::default();

    egui::Window::new("Юниты")
        .id(egui::Id::new("robot_panel"))
        .default_pos([10.0, 310.0])
        .resizable(false)
        .collapsible(true)
        .show(ctx, |ui| {
            match &info {
                SelectionInfo::Single(s) => draw_single_robot_panel(ui, s, single_entity, &mut actions),
                SelectionInfo::Multi(m)  => draw_multi_robot_panel(ui, m),
            }
            draw_command_buttons(ui, &info, &cmd_ui, &mut actions);
        });

    handle_selection_actions(
        actions, &mut selection, &mut robots,
        &selected_entities, &manual_query, &scout_query, &mut commands,
    );
    Ok(())
}

// ──── helpers ────────────────────────────────────────────────────────────────

fn hp_bar_color(pct: f32) -> egui::Color32 {
    if pct > 0.6 {
        egui::Color32::from_rgb(60, 180, 60)
    } else if pct > 0.3 {
        egui::Color32::from_rgb(220, 180, 40)
    } else {
        egui::Color32::from_rgb(220, 60, 60)
    }
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

/// Рисует gizmo-линии к целям выбранных роботов + индикаторы Patrol + ручного управления.
pub fn draw_command_indicators(
    mut gizmos: Gizmos,
    robots: Query<(&Transform, &RobotCommand), With<Selected>>,
    manual_robots: Query<&Transform, With<ManualControl>>,
    cmd_ui: Res<CommandUiState>,
    time: Res<Time>,
) {
    for tf in &manual_robots {
        let pulse = (time.elapsed_secs() * 4.0).sin() * 0.12 + 0.88;
        let pos = tf.translation.with_y(0.06);
        let iso = Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
        gizmos.circle(iso, 1.0 * pulse, Color::srgb(0.0, 0.9, 1.0));
        gizmos.circle(iso, 1.25 * pulse, Color::srgba(0.0, 0.7, 1.0, 0.4));
    }

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

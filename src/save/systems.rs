use bevy::prelude::*;

use crate::{
    ai::state::{AICommander, GameResult},
    command::command::RobotCommand,
    core::{Health, Team},
    core::time::GameTime,
    economy::resource::{EnemyResources, PlayerResources, ResourceType},
    map::{
        grid::MapGrid,
        loader::{MapSpawnPoints, MapStructures, TeamDef},
    },
    player::{
        commands_ui::CommandUiState,
        components::PlayerScout,
        selection::SelectionState,
    },
    robot::{
        builder::RobotBlueprint,
        bundle::spawn_robot,
        components::{Electronics, Nuclear, RobotMarker, WeaponSlots, WeaponType},
        registry::ModuleRegistry,
    },
    structure::{
        capture::{team_color_core, CaptureProgress},
        factory::{Factory, FactoryType},
        warbase::Warbase,
    },
};

use super::{
    io::{autosave_exists, read_save, slot_exists, slot_path, write_save, AUTOSAVE_FILE},
    types::{SaveData, SavedAI, SavedCommand, SavedFactory, SavedResources, SavedRobot, SavedWarbase, SAVE_VERSION},
};

// ── Ресурсы ───────────────────────────────────────────────────────────────────

/// Данные, ожидающие загрузки.
#[derive(Resource, Default)]
pub struct PendingLoad(pub Option<SaveData>);

/// Последний день, для которого выполнено автосохранение.
#[derive(Resource, Default)]
pub struct LastAutoSaveDay(pub u32);

// ── События (Observer-based) ──────────────────────────────────────────────────

/// Сохранить в слот (0..SAVE_SLOT_COUNT).
#[derive(Event, Debug)]
pub struct TriggerSave {
    pub slot: usize,
}

/// Автосохранение.
#[derive(Event, Debug)]
pub struct TriggerAutosave;

/// Загрузить из слота.
#[derive(Event, Debug)]
pub struct TriggerLoad {
    pub slot: usize,
}

/// Загрузить автосохранение.
#[derive(Event, Debug)]
pub struct TriggerLoadAutosave;

// ── Сериализация ──────────────────────────────────────────────────────────────

fn collect_save_data(
    scout_q: &Query<&Transform, With<PlayerScout>>,
    robot_q: &Query<
        (&Transform, &Team, &crate::robot::components::Chassis, &WeaponSlots, &Health, &RobotCommand, Option<&Electronics>, Option<&Nuclear>),
        With<RobotMarker>,
    >,
    factory_q: &Query<(&Transform, &Team, &FactoryType, &CaptureProgress), With<Factory>>,
    warbase_q: &Query<(&Transform, &Team), With<Warbase>>,
    resources: &PlayerResources,
    game_time: &GameTime,
    ai: &AICommander,
) -> SaveData {
    let scout_pos = scout_q
        .single()
        .map(|t| [t.translation.x, t.translation.y, t.translation.z])
        .unwrap_or([32.0, 3.0, 32.0]);

    let robots: Vec<SavedRobot> = robot_q
        .iter()
        .map(|(tf, team, chassis, slots, health, cmd, elec, nuc)| {
            let weapons: Vec<_> = slots.slots.iter().flatten().map(|w| w.weapon_type).collect();
            SavedRobot {
                position: [tf.translation.x, tf.translation.y, tf.translation.z],
                team: *team,
                chassis: chassis.chassis_type,
                weapons,
                has_electronics: elec.is_some(),
                has_nuclear: nuc.is_some(),
                current_hp: health.current,
                nuclear_armed: nuc.map(|n| n.armed).unwrap_or(false),
                command: command_to_saved(cmd),
            }
        })
        .collect();

    let factories: Vec<SavedFactory> = factory_q
        .iter()
        .map(|(tf, team, ft, cp)| SavedFactory {
            position: [tf.translation.x, tf.translation.y, tf.translation.z],
            factory_type: *ft,
            team: *team,
            capture_progress: cp.progress,
            capture_required: cp.required,
        })
        .collect();

    let warbases: Vec<SavedWarbase> = warbase_q
        .iter()
        .map(|(tf, team)| SavedWarbase {
            position: [tf.translation.x, tf.translation.y, tf.translation.z],
            team: *team,
        })
        .collect();

    SaveData {
        version: SAVE_VERSION,
        game_day: game_time.game_day,
        day_elapsed: game_time.day_elapsed,
        seconds_per_day: game_time.seconds_per_day,
        resources: SavedResources {
            general: resources.get(ResourceType::General),
            chassis: resources.get(ResourceType::Chassis),
            cannon: resources.get(ResourceType::Cannon),
            missile: resources.get(ResourceType::Missile),
            phasers: resources.get(ResourceType::Phasers),
            electronics: resources.get(ResourceType::Electronics),
            nuclear: resources.get(ResourceType::Nuclear),
        },
        scout_position: scout_pos,
        robots,
        factories,
        warbases,
        ai: SavedAI {
            decision_timer: ai.decision_timer,
            build_timer: ai.build_timer,
            decision_counter: ai.decision_counter,
            robots_built: ai.robots_built,
        },
    }
}

fn command_to_saved(cmd: &RobotCommand) -> SavedCommand {
    match cmd {
        RobotCommand::Idle => SavedCommand::Idle,
        RobotCommand::MoveTo(v) => SavedCommand::MoveTo([v.x, v.y, v.z]),
        RobotCommand::SeekAndDestroy(_) => SavedCommand::SeekAndDestroy,
        RobotCommand::SeekAndCapture(_) => SavedCommand::SeekAndCapture,
        RobotCommand::DestroyEnemyBase(_) => SavedCommand::DestroyEnemyBase,
        RobotCommand::Defend(v) => SavedCommand::Defend([v.x, v.y, v.z]),
        RobotCommand::Patrol(pts) => {
            SavedCommand::Patrol(pts.iter().map(|v| [v.x, v.y, v.z]).collect())
        }
    }
}

fn saved_to_command(cmd: &SavedCommand) -> RobotCommand {
    match cmd {
        SavedCommand::Idle => RobotCommand::Idle,
        SavedCommand::MoveTo(v) => RobotCommand::MoveTo(Vec3::from(*v)),
        SavedCommand::SeekAndDestroy => RobotCommand::SeekAndDestroy(None),
        SavedCommand::SeekAndCapture => RobotCommand::SeekAndCapture(None),
        SavedCommand::DestroyEnemyBase => RobotCommand::DestroyEnemyBase(None),
        SavedCommand::Defend(v) => RobotCommand::Defend(Vec3::from(*v)),
        SavedCommand::Patrol(pts) => {
            RobotCommand::Patrol(pts.iter().map(|v| Vec3::from(*v)).collect())
        }
    }
}

// ── Observers: сохранение ─────────────────────────────────────────────────────

pub fn on_trigger_save(
    trigger: On<TriggerSave>,
    scout_q: Query<&Transform, With<PlayerScout>>,
    robot_q: Query<
        (&Transform, &Team, &crate::robot::components::Chassis, &WeaponSlots, &Health, &RobotCommand, Option<&Electronics>, Option<&Nuclear>),
        With<RobotMarker>,
    >,
    factory_q: Query<(&Transform, &Team, &FactoryType, &CaptureProgress), With<Factory>>,
    warbase_q: Query<(&Transform, &Team), With<Warbase>>,
    resources: Res<PlayerResources>,
    game_time: Res<GameTime>,
    ai: Res<AICommander>,
) {
    let data = collect_save_data(
        &scout_q, &robot_q, &factory_q, &warbase_q,
        &resources, &game_time, &ai,
    );
    let slot = trigger.event().slot;
    let path = slot_path(slot);
    match write_save(&path, &data) {
        Ok(()) => info!("Сохранено в слот {} (день {})", slot, data.game_day),
        Err(e) => error!("Ошибка сохранения в слот {}: {e}", slot),
    }
}

pub fn on_trigger_autosave(
    _trigger: On<TriggerAutosave>,
    scout_q: Query<&Transform, With<PlayerScout>>,
    robot_q: Query<
        (&Transform, &Team, &crate::robot::components::Chassis, &WeaponSlots, &Health, &RobotCommand, Option<&Electronics>, Option<&Nuclear>),
        With<RobotMarker>,
    >,
    factory_q: Query<(&Transform, &Team, &FactoryType, &CaptureProgress), With<Factory>>,
    warbase_q: Query<(&Transform, &Team), With<Warbase>>,
    resources: Res<PlayerResources>,
    game_time: Res<GameTime>,
    ai: Res<AICommander>,
) {
    let data = collect_save_data(
        &scout_q, &robot_q, &factory_q, &warbase_q,
        &resources, &game_time, &ai,
    );
    match write_save(std::path::Path::new(AUTOSAVE_FILE), &data) {
        Ok(()) => info!("Автосохранение (день {})", data.game_day),
        Err(e) => error!("Ошибка автосохранения: {e}"),
    }
}

// ── Observers: загрузка ───────────────────────────────────────────────────────

pub fn on_trigger_load(
    trigger: On<TriggerLoad>,
    mut pending: ResMut<PendingLoad>,
) {
    let slot = trigger.event().slot;
    if !slot_exists(slot) {
        warn!("Слот {} не найден", slot);
        return;
    }
    match read_save(&slot_path(slot)) {
        Ok(data) => {
            info!("Подготовка загрузки из слота {} (день {})", slot, data.game_day);
            pending.0 = Some(data);
        }
        Err(e) => error!("Ошибка загрузки слота {}: {e}", slot),
    }
}

pub fn on_trigger_load_autosave(
    _trigger: On<TriggerLoadAutosave>,
    mut pending: ResMut<PendingLoad>,
) {
    if !autosave_exists() {
        warn!("Автосохранение не найдено");
        return;
    }
    match read_save(std::path::Path::new(AUTOSAVE_FILE)) {
        Ok(data) => {
            info!("Подготовка загрузки автосохранения (день {})", data.game_day);
            pending.0 = Some(data);
        }
        Err(e) => error!("Ошибка чтения автосохранения: {e}"),
    }
}

// ── Система: автосохранение по дням ──────────────────────────────────────────

pub fn check_autosave(
    game_time: Res<GameTime>,
    mut last_day: ResMut<LastAutoSaveDay>,
    mut commands: Commands,
) {
    if game_time.game_day > last_day.0 {
        last_day.0 = game_time.game_day;
        if game_time.game_day > 0 {
            commands.trigger(TriggerAutosave);
        }
    }
}

// ── Система: применение загруженных данных ────────────────────────────────────

/// Применяет PendingLoad: сбрасывает мир и восстанавливает из SaveData.
#[allow(clippy::too_many_arguments)]
pub fn apply_pending_load(
    mut pending: ResMut<PendingLoad>,
    mut commands: Commands,
    robots_q: Query<Entity, With<RobotMarker>>,
    mut scout_q: Query<&mut Transform, With<PlayerScout>>,
    mut structures_q: Query<
        (Entity, &mut Team, &MeshMaterial3d<StandardMaterial>, Option<&FactoryType>, Option<&mut CaptureProgress>),
        Or<(With<Factory>, With<Warbase>)>,
    >,
    mut resources: ResMut<PlayerResources>,
    mut game_time: ResMut<GameTime>,
    mut ai: ResMut<AICommander>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    registry: Res<ModuleRegistry>,
    map: Res<MapGrid>,
) {
    let Some(data) = pending.0.take() else {
        return;
    };

    // 1. Удалить все роботы
    for entity in &robots_q {
        commands.entity(entity).despawn();
    }

    // 2. Обновить структуры по позиции через MapGrid
    for saved_factory in &data.factories {
        let pos = Vec3::from(saved_factory.position);
        if let Some((gx, gy)) = map.world_to_grid(pos) {
            if let Some(crate::map::grid::CellType::Structure(entity)) = map.get(gx, gy) {
                if let Ok((_, mut team, mat_handle, _, cp_opt)) = structures_q.get_mut(entity) {
                    *team = saved_factory.team;
                    if let Some(mat) = materials.get_mut(mat_handle.id()) {
                        mat.base_color = team_color_core(saved_factory.team);
                    }
                    if let Some(mut cp) = cp_opt {
                        cp.progress = saved_factory.capture_progress;
                        cp.required = saved_factory.capture_required;
                    }
                }
            }
        }
    }

    for saved_warbase in &data.warbases {
        let pos = Vec3::from(saved_warbase.position);
        if let Some((gx, gy)) = map.world_to_grid(pos) {
            if let Some(crate::map::grid::CellType::Structure(entity)) = map.get(gx, gy) {
                if let Ok((_, mut team, mat_handle, _, _)) = structures_q.get_mut(entity) {
                    *team = saved_warbase.team;
                    let color = team_color_core(saved_warbase.team);
                    if let Some(mat) = materials.get_mut(mat_handle.id()) {
                        mat.base_color = color;
                        mat.emissive = LinearRgba::from(color) * 0.3;
                    }
                }
            }
        }
    }

    // 3. Ресурсы игрока
    resources.stocks.insert(ResourceType::General,     data.resources.general);
    resources.stocks.insert(ResourceType::Chassis,     data.resources.chassis);
    resources.stocks.insert(ResourceType::Cannon,      data.resources.cannon);
    resources.stocks.insert(ResourceType::Missile,     data.resources.missile);
    resources.stocks.insert(ResourceType::Phasers,     data.resources.phasers);
    resources.stocks.insert(ResourceType::Electronics, data.resources.electronics);
    resources.stocks.insert(ResourceType::Nuclear,     data.resources.nuclear);

    // 4. Игровое время
    game_time.game_day        = data.game_day;
    game_time.day_elapsed     = data.day_elapsed;
    game_time.seconds_per_day = data.seconds_per_day;

    // 5. Состояние ИИ
    ai.decision_timer   = data.ai.decision_timer;
    ai.build_timer      = data.ai.build_timer;
    ai.decision_counter = data.ai.decision_counter;
    ai.robots_built     = data.ai.robots_built;

    // 6. Позиция скаута
    if let Ok(mut scout_tf) = scout_q.single_mut() {
        scout_tf.translation = Vec3::from(data.scout_position);
    }

    // 7. Заспавнить роботов
    for saved in &data.robots {
        let blueprint = RobotBlueprint {
            chassis: saved.chassis,
            weapons: saved.weapons.clone(),
            has_electronics: saved.has_electronics,
            has_nuclear: saved.has_nuclear,
        };
        let pos = Vec3::from(saved.position);
        if let Some(entity) = spawn_robot(
            &mut commands, &mut meshes, &mut materials,
            &blueprint, &registry, saved.team, pos,
        ) {
            // Восстановить HP
            let chassis_def = registry.chassis(saved.chassis);
            let slots_weight: f32 = saved.weapons.iter().map(|wt| match wt {
                WeaponType::Cannon  => 10.0,
                WeaponType::Missile => 25.0,
                WeaponType::Phasers => 30.0,
            }).sum();
            let max_hp = chassis_def.map(|c| c.base_hp + slots_weight * 2.0).unwrap_or(50.0);
            if (saved.current_hp - max_hp).abs() > 0.01 {
                commands.entity(entity).insert(Health {
                    current: saved.current_hp.max(0.0),
                    max: max_hp,
                });
            }

            // Восстановить приказ
            let robot_cmd = saved_to_command(&saved.command);
            if !matches!(robot_cmd, RobotCommand::Idle) {
                commands.entity(entity).insert(robot_cmd);
            }

            // Восстановить nuclear armed
            if saved.has_nuclear && saved.nuclear_armed {
                let nuc_def = &registry.nuclear;
                commands.entity(entity).insert(Nuclear {
                    blast_radius: nuc_def.blast_radius,
                    detonation_delay: nuc_def.detonation_delay,
                    armed: true,
                });
            }
        }
    }

    info!("Загрузка завершена: день {}, {} роботов", data.game_day, data.robots.len());
}

// ── Новая игра ────────────────────────────────────────────────────────────────

/// Сброс игрового мира до начального состояния (Новая игра).
#[derive(Event, Debug)]
pub struct TriggerNewGame;

#[allow(clippy::too_many_arguments)]
pub fn on_trigger_new_game(
    _trigger: On<TriggerNewGame>,
    mut commands: Commands,
    robots_q: Query<Entity, With<RobotMarker>>,
    mut scout_q: Query<(&mut Transform, &mut crate::player::components::ScoutMovement), With<PlayerScout>>,
    mut structures_q: Query<
        (Entity, &mut Team, &MeshMaterial3d<StandardMaterial>, Option<&mut CaptureProgress>),
        Or<(With<Factory>, With<Warbase>)>,
    >,
    mut resources: ResMut<PlayerResources>,
    mut enemy_resources: ResMut<EnemyResources>,
    mut game_time: ResMut<GameTime>,
    mut ai: ResMut<AICommander>,
    mut game_result: ResMut<GameResult>,
    mut selection: ResMut<SelectionState>,
    mut cmd_ui: ResMut<CommandUiState>,
    mut last_autosave: ResMut<LastAutoSaveDay>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<MapGrid>,
    map_structures: Res<MapStructures>,
    spawn_points: Res<MapSpawnPoints>,
) {
    // 1. Удалить всех роботов
    for entity in &robots_q {
        commands.entity(entity).despawn();
    }

    // 2. Сбросить структуры до начальных команд из карты
    for factory_def in &map_structures.factories {
        let world_pos = map.grid_to_world(factory_def.x, factory_def.y);
        if let Some((gx, gy)) = map.world_to_grid(world_pos) {
            if let Some(crate::map::grid::CellType::Structure(entity)) = map.get(gx, gy) {
                if let Ok((_, mut team, mat_handle, cp_opt)) = structures_q.get_mut(entity) {
                    let new_team = teamdef_to_team(&factory_def.team);
                    *team = new_team;
                    if let Some(mat) = materials.get_mut(mat_handle.id()) {
                        mat.base_color = team_color_core(new_team);
                    }
                    if let Some(mut cp) = cp_opt {
                        cp.progress = 0.0;
                    }
                }
            }
        }
    }

    for warbase_def in &map_structures.warbases {
        let world_pos = map.grid_to_world(warbase_def.x, warbase_def.y);
        if let Some((gx, gy)) = map.world_to_grid(world_pos) {
            if let Some(crate::map::grid::CellType::Structure(entity)) = map.get(gx, gy) {
                if let Ok((_, mut team, mat_handle, _)) = structures_q.get_mut(entity) {
                    let new_team = teamdef_to_team(&warbase_def.team);
                    *team = new_team;
                    let color = team_color_core(new_team);
                    if let Some(mat) = materials.get_mut(mat_handle.id()) {
                        mat.base_color = color;
                        mat.emissive = LinearRgba::from(color) * 0.3;
                    }
                }
            }
        }
    }

    // 3. Ресурсы игрока и ИИ
    *resources = PlayerResources::with_starting_values();
    *enemy_resources = EnemyResources(PlayerResources::with_starting_values());

    // 4. Игровое время
    *game_time = GameTime::default();

    // 5. ИИ (сбросить счётчики, сохранить конфиг)
    ai.decision_timer   = 0.0;
    ai.build_timer      = 0.0;
    ai.decision_counter = 0;
    ai.robots_built     = 0;

    // 6. Результат игры
    *game_result = GameResult::default();

    // 7. UI-состояние
    selection.selected.clear();
    cmd_ui.patrol_points.clear();
    cmd_ui.show_patrol_hint = false;

    // 8. Автосохранение
    last_autosave.0 = 0;

    // 9. Позиция скаута
    let (sx, sy) = spawn_points.player_spawn;
    let spawn_world = map.grid_to_world(sx, sy);
    if let Ok((mut tf, movement)) = scout_q.single_mut() {
        tf.translation = spawn_world.with_y(movement.altitude);
    }

    info!("Новая игра начата");
}

fn teamdef_to_team(def: &TeamDef) -> Team {
    match def {
        TeamDef::Player  => Team::Player,
        TeamDef::Enemy   => Team::Enemy,
        TeamDef::Neutral => Team::Neutral,
    }
}

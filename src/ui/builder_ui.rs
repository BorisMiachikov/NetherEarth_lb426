use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    core::{events::ResourceChanged, Team},
    economy::resource::{PlayerResources, ResourceType},
    localization::Localization,
    player::components::PlayerScout,
    robot::{
        builder::RobotBlueprint,
        components::{ChassisType, WeaponType},
        registry::ModuleRegistry,
    },
    structure::warbase::{ProductionQueue, Warbase},
};

/// Состояние UI строительства роботов.
#[derive(Resource)]
pub struct BuilderUiState {
    pub open: bool,
    pub warbase_entity: Option<Entity>,
    pub chassis: ChassisType,
    pub weapons: [Option<WeaponType>; 3],
    pub has_electronics: bool,
    pub has_nuclear: bool,
}

impl Default for BuilderUiState {
    fn default() -> Self {
        Self {
            open: false,
            warbase_entity: None,
            chassis: ChassisType::Wheels,
            weapons: [None, None, None],
            has_electronics: false,
            has_nuclear: false,
        }
    }
}

pub const BUILDER_RANGE: f32 = 6.0;

/// Открывает/закрывает Builder UI по нажатию B рядом со своим варбейсом.
/// Также закрывает меню, если игрок отошёл от базы.
pub fn open_builder_input(
    keys: Res<ButtonInput<KeyCode>>,
    scout: Query<&Transform, With<PlayerScout>>,
    warbases: Query<(Entity, &Transform, &Team), With<Warbase>>,
    mut state: ResMut<BuilderUiState>,
) {
    let Ok(scout_tf) = scout.single() else {
        return;
    };

    // Ближайший варбейс игрока
    let nearby = warbases
        .iter()
        .filter(|(_, _, t)| **t == Team::Player)
        .find(|(_, tf, _)| tf.translation.xz().distance(scout_tf.translation.xz()) < BUILDER_RANGE);

    // Если меню открыто — проверяем, не вышел ли игрок из зоны
    if state.open {
        let still_near = nearby.map_or(false, |(e, _, _)| Some(e) == state.warbase_entity);
        if !still_near {
            state.open = false;
            state.warbase_entity = None;
            return;
        }
    }

    // Открыть/закрыть по нажатию B
    if keys.just_pressed(KeyCode::KeyB) {
        if let Some((entity, _, _)) = nearby {
            state.open = !state.open;
            state.warbase_entity = if state.open { Some(entity) } else { None };
        }
    }
}

/// ЛКМ по своей варбейзе → открыть Builder UI (если скаут в радиусе).
pub fn on_warbase_click(
    click: On<Pointer<Click>>,
    warbases: Query<(&Transform, &Team), With<Warbase>>,
    scout: Query<&Transform, With<PlayerScout>>,
    mut state: ResMut<BuilderUiState>,
) {
    if click.button != PointerButton::Primary {
        return;
    }
    let entity = click.entity;
    let Ok((tf, team)) = warbases.get(entity) else {
        return;
    };
    if *team != Team::Player {
        return;
    }
    let Ok(scout_tf) = scout.single() else {
        return;
    };
    if tf.translation.xz().distance(scout_tf.translation.xz()) >= BUILDER_RANGE {
        return;
    }
    state.open = true;
    state.warbase_entity = Some(entity);
}

/// Стоимость blueprint по типам ресурсов.
fn blueprint_typed_costs(
    bp: &RobotBlueprint,
    registry: &ModuleRegistry,
) -> Vec<(ResourceType, u32)> {
    let mut costs: Vec<(ResourceType, u32)> = vec![];

    if let Some(c) = registry.chassis(bp.chassis) {
        if c.cost_chassis > 0 {
            costs.push((ResourceType::Chassis, c.cost_chassis));
        }
    }

    for wt in &bp.weapons {
        if let Some(w) = registry.weapon(*wt) {
            let rt = match wt {
                WeaponType::Cannon => ResourceType::Cannon,
                WeaponType::Missile => ResourceType::Missile,
                WeaponType::Phasers => ResourceType::Phasers,
            };
            // Суммировать если тип уже есть
            if let Some(entry) = costs.iter_mut().find(|(r, _)| *r == rt) {
                entry.1 += w.cost_resource;
            } else {
                costs.push((rt, w.cost_resource));
            }
        }
    }

    if bp.has_electronics {
        costs.push((ResourceType::Electronics, registry.electronics.cost_electronics));
    }
    if bp.has_nuclear {
        costs.push((ResourceType::Nuclear, registry.nuclear.cost_nuclear));
    }

    // General — всегда
    let build_cost = bp.cost(registry);
    if build_cost.general > 0 {
        costs.push((ResourceType::General, build_cost.general));
    }

    costs
}

fn can_afford(costs: &[(ResourceType, u32)], res: &PlayerResources) -> bool {
    costs.iter().all(|(rt, amount)| res.get(*rt) >= *amount as i32)
}

fn resource_label_key(rt: ResourceType) -> &'static str {
    match rt {
        ResourceType::General     => "ui.resource.general",
        ResourceType::Chassis     => "ui.resource.chassis",
        ResourceType::Cannon      => "ui.resource.cannon",
        ResourceType::Missile     => "ui.resource.missile",
        ResourceType::Phasers     => "ui.resource.phasers",
        ResourceType::Electronics => "ui.resource.electronics",
        ResourceType::Nuclear     => "ui.resource.nuclear",
    }
}

fn chassis_label_key(ct: ChassisType) -> &'static str {
    match ct {
        ChassisType::Wheels   => "ui.chassis.wheels",
        ChassisType::Bipod    => "ui.chassis.bipod",
        ChassisType::Tracks   => "ui.chassis.tracks",
        ChassisType::AntiGrav => "ui.chassis.antigrav",
    }
}

fn weapon_label_key(wt: Option<WeaponType>) -> &'static str {
    match wt {
        None                        => "---",
        Some(WeaponType::Cannon)    => "ui.weapon.cannon",
        Some(WeaponType::Missile)   => "ui.weapon.missile",
        Some(WeaponType::Phasers)   => "ui.weapon.phasers",
    }
}

/// Рендер окна строительства роботов.
pub fn draw_builder_ui(
    mut state: ResMut<BuilderUiState>,
    mut contexts: EguiContexts,
    registry: Res<ModuleRegistry>,
    mut player_res: ResMut<PlayerResources>,
    mut warbase_queues: Query<&mut ProductionQueue, With<Warbase>>,
    mut commands: Commands,
    loc: Res<Localization>,
) -> Result {
    if !state.open {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    let mut close = false;
    let mut do_build = false;

    egui::Window::new(loc.t("builder.title"))
        .id(egui::Id::new("builder_ui"))
        .resizable(false)
        .collapsible(false)
        .default_pos([440.0, 160.0])
        .show(ctx, |ui| {
            // ── Шасси ──────────────────────────────────────────────────────
            ui.label(egui::RichText::new(loc.t("builder.section.chassis")).strong());
            ui.horizontal(|ui| {
                for ct in [
                    ChassisType::Wheels,
                    ChassisType::Bipod,
                    ChassisType::Tracks,
                    ChassisType::AntiGrav,
                ] {
                    if ui
                        .selectable_label(state.chassis == ct, loc.t(chassis_label_key(ct)))
                        .clicked()
                    {
                        state.chassis = ct;
                    }
                }
            });

            ui.add_space(6.0);

            // ── Слоты оружия ───────────────────────────────────────────────
            ui.label(egui::RichText::new(loc.t("builder.section.weapons")).strong());
            for i in 0..3 {
                ui.horizontal(|ui| {
                    ui.label(format!("{} {}:", loc.t("builder.slot_label"), i + 1));
                    let selected_text = {
                        let key = weapon_label_key(state.weapons[i]);
                        if key == "---" { "---".to_string() } else { loc.t(key).to_string() }
                    };
                    egui::ComboBox::from_id_salt(format!("weapon_slot_{i}"))
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.weapons[i], None, "---");
                            ui.selectable_value(
                                &mut state.weapons[i],
                                Some(WeaponType::Cannon),
                                loc.t("ui.weapon.cannon"),
                            );
                            ui.selectable_value(
                                &mut state.weapons[i],
                                Some(WeaponType::Missile),
                                loc.t("ui.weapon.missile"),
                            );
                            ui.selectable_value(
                                &mut state.weapons[i],
                                Some(WeaponType::Phasers),
                                loc.t("ui.weapon.phasers"),
                            );
                        });
                });
            }

            ui.add_space(6.0);

            // ── Модули ─────────────────────────────────────────────────────
            ui.label(egui::RichText::new(loc.t("builder.section.modules")).strong());
            ui.checkbox(&mut state.has_electronics, loc.t("builder.module.electronics"));
            ui.checkbox(&mut state.has_nuclear, loc.t("builder.module.nuclear"));

            ui.add_space(6.0);
            ui.separator();

            // Собрать blueprint для расчёта стоимости
            let weapons: Vec<WeaponType> =
                state.weapons.iter().filter_map(|w| *w).collect();
            let bp = RobotBlueprint {
                chassis: state.chassis,
                weapons,
                has_electronics: state.has_electronics,
                has_nuclear: state.has_nuclear,
            };

            let validation = bp.validate();
            let costs = blueprint_typed_costs(&bp, &registry);
            let affordable = can_afford(&costs, &player_res);
            let build_cost = bp.cost(&registry);

            // ── Стоимость ──────────────────────────────────────────────────
            ui.label(egui::RichText::new(loc.t("builder.section.cost")).strong());
            egui::Grid::new("cost_grid")
                .num_columns(3)
                .spacing([12.0, 2.0])
                .show(ui, |ui| {
                    ui.label(loc.t("builder.col.resource"));
                    ui.label(loc.t("builder.col.need"));
                    ui.label(loc.t("builder.col.have"));
                    ui.end_row();

                    for (rt, needed) in &costs {
                        let available = player_res.get(*rt);
                        let enough = available >= *needed as i32;
                        let color = if enough {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::RED
                        };
                        ui.label(loc.t(resource_label_key(*rt)));
                        ui.colored_label(color, format!("{}", needed));
                        ui.colored_label(color, format!("{}", available));
                        ui.end_row();
                    }
                });

            ui.label(format!(
                "{}: {:.1} {}",
                loc.t("builder.time_label"),
                build_cost.build_time,
                loc.t("builder.unit.sec"),
            ));

            ui.add_space(6.0);

            // ── Ошибки валидации ───────────────────────────────────────────
            if let Err(ref e) = validation {
                ui.colored_label(egui::Color32::YELLOW, format!("⚠ {e}"));
            }
            if !affordable && validation.is_ok() {
                ui.colored_label(egui::Color32::RED, loc.t("builder.error.insufficient"));
            }

            ui.add_space(4.0);

            // ── Кнопки ─────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                let can_build = validation.is_ok() && affordable;
                if ui
                    .add_enabled(can_build, egui::Button::new(loc.t("builder.btn.build")))
                    .clicked()
                {
                    do_build = true;
                }
                if ui.button(loc.t("builder.btn.close")).clicked() {
                    close = true;
                }
            });
        });

    // ── Обработка нажатия Build ──────────────────────────────────────────
    if do_build {
        let weapons: Vec<WeaponType> = state.weapons.iter().filter_map(|w| *w).collect();
        let bp = RobotBlueprint {
            chassis: state.chassis,
            weapons,
            has_electronics: state.has_electronics,
            has_nuclear: state.has_nuclear,
        };
        let costs = blueprint_typed_costs(&bp, &registry);
        let build_cost = bp.cost(&registry);

        // Списать ресурсы
        for (rt, amount) in &costs {
            player_res.spend(*rt, *amount as i32);
            let new_total = player_res.get(*rt);
            commands.trigger(ResourceChanged {
                team: Team::Player,
                resource_type: *rt,
                delta: -(*amount as i32),
                new_total,
            });
        }

        // Добавить в очередь варбейса
        if let Some(warbase_entity) = state.warbase_entity {
            if let Ok(mut queue) = warbase_queues.get_mut(warbase_entity) {
                queue.enqueue(bp, build_cost.build_time);
                info!(
                    "Robot queued (build time {:.1}s)",
                    build_cost.build_time
                );
            }
        }

        // Сбросить слоты оружия
        state.weapons = [None, None, None];
        state.has_electronics = false;
        state.has_nuclear = false;
    }

    if close {
        state.open = false;
    }

    Ok(())
}

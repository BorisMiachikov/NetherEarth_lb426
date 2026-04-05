use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    core::{events::ResourceChanged, Team},
    economy::resource::{PlayerResources, ResourceType},
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

/// Открывает Builder UI по нажатию B рядом со своим варбейсом.
pub fn open_builder_input(
    keys: Res<ButtonInput<KeyCode>>,
    scout: Query<&Transform, With<PlayerScout>>,
    warbases: Query<(Entity, &Transform, &Team), With<Warbase>>,
    mut state: ResMut<BuilderUiState>,
) {
    if !keys.just_pressed(KeyCode::KeyB) {
        return;
    }
    let Ok(scout_tf) = scout.single() else {
        return;
    };

    const BUILDER_RANGE: f32 = 6.0;

    let nearby = warbases
        .iter()
        .filter(|(_, _, t)| **t == Team::Player)
        .find(|(_, tf, _)| {
            tf.translation
                .xz()
                .distance(scout_tf.translation.xz())
                < BUILDER_RANGE
        });

    if nearby.is_some() {
        state.open = !state.open;
        if state.open {
            state.warbase_entity = nearby.map(|(e, _, _)| e);
        }
    } else if state.open {
        state.open = false;
    }
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

fn resource_label(rt: ResourceType) -> &'static str {
    match rt {
        ResourceType::General => "Общий",
        ResourceType::Chassis => "Шасси",
        ResourceType::Cannon => "Пушки",
        ResourceType::Missile => "Ракеты",
        ResourceType::Phasers => "Фазеры",
        ResourceType::Electronics => "Электроника",
        ResourceType::Nuclear => "Ядерный",
    }
}

fn chassis_label(ct: ChassisType) -> &'static str {
    match ct {
        ChassisType::Wheels => "Колёса",
        ChassisType::Bipod => "Бипод",
        ChassisType::Tracks => "Гусеницы",
        ChassisType::AntiGrav => "АнтиГрав",
    }
}

fn weapon_label(wt: Option<WeaponType>) -> &'static str {
    match wt {
        None => "---",
        Some(WeaponType::Cannon) => "Пушка",
        Some(WeaponType::Missile) => "Ракета",
        Some(WeaponType::Phasers) => "Фазеры",
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
) -> Result {
    if !state.open {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    let mut close = false;
    let mut do_build = false;

    egui::Window::new("Строительство робота")
        .id(egui::Id::new("builder_ui"))
        .resizable(false)
        .collapsible(false)
        .default_pos([440.0, 160.0])
        .show(ctx, |ui| {
            // ── Шасси ──────────────────────────────────────────────────────
            ui.label(egui::RichText::new("Шасси").strong());
            ui.horizontal(|ui| {
                for ct in [
                    ChassisType::Wheels,
                    ChassisType::Bipod,
                    ChassisType::Tracks,
                    ChassisType::AntiGrav,
                ] {
                    if ui
                        .selectable_label(state.chassis == ct, chassis_label(ct))
                        .clicked()
                    {
                        state.chassis = ct;
                    }
                }
            });

            ui.add_space(6.0);

            // ── Слоты оружия ───────────────────────────────────────────────
            ui.label(egui::RichText::new("Оружие (до 3 слотов)").strong());
            for i in 0..3 {
                ui.horizontal(|ui| {
                    ui.label(format!("Слот {}:", i + 1));
                    egui::ComboBox::from_id_salt(format!("weapon_slot_{i}"))
                        .selected_text(weapon_label(state.weapons[i]))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.weapons[i], None, "---");
                            ui.selectable_value(
                                &mut state.weapons[i],
                                Some(WeaponType::Cannon),
                                "Пушка",
                            );
                            ui.selectable_value(
                                &mut state.weapons[i],
                                Some(WeaponType::Missile),
                                "Ракета",
                            );
                            ui.selectable_value(
                                &mut state.weapons[i],
                                Some(WeaponType::Phasers),
                                "Фазеры",
                            );
                        });
                });
            }

            ui.add_space(6.0);

            // ── Модули ─────────────────────────────────────────────────────
            ui.label(egui::RichText::new("Модули").strong());
            ui.checkbox(&mut state.has_electronics, "Электроника (+точность, +скор. огня, -время захвата)");
            ui.checkbox(&mut state.has_nuclear, "Ядерный заряд (уничтожает всё в R=8)");

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
            ui.label(egui::RichText::new("Стоимость").strong());
            egui::Grid::new("cost_grid")
                .num_columns(3)
                .spacing([12.0, 2.0])
                .show(ui, |ui| {
                    ui.label("Ресурс");
                    ui.label("Нужно");
                    ui.label("Есть");
                    ui.end_row();

                    for (rt, needed) in &costs {
                        let available = player_res.get(*rt);
                        let enough = available >= *needed as i32;
                        let color = if enough {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::RED
                        };
                        ui.label(resource_label(*rt));
                        ui.colored_label(color, format!("{}", needed));
                        ui.colored_label(color, format!("{}", available));
                        ui.end_row();
                    }
                });

            ui.label(format!(
                "Время постройки: {:.1} с",
                build_cost.build_time
            ));

            ui.add_space(6.0);

            // ── Ошибки валидации ───────────────────────────────────────────
            if let Err(ref e) = validation {
                ui.colored_label(egui::Color32::YELLOW, format!("⚠ {e}"));
            }
            if !affordable && validation.is_ok() {
                ui.colored_label(egui::Color32::RED, "Недостаточно ресурсов");
            }

            ui.add_space(4.0);

            // ── Кнопки ─────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                let can_build = validation.is_ok() && affordable;
                if ui
                    .add_enabled(can_build, egui::Button::new("Построить"))
                    .clicked()
                {
                    do_build = true;
                }
                if ui.button("Закрыть").clicked() {
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
                    "Робот добавлен в очередь (постройка {:.1}с)",
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

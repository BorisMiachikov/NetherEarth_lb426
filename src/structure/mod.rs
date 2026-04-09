pub mod capture;
pub mod factory;
pub mod warbase;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::{
    core::Team,
    map::{
        grid::{CellType, MapGrid},
        loader::{FactoryTypeDef, MapStructures, TeamDef},
    },
};

use capture::{
    draw_capture_progress, on_structure_captured, seek_capture_navigation,
    update_capture_progress, CaptureProgress, Capturable,
};
use factory::{Factory, FactoryType, ProductionRate};
use warbase::{draw_production_progress, tick_production_queue, ProductionQueue, Warbase};

pub use factory::FactoryType as FactType;

pub struct StructurePlugin;

impl Plugin for StructurePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_structures.after(crate::map::spawn_ground))
            .add_observer(on_structure_captured)
            .add_systems(
                FixedUpdate,
                (
                    seek_capture_navigation,
                    update_capture_progress.after(seek_capture_navigation),
                    tick_production_queue,
                ),
            )
            // Не-egui визуалы — в Update
            .add_systems(
                Update,
                (draw_capture_progress, draw_production_progress),
            )
            // Тултип типа/владельца структуры — всегда активен
            .add_systems(EguiPrimaryContextPass, structure_tooltip);
    }
}

fn team_color(team: &TeamDef) -> Color {
    match team {
        TeamDef::Player => Color::srgb(0.15, 0.75, 0.2),
        TeamDef::Enemy => Color::srgb(0.8, 0.15, 0.15),
        TeamDef::Neutral => Color::srgb(0.55, 0.55, 0.55),
    }
}

fn team_to_core(team: &TeamDef) -> Team {
    match team {
        TeamDef::Player => Team::Player,
        TeamDef::Enemy => Team::Enemy,
        TeamDef::Neutral => Team::Neutral,
    }
}

fn factory_type(def: &FactoryTypeDef) -> FactoryType {
    match def {
        FactoryTypeDef::Chassis => FactoryType::Chassis,
        FactoryTypeDef::Cannon => FactoryType::Cannon,
        FactoryTypeDef::Missile => FactoryType::Missile,
        FactoryTypeDef::Phasers => FactoryType::Phasers,
        FactoryTypeDef::Electronics => FactoryType::Electronics,
        FactoryTypeDef::Nuclear => FactoryType::Nuclear,
        FactoryTypeDef::General => FactoryType::Chassis, // устаревший тип → Chassis
    }
}

pub fn spawn_structures(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut map: ResMut<MapGrid>,
    structures: Res<MapStructures>,
) {
    // --- Фабрики ---
    let factory_mesh = meshes.add(Cuboid::new(1.4, 1.0, 1.4));

    for def in &structures.factories {
        let color = team_color(&def.team);
        let mat = materials.add(StandardMaterial {
            base_color: color,
            ..default()
        });
        let world_pos = map.grid_to_world(def.x, def.y);
        let ft = factory_type(&def.factory_type);

        let entity = commands
            .spawn((
                Name::new(format!("Factory {:?}", ft)),
                Factory,
                ft,
                ProductionRate::default(),
                Capturable,
                CaptureProgress::new(1.0),
                team_to_core(&def.team),
                crate::core::Health::new(200.0),
                Mesh3d(factory_mesh.clone()),
                MeshMaterial3d(mat),
                Transform::from_translation(world_pos.with_y(0.5)),
            ))
            .id();

        map.set(def.x, def.y, CellType::Structure(entity));
    }

    // --- Варбейсы ---
    let warbase_mesh = meshes.add(Cuboid::new(2.0, 2.0, 2.0));

    for def in &structures.warbases {
        let color = team_color(&def.team);
        let mat = materials.add(StandardMaterial {
            base_color: color,
            emissive: LinearRgba::from(color) * 0.3,
            ..default()
        });
        let world_pos = map.grid_to_world(def.x, def.y);

        let entity = commands
            .spawn((
                Name::new(format!("Warbase {:?}", def.team)),
                Warbase,
                ProductionQueue::default(),
                team_to_core(&def.team),
                crate::core::Health::new(9999.0),
                Mesh3d(warbase_mesh.clone()),
                MeshMaterial3d(mat),
                Transform::from_translation(world_pos.with_y(1.0)),
            ))
            .id();

        map.set(def.x, def.y, CellType::Structure(entity));
    }
}

/// Tooltip: при приближении скаута к структуре показывает тип, владельца и производство.
pub fn structure_tooltip(
    scout: Query<&Transform, With<crate::player::components::PlayerScout>>,
    factories: Query<(&Transform, &FactoryType, &Team), With<Factory>>,
    warbases: Query<(&Transform, &Team), With<Warbase>>,
    mut contexts: bevy_egui::EguiContexts,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let Ok(scout_tf) = scout.single() else {
        return Ok(());
    };

    const TOOLTIP_DIST: f32 = 3.5;

    struct StructInfo {
        title: String,
        owner: Team,
        production: Option<(String, String)>, // (специфический ресурс, количество)
    }

    let mut found: Option<StructInfo> = None;

    for (tf, ft, team) in &factories {
        if scout_tf.translation.xz().distance(tf.translation.xz()) < TOOLTIP_DIST {
            let prod = match ft {
                FactoryType::Chassis     => ("Шасси".to_string(),       "+5 Шасси, +2 Общий/день".to_string()),
                FactoryType::Cannon      => ("Пушки".to_string(),       "+5 Пушки, +2 Общий/день".to_string()),
                FactoryType::Missile     => ("Ракеты".to_string(),      "+5 Ракеты, +2 Общий/день".to_string()),
                FactoryType::Phasers     => ("Фазеры".to_string(),      "+5 Фазеры, +2 Общий/день".to_string()),
                FactoryType::Electronics => ("Электроника".to_string(), "+5 Электроника, +2 Общий/день".to_string()),
                FactoryType::Nuclear     => ("Ядерный".to_string(),     "+5 Ядерный, +2 Общий/день".to_string()),
            };
            found = Some(StructInfo {
                title: format!("Фабрика: {ft}"),
                owner: *team,
                production: Some(prod),
            });
            break;
        }
    }
    if found.is_none() {
        for (tf, team) in &warbases {
            if scout_tf.translation.xz().distance(tf.translation.xz()) < TOOLTIP_DIST {
                found = Some(StructInfo {
                    title: "Главная база".to_string(),
                    owner: *team,
                    production: Some(("Общий".to_string(), "+5 Общий/день".to_string())),
                });
                break;
            }
        }
    }

    if let Some(info) = found {
        let owner_color = match info.owner {
            Team::Player  => bevy_egui::egui::Color32::from_rgb(60, 200, 100),
            Team::Enemy   => bevy_egui::egui::Color32::from_rgb(220, 80, 80),
            Team::Neutral => bevy_egui::egui::Color32::GRAY,
        };
        let owner_label = match info.owner {
            Team::Player  => "Игрок",
            Team::Enemy   => "Враг",
            Team::Neutral => "Нейтрал",
        };

        bevy_egui::egui::Window::new(&info.title)
            .id(bevy_egui::egui::Id::new("structure_tooltip"))
            .anchor(bevy_egui::egui::Align2::RIGHT_TOP, bevy_egui::egui::vec2(-10.0, 10.0))
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.colored_label(owner_color, owner_label);
                if let Some((_, prod_text)) = info.production {
                    ui.separator();
                    ui.label(
                        bevy_egui::egui::RichText::new(prod_text)
                            .small()
                            .color(bevy_egui::egui::Color32::from_rgb(160, 220, 160)),
                    );
                }
            });
    }

    Ok(())
}

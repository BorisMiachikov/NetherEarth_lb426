pub mod capture;
pub mod factory;
pub mod warbase;

use bevy::prelude::*;

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
use warbase::{ProductionQueue, Warbase};

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
                ),
            )
            .add_systems(Update, (structure_tooltip, draw_capture_progress));
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
        FactoryTypeDef::General => FactoryType::General,
        FactoryTypeDef::Chassis => FactoryType::Chassis,
        FactoryTypeDef::Cannon => FactoryType::Cannon,
        FactoryTypeDef::Missile => FactoryType::Missile,
        FactoryTypeDef::Phasers => FactoryType::Phasers,
        FactoryTypeDef::Electronics => FactoryType::Electronics,
        FactoryTypeDef::Nuclear => FactoryType::Nuclear,
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
                CaptureProgress::new(12.0),
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

/// Tooltip: при приближении скаута к структуре показывает egui-окошко.
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

    const TOOLTIP_DIST: f32 = 3.0;
    let mut info: Option<String> = None;

    for (tf, ft, team) in &factories {
        if scout_tf.translation.xz().distance(tf.translation.xz()) < TOOLTIP_DIST {
            info = Some(format!("Фабрика: {ft}\nВладелец: {team:?}"));
            break;
        }
    }
    if info.is_none() {
        for (tf, team) in &warbases {
            if scout_tf.translation.xz().distance(tf.translation.xz()) < TOOLTIP_DIST {
                info = Some(format!("Warbase\nВладелец: {team:?}"));
                break;
            }
        }
    }

    if let Some(text) = info {
        bevy_egui::egui::Window::new("Структура")
            .id(bevy_egui::egui::Id::new("structure_tooltip"))
            .default_pos([200.0, 10.0])
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(text);
            });
    }

    Ok(())
}

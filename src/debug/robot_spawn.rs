use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{
    core::Team,
    map::grid::MapGrid,
    movement::steering::CurrentPath,
    player::components::PlayerScout,
    robot::{
        builder::RobotBlueprint,
        bundle::spawn_robot,
        components::{ChassisType, WeaponType},
        registry::ModuleRegistry,
    },
};

#[derive(Resource, Default)]
pub struct RobotSpawnUi {
    pub chassis: ChassisType,
    pub weapon: WeaponType,
    pub has_electronics: bool,
    pub has_nuclear: bool,
}

impl Default for ChassisType {
    fn default() -> Self {
        ChassisType::Wheels
    }
}

impl Default for WeaponType {
    fn default() -> Self {
        WeaponType::Cannon
    }
}

/// egui-панель спавна роботов (запускается в EguiPrimaryContextPass).
pub fn robot_spawn_panel(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<RobotSpawnUi>,
    scout: Query<&Transform, With<PlayerScout>>,
    registry: Res<ModuleRegistry>,
    map: Res<MapGrid>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::Window::new("Спавн робота")
        .default_pos([10.0, 120.0])
        .resizable(false)
        .show(ctx, |ui| {
            // Шасси
            ui.label("Шасси:");
            for ct in [ChassisType::Wheels, ChassisType::Bipod, ChassisType::Tracks, ChassisType::AntiGrav] {
                ui.radio_value(&mut ui_state.chassis, ct, format!("{ct}"));
            }

            ui.separator();
            ui.label("Оружие:");
            for wt in [WeaponType::Cannon, WeaponType::Missile, WeaponType::Phasers] {
                ui.radio_value(&mut ui_state.weapon, wt, format!("{wt:?}"));
            }

            ui.separator();
            ui.checkbox(&mut ui_state.has_electronics, "Электроника");
            ui.checkbox(&mut ui_state.has_nuclear, "Ядерный заряд");

            ui.separator();
            if ui.button("Спавн (Player)").clicked() {
                let pos = scout.single().map(|t| t.translation).unwrap_or(Vec3::new(8.0, 0.0, 8.0));
                let bp = RobotBlueprint::new(ui_state.chassis)
                    .with_weapon(ui_state.weapon);
                let bp = if ui_state.has_electronics { bp.with_electronics() } else { bp };
                let bp = if ui_state.has_nuclear { bp.with_nuclear() } else { bp };
                spawn_robot(&mut commands, &mut meshes, &mut materials, &bp, &registry, Team::Player, pos, &map);
            }
            if ui.button("Спавн (Enemy)").clicked() {
                let pos = scout.single().map(|t| t.translation).unwrap_or(Vec3::new(8.0, 0.0, 8.0));
                let bp = RobotBlueprint::new(ui_state.chassis).with_weapon(ui_state.weapon);
                spawn_robot(&mut commands, &mut meshes, &mut materials, &bp, &registry, Team::Enemy, pos, &map);
            }
        });

    Ok(())
}

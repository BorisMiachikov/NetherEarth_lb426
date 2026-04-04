use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use super::state::AppState;
use crate::{
    audio::AudioPlugin,
    camera::CameraPlugin,
    combat::CombatPlugin,
    command::CommandPlugin,
    core::CorePlugin,
    debug::DebugPlugin,
    economy::EconomyPlugin,
    map::MapPlugin,
    movement::MovementPlugin,
    player::PlayerPlugin,
    robot::RobotPlugin,
    save::SavePlugin,
    structure::StructurePlugin,
    ui::UiPlugin,
};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Nether Earth".into(),
                    resolution: (1280_u32, 720_u32).into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(MeshPickingPlugin)
        .init_state::<AppState>()
        .add_plugins((
            CorePlugin,
            MapPlugin,
            CameraPlugin,
            PlayerPlugin,
            RobotPlugin,
            CommandPlugin,
            MovementPlugin,
            CombatPlugin,
            EconomyPlugin,
            StructurePlugin,
            UiPlugin,
            AudioPlugin,
            SavePlugin,
            DebugPlugin,
        ));
    }
}

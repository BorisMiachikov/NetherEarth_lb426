use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use super::state::AppState;
use crate::{
    ai::AiPlugin,
    audio::AudioPlugin,
    camera::CameraPlugin,
    combat::CombatPlugin,
    command::CommandPlugin,
    core::CorePlugin,
    economy::EconomyPlugin,
    editor::EditorPlugin,
    localization::LocalizationPlugin,
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
            LocalizationPlugin,
            CorePlugin,
            MapPlugin,
            CameraPlugin,
            PlayerPlugin,
            RobotPlugin,
            CommandPlugin,
            MovementPlugin,
        ))
        .add_plugins((
            CombatPlugin,
            EconomyPlugin,
            StructurePlugin,
            AiPlugin,
            UiPlugin,
            AudioPlugin,
            SavePlugin,
            EditorPlugin,
        ));

        #[cfg(feature = "debug_tools")]
        app.add_plugins(crate::debug::DebugPlugin);
    }
}

use std::time::SystemTime;

use bevy::prelude::*;

use crate::{
    core::resources::{load_game_config, GameConfig},
    robot::registry::{load_module_registry, ModuleRegistry},
};

const POLL_INTERVAL_SECS: f32 = 2.0;

const CONFIG_FILES: &[&str] = &[
    "configs/game.ron",
    "configs/chassis.ron",
    "configs/weapons.ron",
    "configs/electronics.ron",
    "configs/nuclear.ron",
];

/// Хранит время последней модификации каждого конфиг-файла.
#[derive(Resource)]
pub struct HotReloadState {
    timer: f32,
    mtimes: Vec<Option<SystemTime>>,
}

impl Default for HotReloadState {
    fn default() -> Self {
        Self {
            timer: 0.0,
            mtimes: CONFIG_FILES
                .iter()
                .map(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok())
                .collect(),
        }
    }
}

/// Раз в POLL_INTERVAL_SECS проверяет mtime конфигов.
/// При изменении перезагружает GameConfig и/или ModuleRegistry без рестарта.
pub fn poll_config_hot_reload(
    time: Res<Time>,
    mut state: ResMut<HotReloadState>,
    mut game_config: ResMut<GameConfig>,
    mut registry: ResMut<ModuleRegistry>,
) {
    state.timer += time.delta_secs();
    if state.timer < POLL_INTERVAL_SECS {
        return;
    }
    state.timer = 0.0;

    let mut game_changed = false;
    let mut modules_changed = false;

    for (i, path) in CONFIG_FILES.iter().enumerate() {
        let new_mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        if new_mtime != state.mtimes[i] {
            state.mtimes[i] = new_mtime;
            if i == 0 {
                game_changed = true;
            } else {
                modules_changed = true;
            }
        }
    }

    if game_changed {
        *game_config = load_game_config();
        info!("hot-reload: GameConfig перезагружен");
    }
    if modules_changed {
        match load_module_registry() {
            Ok(new_registry) => {
                *registry = new_registry;
                info!("hot-reload: ModuleRegistry перезагружен");
            }
            Err(e) => warn!("hot-reload: не удалось перезагрузить ModuleRegistry: {e}"),
        }
    }
}

pub struct DevHotReloadPlugin;

impl Plugin for DevHotReloadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HotReloadState>()
            .add_systems(Update, poll_config_hot_reload);
        info!("DevHotReloadPlugin: мониторинг {:?}", CONFIG_FILES);
    }
}

use bevy::prelude::*;
use serde::Deserialize;

use crate::app::state::AppState;

use super::time::GameTime;

/// Конфигурация игры, загружаемая из `configs/game.ron`.
#[derive(Resource, Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct GameConfig {
    /// Секунд реального времени в одном игровом дне.
    pub seconds_per_day: f32,
    pub map_width: u32,
    pub map_height: u32,
    pub scout_speed: f32,
    pub scout_min_altitude: f32,
    pub scout_max_altitude: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            seconds_per_day: 30.0,
            map_width: 64,
            map_height: 64,
            scout_speed: 8.0,
            scout_min_altitude: 1.0,
            scout_max_altitude: 10.0,
        }
    }
}

/// Загружает `GameConfig` из RON-файла. При ошибке возвращает значения по умолчанию.
pub fn load_game_config() -> GameConfig {
    let path = "configs/game.ron";
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            warn!("Не удалось прочитать {path}: {e}. Используется GameConfig::default().");
            return GameConfig::default();
        }
    };
    match ron::from_str::<GameConfig>(&content) {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("Ошибка парсинга {path}: {e}. Используется GameConfig::default().");
            GameConfig::default()
        }
    }
}

/// Синхронизирует `GameTime.paused` с состоянием приложения.
/// При входе в `Paused` — ставит на паузу, при входе в `Playing` — снимает.
pub fn on_enter_paused(mut game_time: ResMut<GameTime>) {
    game_time.paused = true;
}

pub fn on_enter_playing(mut game_time: ResMut<GameTime>) {
    game_time.paused = false;
}

/// Регистрирует системы синхронизации паузы в приложении.
pub fn add_pause_sync_systems(app: &mut App) {
    app.add_systems(OnEnter(AppState::Paused), on_enter_paused)
        .add_systems(OnEnter(AppState::Playing), on_enter_playing);
}

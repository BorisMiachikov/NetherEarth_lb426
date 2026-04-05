use serde::Deserialize;

/// Конфигурация ИИ командира (загружается из configs/ai.ron).
#[derive(Deserialize, Debug, Clone)]
pub struct AiConfig {
    /// Интервал в секундах между решениями о назначении приказов.
    pub decision_interval: f32,
    /// Интервал в секундах между попытками построить нового робота.
    pub build_interval: f32,
    /// Агрессивность [0..1]: вероятность атаки вместо захвата при прочих равных.
    pub aggression: f32,
    /// Сколько фабрик нужно ИИ для применения ядерной стратегии.
    pub nuclear_factory_threshold: u32,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            decision_interval: 5.0,
            build_interval: 12.0,
            aggression: 0.65,
            nuclear_factory_threshold: 2,
        }
    }
}

/// Результат игры (устанавливается при победе или поражении).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOutcome {
    PlayerWin,
    PlayerLose,
}

/// Ресурс: итог игры.
#[derive(bevy::prelude::Resource, Default)]
pub struct GameResult {
    pub outcome: Option<GameOutcome>,
    pub game_days: u32,
    pub enemy_factories: u32,
    pub player_factories: u32,
}

/// Ресурс-состояние ИИ командира.
#[derive(bevy::prelude::Resource)]
pub struct AICommander {
    pub config: AiConfig,
    /// Таймер между решениями о приказах.
    pub decision_timer: f32,
    /// Таймер между постройкой роботов.
    pub build_timer: f32,
    /// Счётчик решений (для детерминированного выбора blueprint).
    pub decision_counter: u32,
    /// Счётчик построенных роботов.
    pub robots_built: u32,
}

impl AICommander {
    pub fn new(config: AiConfig) -> Self {
        Self {
            config,
            decision_timer: 0.0,
            build_timer: 0.0,
            decision_counter: 0,
            robots_built: 0,
        }
    }
}

pub fn load_ai_config() -> AiConfig {
    let content = match std::fs::read_to_string("configs/ai.ron") {
        Ok(s) => s,
        Err(e) => {
            bevy::log::warn!("Не удалось загрузить configs/ai.ron: {e}. Используются значения по умолчанию.");
            return AiConfig::default();
        }
    };
    match ron::from_str(&content) {
        Ok(cfg) => cfg,
        Err(e) => {
            bevy::log::warn!("Ошибка парсинга ai.ron: {e}. Используются значения по умолчанию.");
            AiConfig::default()
        }
    }
}

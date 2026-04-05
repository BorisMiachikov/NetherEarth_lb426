use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ── Язык ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    Russian,
    English,
}

impl Language {
    pub fn file_name(self) -> &'static str {
        match self {
            Language::Russian => "assets/locales/ru.ron",
            Language::English => "assets/locales/en.ron",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Language::Russian => "RU",
            Language::English => "EN",
        }
    }
}

// ── Ресурс ────────────────────────────────────────────────────────────────────

/// Система локализации. Хранит строки активного языка.
#[derive(Resource)]
pub struct Localization {
    pub language: Language,
    strings: HashMap<String, String>,
}

impl Localization {
    pub fn load(lang: Language) -> Self {
        let strings = Self::load_strings(lang);
        Self { language: lang, strings }
    }

    fn load_strings(lang: Language) -> HashMap<String, String> {
        match std::fs::read_to_string(lang.file_name()) {
            Ok(content) => {
                ron::from_str::<HashMap<String, String>>(&content)
                    .unwrap_or_else(|e| {
                        warn!("Ошибка парсинга локали {:?}: {e}", lang.file_name());
                        HashMap::new()
                    })
            }
            Err(e) => {
                warn!("Не удалось загрузить локаль {:?}: {e}", lang.file_name());
                HashMap::new()
            }
        }
    }

    /// Получить строку по ключу. Если не найдена — возвращает ключ.
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings.get(key).map(|s| s.as_str()).unwrap_or(key)
    }

    /// Сменить язык и перезагрузить строки.
    pub fn set_language(&mut self, lang: Language) {
        self.language = lang;
        self.strings = Self::load_strings(lang);
    }
}

// ── Событие смены языка (Observer) ────────────────────────────────────────────

/// Запрос на смену языка.
#[derive(Event)]
pub struct ChangeLanguage(pub Language);

/// Observer: обрабатывает запрос смены языка.
pub fn on_change_language(
    trigger: On<ChangeLanguage>,
    mut loc: ResMut<Localization>,
) {
    let lang = trigger.event().0;
    if loc.language != lang {
        loc.set_language(lang);
        info!("Язык изменён на {:?}", lang);
    }
}

// ── Плагин ────────────────────────────────────────────────────────────────────

pub struct LocalizationPlugin;

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Localization::load(Language::Russian))
            .add_observer(on_change_language);
    }
}

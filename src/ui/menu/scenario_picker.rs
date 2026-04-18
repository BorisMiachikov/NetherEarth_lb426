use serde::Deserialize;

use crate::economy::resource::ScenarioInitialResources;

/// Описание одного сценария (десериализуется из data/scenarios/*.ron).
#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioDef {
    pub name: String,
    pub description: String,
    pub map_path: String,
    /// Опциональные начальные ресурсы. Если не задано — используются значения по умолчанию.
    #[serde(default)]
    pub initial_resources: Option<ScenarioInitialResources>,
}

/// Список доступных сценариев и индекс выбранного.
#[derive(bevy::prelude::Resource)]
pub struct ScenarioList {
    pub scenarios: Vec<ScenarioDef>,
    pub selected: usize,
}

impl ScenarioList {
    /// Сканирует `data/scenarios/` и загружает все .ron файлы.
    pub fn load_from_dir() -> Self {
        let mut scenarios: Vec<ScenarioDef> = Vec::new();

        if let Ok(entries) = std::fs::read_dir("data/scenarios") {
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map_or(false, |ext| ext == "ron"))
                .collect();
            paths.sort();

            for path in paths {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match ron::from_str::<ScenarioDef>(&content) {
                        Ok(def) => scenarios.push(def),
                        Err(e) => bevy::prelude::warn!("Ошибка парсинга {:?}: {e}", path),
                    },
                    Err(e) => bevy::prelude::warn!("Не удалось прочитать {:?}: {e}", path),
                }
            }
        }

        if scenarios.is_empty() {
            scenarios.push(ScenarioDef {
                name: "Standard Battle".into(),
                description: "8 neutral factories.".into(),
                map_path: "data/maps/default.ron".into(),
                initial_resources: None,
            });
        }

        Self { scenarios, selected: 0 }
    }

    pub fn current(&self) -> &ScenarioDef {
        &self.scenarios[self.selected]
    }
}

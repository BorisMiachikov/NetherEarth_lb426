use bevy::prelude::*;

use crate::{
    economy::resource::ScenarioInitialResources,
    localization::Localization,
    map::loader::{CellTypeDef, FactoryDef, FactoryTypeDef, TeamDef, WarbaseDef},
};

// ---------------------------------------------------------------------------
// Начальные ресурсы сценария (UI-форма)
// ---------------------------------------------------------------------------

/// UI-представление начальных ресурсов сценария. Все поля — числа (без Option),
/// включение/выключение override'а управляется флагом в EditorState.
#[derive(Debug, Clone, Copy)]
pub struct InitialResourcesUi {
    pub general:     i32,
    pub chassis:     i32,
    pub cannon:      i32,
    pub missile:     i32,
    pub phasers:     i32,
    pub electronics: i32,
    pub nuclear:     i32,
}

impl Default for InitialResourcesUi {
    fn default() -> Self {
        // Совпадает с PlayerResources::with_starting_values().
        Self {
            general: 50, chassis: 20, cannon: 15, missile: 10,
            phasers: 10, electronics: 10, nuclear: 5,
        }
    }
}

impl InitialResourcesUi {
    /// Конвертирует UI → ScenarioInitialResources (все поля Some).
    pub fn to_scenario(self) -> ScenarioInitialResources {
        ScenarioInitialResources {
            general:     Some(self.general),
            chassis:     Some(self.chassis),
            cannon:      Some(self.cannon),
            missile:     Some(self.missile),
            phasers:     Some(self.phasers),
            electronics: Some(self.electronics),
            nuclear:     Some(self.nuclear),
        }
    }

    /// Заполняет значения из сценария; None — оставляет default.
    pub fn from_scenario(s: &ScenarioInitialResources) -> Self {
        let d = Self::default();
        Self {
            general:     s.general.unwrap_or(d.general),
            chassis:     s.chassis.unwrap_or(d.chassis),
            cannon:      s.cannon.unwrap_or(d.cannon),
            missile:     s.missile.unwrap_or(d.missile),
            phasers:     s.phasers.unwrap_or(d.phasers),
            electronics: s.electronics.unwrap_or(d.electronics),
            nuclear:     s.nuclear.unwrap_or(d.nuclear),
        }
    }
}

// ---------------------------------------------------------------------------
// Инструменты редактора
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EditorTool {
    #[default]
    TerrainBrush,
    PlaceFactory,
    PlaceWarbase,
    PlacePlayerSpawn,
    Erase,
}

/// Размер кисти в клетках (1, 3 или 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrushSize {
    #[default]
    One = 1,
    Three = 3,
    Five = 5,
}

impl BrushSize {
    pub fn radius(self) -> i32 {
        match self {
            BrushSize::One   => 0,
            BrushSize::Three => 1,
            BrushSize::Five  => 2,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            BrushSize::One   => "1×1",
            BrushSize::Three => "3×3",
            BrushSize::Five  => "5×5",
        }
    }
}

// ---------------------------------------------------------------------------
// История действий (Undo/Redo)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum EditorAction {
    CellChanged {
        x: u32,
        y: u32,
        from: CellTypeDef,
        to: CellTypeDef,
    },
    StructurePlaced {
        kind: PlacedStructureKind,
    },
    StructureRemoved {
        kind: PlacedStructureKind,
    },
    PlayerSpawnMoved {
        from: (u32, u32),
        to: (u32, u32),
    },
}

#[derive(Debug, Clone)]
pub enum PlacedStructureKind {
    Factory(FactoryDef),
    Warbase(WarbaseDef),
}

// ---------------------------------------------------------------------------
// Основное состояние редактора
// ---------------------------------------------------------------------------

/// Размер карты, который может выбрать пользователь.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MapSize {
    Small  = 32,
    #[default]
    Normal = 64,
    Large  = 96,
}

impl MapSize {
    pub fn value(self) -> u32 {
        self as u32
    }
    pub fn label(self) -> &'static str {
        match self {
            MapSize::Small  => "32×32",
            MapSize::Normal => "64×64",
            MapSize::Large  => "96×96",
        }
    }
}

#[derive(Resource, Debug)]
pub struct EditorState {
    // --- Инструменты ---
    pub current_tool: EditorTool,
    pub brush_cell_type: CellTypeDef,
    pub brush_size: BrushSize,
    /// Тип фабрики при PlaceFactory.
    pub factory_type: FactoryTypeDef,
    /// Команда при PlaceFactory/PlaceWarbase.
    pub place_team: TeamDef,

    // --- Данные карты ---
    pub map_name: String,
    pub map_description: String,
    pub map_size: MapSize,
    pub player_spawn: (u32, u32),
    pub factories: Vec<FactoryDef>,
    pub warbases: Vec<WarbaseDef>,

    // --- Файл ---
    pub file_name: String,
    pub dirty: bool,

    // --- История ---
    pub undo_stack: Vec<EditorAction>,
    pub redo_stack: Vec<EditorAction>,
    pub undo_requested: bool,
    pub redo_requested: bool,
    pub play_test_requested: bool,
    pub show_exit_dialog: bool,

    // --- Hover ---
    /// Клетка под курсором (для предпросмотра).
    pub hovered_cell: Option<(u32, u32)>,

    // --- Начальные ресурсы сценария ---
    pub initial_resources_enabled: bool,
    pub initial_resources: InitialResourcesUi,

    // --- Диалоги ---
    pub show_new_map_dialog: bool,
    pub show_open_dialog: bool,
    pub show_validation_error: Option<String>,
    /// Список файлов в data/maps/ для диалога открытия.
    pub available_maps: Vec<String>,
    pub new_map_size: MapSize,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            current_tool: EditorTool::default(),
            brush_cell_type: CellTypeDef::Rock,
            brush_size: BrushSize::default(),
            factory_type: FactoryTypeDef::Chassis,
            place_team: TeamDef::Neutral,

            map_name: "custom".into(),
            map_description: "".into(),
            map_size: MapSize::default(),
            player_spawn: (5, 5),
            factories: Vec::new(),
            warbases: Vec::new(),

            file_name: "custom".into(),
            dirty: false,

            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            undo_requested: false,
            redo_requested: false,
            play_test_requested: false,
            show_exit_dialog: false,

            hovered_cell: None,

            initial_resources_enabled: false,
            initial_resources: InitialResourcesUi::default(),

            show_new_map_dialog: false,
            show_open_dialog: false,
            show_validation_error: None,
            available_maps: Vec::new(),
            new_map_size: MapSize::default(),
        }
    }
}

impl EditorState {
    /// Сброс данных карты до пустой заданного размера.
    pub fn reset_to_empty(&mut self, size: MapSize) {
        self.map_size = size;
        self.player_spawn = (2, 2);
        self.factories.clear();
        self.warbases.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.dirty = false;
        self.show_new_map_dialog = false;
        self.initial_resources_enabled = false;
        self.initial_resources = InitialResourcesUi::default();
    }

    /// Применяет действие и кладёт его в undo-стек.
    pub fn push_action(&mut self, action: EditorAction) {
        self.undo_stack.push(action);
        self.redo_stack.clear();
        // Ограничиваем стек 50 операциями
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.dirty = true;
    }

    /// Сканирует data/maps/ и заполняет available_maps.
    pub fn refresh_map_list(&mut self) {
        self.available_maps.clear();
        if let Ok(entries) = std::fs::read_dir("data/maps") {
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map_or(false, |ext| ext == "ron"))
                .collect();
            paths.sort();
            for p in paths {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    self.available_maps.push(stem.to_owned());
                }
            }
        }
    }

    /// Валидирует карту перед сохранением. Возвращает сообщение об ошибке или None.
    pub fn validate(&self, loc: &Localization) -> Option<String> {
        let has_player_wb = self.warbases.iter().any(|w| matches!(w.team, TeamDef::Player));
        let has_enemy_wb  = self.warbases.iter().any(|w| matches!(w.team, TeamDef::Enemy));
        if !has_player_wb {
            return Some(loc.t("editor.error.no_player_wb").to_owned());
        }
        if !has_enemy_wb {
            return Some(loc.t("editor.error.no_enemy_wb").to_owned());
        }
        let size = self.map_size.value();
        let (sx, sy) = self.player_spawn;
        if sx >= size || sy >= size {
            return Some(format!("{} ({sx},{sy}): {size}×{size}", loc.t("editor.error.spawn_oob")));
        }
        None
    }
}

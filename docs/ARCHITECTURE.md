# Техническая архитектура — Nether Earth LB426

> Документ описывает архитектурные решения, паттерны и структуру кода проекта.

---

## 1. Обзор ECS-архитектуры

Игра построена на **Bevy ECS** (Entity-Component-System). Ключевые принципы:

- **Нет наследования** — поведение через комбинацию компонентов
- **Нет `Vec<Box<dyn Trait>>`** внутри компонентов — нарушает cache-locality и параллельные запросы
- **Observers** вместо `EventReader/EventWriter` для реакций на события
- **Data-driven** — весь баланс в RON-файлах, без пересборки

---

## 2. Состояния приложения (`AppState`)

```rust
pub enum AppState {
    Loading,    // загрузка ассетов
    MainMenu,   // главное меню
    Playing,    // активная игра
    Paused,     // пауза
    GameOver,   // экран победы/поражения
    Editor,     // редактор уровней
}
```

**Правило:** каждая система должна иметь `run_if(in_state(...))` guard, если она не должна работать во всех состояниях.

```
┌──────────┐    New Game    ┌─────────┐    ESC      ┌────────┐
│ MainMenu │ ─────────────▶ │ Playing │ ──────────▶ │ Paused │
└──────────┘                └─────────┘             └────────┘
     ▲                           │ Warbase              │
     │ ESC                       │ destroyed            │ Continue
     │                           ▼                      │
     │                      ┌──────────┐ ◀─────────────┘
     └──────────────────────│ GameOver │
     │                      └──────────┘
     │ Editor button
     ▼
┌────────┐
│ Editor │
└────────┘
```

---

## 3. Структура модулей

### Ядро (`core/`)

| Файл | Описание |
|------|----------|
| `time.rs` | `GameTime` — игровые часы, `game_day` счётчик, `seconds_per_day=30.0` |
| `team.rs` | `Team { Player, Enemy, Neutral }` — компонент команды |
| `health.rs` | `Health { current, max }`, `apply_damage()`, `heal()` |
| `events.rs` | `EntityDamaged`, `EntityDestroyed` — Bevy observers |
| `resources.rs` | `GameConfig` (из game.ron), `GameState` |

### Роботы (`robot/`)

Модульная система без наследования:

```rust
// Каждый компонент независим
#[derive(Component)] pub struct Chassis { chassis_type: ChassisType, base_hp: f32, speed: f32 }
#[derive(Component)] pub struct WeaponSlots { slots: [Option<WeaponData>; 3] }
#[derive(Component)] pub struct Electronics { radar_range: f32, accuracy_bonus: f32, fire_rate_bonus: f32 }
#[derive(Component)] pub struct Nuclear { blast_radius: f32, detonation_delay: f32, armed: bool }
#[derive(Component)] pub struct RobotStats { max_hp: f32, speed: f32, capture_time: f32 }
```

`RobotBlueprint` → валидация → `spawn_robot()` → entity с ~15 компонентами (два вызова `insert` из-за лимита tuple 15 элементов в Bevy).

### Движение (`movement/`)

A* pathfinding на `MapGrid 64×64`:

```
ChangedMovementTarget → compute_path() → Path(VecDeque<Vec3>)
                                              ↓
                                    follow_path() [FixedUpdate]
                                              ↓
                                    Velocity → Transform
```

AntiGrav шасси (`can_fly=true`) игнорирует заблокированные клетки.

### Пространственный индекс (`spatial/`)

`SpatialIndex` — uniform grid, обновляется раз в `FixedUpdate`. Используется:
- `combat/targeting.rs` — `acquire_targets()` ищет врагов в радиусе
- `movement/steering.rs` — `separate_robots()` находит соседей для расталкивания
- `ai/command.rs` — ближайшие враги/фабрики для AI-решений

Это устраняет O(n²) перебор при ≥50 роботах.

### Боевая система (`combat/`)

```
acquire_targets() → CombatTarget(Entity)     ← через SpatialIndex
    ↓
fire_weapons()
    ├── Cannon/Phasers → EntityDamaged observer (hitscan)
    └── Missile → spawn Projectile entity
                      ↓
              move_projectiles() → EntityDamaged при попадании
```

Цель пересчитывается только при смерти текущей или выходе за радиус.
Все в `FixedUpdate`, только в `Playing`.

### AI (`ai/`)

**Utility-based scoring** с FSM состояниями. Модули:

| Файл | Содержание |
|------|------------|
| `ai/build.rs` | `ai_build_robots` — очередь постройки, тратит `EnemyResources` |
| `ai/command.rs` | `ai_assign_commands` — SeekAndCapture/SeekAndDestroy/DestroyBase |
| `ai/scoring.rs` | `select_blueprint`, `capture_priority`, `threat_ratio` |
| `ai/victory.rs` | проверка победы/поражения |
| `ai/state.rs` | `AICommander` ресурс, `GameResult` |

```
AICommander (Decision loop каждые decision_interval сек)
    ↓
Utility scoring:
  capture_priority  = нейтральные фабрики / угрозы
  threat_ratio      = вражеские роботы / свои роботы
  select_blueprint  = выбор конфигурации по counter % 10

→ ai_build_robots (очередь на Enemy warbase, с расходом EnemyResources)
→ ai_assign_commands (idle роботы → SeekAndCapture/SeekAndDestroy)
→ Ядерная стратегия при ≥N фабриках
```

ИИ использует `EnemyResources` — ту же экономику, что и игрок, без читов.

### Структуры (`structure/`)

Механика захвата:

```
Робот с CaptureProgress рядом со Structure
    ↓
capture_progress += dt (10 сек базово, 7 с электроникой)
    ↓ (при полном захвате)
observer on_structure_captured
    ↓
Factory.owner = Team::Player
ProductionRate → PlayerResources (каждый game_day)
```

Варбейс — только ядерный заряд (blast_radius=8).

---

## 4. Системы по расписанию

```
Startup:
  spawn_camera, load_map, spawn_player_scout, spawn_structures

FixedUpdate (только Playing):
  acquire_targets → fire_weapons → move_projectiles
  ai_assign_commands, ai_build_robots
  follow_path, compute_path
  tick_game_time, tick_production
  capture_progress, scout_collision
  process_commands, update_patrol
  recalc_stats

Update:
  player_input → scout_movement
  zoom_camera, rotate_camera [не в Editor]
  draw_muzzle_flashes, draw_projectiles [Playing/Paused]
  editor systems [только Editor]

PostUpdate:
  follow_target [только Playing/Paused]

EguiPrimaryContextPass:
  draw_hud, draw_minimap [Playing/Paused/GameOver]
  draw_editor_toolbox, draw_editor_map_props [только Editor]
  draw_main_menu [только MainMenu]
  draw_pause_menu [только Paused]
```

---

## 5. Архитектура камеры

**Принцип:** одна единственная камера `IsometricCamera` — всегда активна.

Это критично для **bevy_egui 0.39**: `setup_primary_egui_context_system` привязывает egui к первой обнаруженной камере при старте. Если камера отключается или удаляется — egui перестаёт рендериться.

```
IsometricCamera (всегда активна)
    │
    ├── [Playing/Paused]  CameraTarget(scout_entity) → follow_target в PostUpdate
    │
    └── [Editor]          EditorCamera (маркер-компонент)
                              ↓
                          free_camera_movement (WASD + зум + Z/C вращение)
```

**Переход в Editor:**
```rust
// OnEnter(Editor)
commands.entity(iso_cam).insert(EditorCamera);  // добавляем маркер
*vis = Visibility::Hidden;                       // скрываем GameWorldEntity

// OnExit(Editor)
commands.entity(iso_cam).remove::<EditorCamera>();
*vis = Visibility::Inherited;
```

**Конфликт систем:**
- `zoom_camera` / `rotate_camera` → `run_if(not(in_state(Editor)))` — чтобы не дублировать управление
- `follow_target` → `run_if(Playing.or(Paused))` — чтобы не перехватывать WASD в Editor

---

## 6. Редактор уровней (`editor/`)

```
EditorState {
    current_tool: EditorTool,
    brush_cell_type: CellType,
    factories: Vec<FactoryDef>,
    warbases: Vec<WarbaseDef>,
    player_spawn: (u32, u32),
    dirty: bool,
    file_name: String,
}

EditorTool { TerrainBrush, PlaceFactory, PlaceWarbase, PlacePlayerSpawn, Erase }

Системы:
  pick_cell           — raycast к Y=0, получить (x, z) клетки
  apply_tool          — применить инструмент по клику
  update_hover_preview — подсветка клетки под курсором
  draw_editor_grid    — gizmos grid overlay (alpha=0.08)
  draw_editor_structures — gizmos фабрик/варбейсов/спавна

Observers:
  on_rebuild_terrain_cell — пересборка mesh клетки при изменении типа
```

Сохранение в `data/maps/{name}.ron` + `data/scenarios/{name}.ron`.

---

## 7. Сохранения (`save/`)

- Формат: RON (человекочитаемый)
- 3 именованных слота + автосохранение (`saves/autosave.ron`)
- Версионирование: `SAVE_VERSION = 2`, миграция через `migrate_v1_to_v2()`
- Автосохранение при смене игрового дня (`check_autosave` в FixedUpdate)

`SaveData` содержит: все роботы (компоненты), структуры (владелец, прогресс захвата), ресурсы игрока и врага, `GameTime`, позицию скаута, состояние AI-командира.

`apply_pending_load` разбит на этапы (7 шагов с `info!` логами). `warn!` при stale entity в `MapGrid`.

---

## 8. Data-driven дизайн

Все характеристики вынесены в RON без пересборки:

```
configs/
├── game.ron          — seconds_per_day, map размер, scout_speed
├── chassis.ron       — 4 шасси × {hp, speed, mobility, cost}
├── weapons.ron       — 3 оружия × {damage, range, reload, cost}
├── electronics.ron   — бонусы {accuracy, fire_rate, radar, capture}
├── nuclear.ron       — {blast_radius, detonation_delay, cost}
├── factories.ron     — production_rate по типам
└── ai.ron            — {decision_interval, build_interval, aggression, nuclear_threshold}
```

---

## 9. Паттерны Bevy, используемые в проекте

| Паттерн | Применение |
|---------|------------|
| Observer | `EntityDamaged`, `EntityDestroyed`, `on_rebuild_terrain_cell`, `on_structure_captured` |
| Marker component | `EditorCamera`, `GameWorldEntity`, `EditorEntity`, `Selected`, `RobotMarker` |
| Command pattern | `CommandQueue { current, queue: VecDeque<RobotCommand> }` |
| Run conditions | `run_if(in_state(...))` на всех игровых системах |
| Component insert/remove | Динамическая логика через добавление/удаление компонентов |
| Resource | `MapGrid`, `PlayerResources`, `EditorState`, `ModuleRegistry`, `GameConfig` |
| `OnEnter`/`OnExit` | Переходы между состояниями (setup/cleanup) |

---

## 10. Feature flags

```toml
[features]
dev         = ["bevy/dynamic_linking"]  # быстрая перекомпиляция + hot-reload конфигов
debug_tools = []                         # debug overlay, robot spawn panel
```

- `dev` — добавляет `DevHotReloadPlugin` (`src/dev_tools/`): polling mtime каждые 2с,
  при изменении перезагружает `GameConfig` и `ModuleRegistry` без рестарта.
- `debug_tools` — панель спавна роботов, FPS overlay, debug gizmos.

В release-сборке оба флага не включаются.

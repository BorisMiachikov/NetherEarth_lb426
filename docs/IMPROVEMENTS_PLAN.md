# Nether Earth LB426 — План улучшений архитектуры и кода

**Дата аудита:** 2026-04-18
**Автор:** Claude Opus 4.7
**Базовая ревизия:** `master` @ `dba8522` (после подключения SFX/музыки)
**Объём кодовой базы:** ~9 670 LOC, 80 файлов
**Общая оценка:** архитектура ECS чистая (нет `Vec<Box<dyn Trait>>`, `Arc<Mutex>`, `EventReader/Writer`), 29 unit-тестов проходят. Основные риски — **масштабируемость combat/AI при >50 роботах**, **монолитные UI/save-функции**, **технический долг** (49 warnings) и **отсутствие версионирования сохранений**.

---

## 1. Сводка метрик

| Метрика | Значение | Комментарий |
|---------|----------|-------------|
| LOC | 9 670 | 80 файлов |
| Файлов >300 строк | 8 | `save/systems.rs` (530), `player/commands_ui.rs` (507), `ui/menu.rs` (443), `ai/systems.rs` (394), `ui/builder_ui.rs` (386), `editor/mod.rs` (365), `editor/ui.rs` (361), `editor/tools.rs` (292) |
| `unwrap()/expect()` в runtime | 4 | `structure/capture.rs:145`, `structure/warbase.rs:68`, `player/commands_ui.rs:181`, `save/io.rs:89-90` (тесты) |
| `TODO/FIXME/XXX/HACK` | 0 | ✅ чисто |
| `EventReader/EventWriter` | 0 | ✅ только observers |
| `Arc<Mutex>`, `RwLock`, `Vec<Box<dyn Trait>>` | 0 | ✅ |
| Compiler warnings | ~49 | unused imports, deprecated egui API, dead_code |
| Unit-тесты | 29 в 8 файлах | integration/system тестов нет |

---

## 2. Критические проблемы (🔴 P1)

### 2.1 Монолит `apply_pending_load()` — 134 строки, 16 параметров
- **Где:** [`src/save/systems.rs:285-418`](../src/save/systems.rs)
- **Проблема:** функция оркестрирует despawn роботов, обновление структур через `MapGrid`, восстановление ресурсов, времени, AI, скаута, respawn роботов, HP, приказов, ядерного заряда. `#[allow(clippy::too_many_arguments)]` подавляет warning. Тяжело отлаживать и покрывать тестами.
- **Действие:**
  1. Выделить `restore_structures(&mut structures_q, &mut map, ...)`
  2. `restore_resources(&mut resources, &mut game_time, ...)`
  3. `restore_robots(&mut commands, &registry, ...)` (после despawn)
  4. `restore_game_state(&mut ai, &mut scout, ...)`
  5. Оставить `apply_pending_load` как тонкий оркестратор ≤40 строк.
- **Тесты:** round-trip save→load в integration-тесте (стандартный сценарий → сохранение → новая игра → загрузка → сверить счётчики).

### 2.2 Монолит `robot_info_panel()` — 359 строк
- **Где:** [`src/player/commands_ui.rs:92-450`](../src/player/commands_ui.rs)
- **Проблема:** одна функция рисует single-robot и multi-selection UI, считает HP-бар, собирает составы шасси, обрабатывает клики toggle manual / deselect. Изменения ломают всё.
- **Действие:** разделить на
  - `collect_selection_info(...) -> SelectionView` (immutable сбор)
  - `draw_single_robot_panel(ui, &view, &localization, ...)`
  - `draw_multi_robot_panel(ui, &view, ...)`
  - `handle_selection_actions(events, ...)` (observers/commands)
- **Целевой размер:** ≤100 строк на подфункцию.

### 2.3 Монолит `on_trigger_new_game()` — 97 строк, 15+ параметров
- **Где:** [`src/save/systems.rs:427-523`](../src/save/systems.rs)
- **Действие:** разделить на `reset_structures`, `reset_resources`, `reset_player_scout`, `reset_ai_commander`, `spawn_initial_robots`. Оркестратор вызывает их по порядку.

### 2.4 Отсутствие версионирования/миграций сохранений
- **Где:** [`src/save/types.rs`](../src/save/types.rs), [`src/save/io.rs`](../src/save/io.rs)
- **Проблема:** константа `SAVE_VERSION = 1` объявлена, но при чтении не проверяется; нет ни schema-тега в файле, ни фолбэка на несовместимый формат. Следующее расширение `SaveData` молча поломает существующие автосохранения.
- **Действие:**
  1. Добавить поле `version: u32` в `SaveData` (serde default = 1 для обратной совместимости).
  2. В `read_save()` сравнивать с `SAVE_VERSION`, при несовпадении вызывать цепочку `migrate_v1_to_v2` и т. д.
  3. Покрыть тестом: «читаем файл v1, получаем корректный v2 в памяти».
  4. При критическом несовпадении — возвращать `Err` и показывать игроку сообщение через тост/диалог, не ронять игру.

### 2.5 `MapGrid` рассинхрон при загрузке
- **Где:** [`src/save/systems.rs:312-327`](../src/save/systems.rs)
- **Проблема:** при `apply_pending_load()` обновляются только существующие `Structure(Entity)`-ячейки. Если сохранена структура, которой сейчас нет в мире (например, пересобрали сценарий), `MapGrid` останется с мёртвой `Entity`-ссылкой → picking и pathfinding получат фантомную клетку.
- **Действие:**
  - Перед применением загрузки полностью перестраивать `MapGrid.cells` на основе актуальной сцены структур (или на основе `SaveData`).
  - Добавить invariant-тест: после load все `MapGrid::Structure(e)` валидны через `World::get::<Transform>(e)`.

### 2.6 Production acquire_targets: квадратичная сложность
- **Где:** [`src/combat/targeting.rs:11-44`](../src/combat/targeting.rs)
- **Проблема:** для каждого атакующего перебираются все роботы. При 50 vs 50 ~5 000 итераций/кадр; на FixedUpdate 60 Hz это ещё не критично, но с 100+ юнитами деградация заметная.
- **Действие:** ввести `Res<SpatialIndex>` (uniform grid по клеткам карты, размер ячейки = max weapon range / 2), обновлять раз в FixedUpdate. `acquire_targets` запрашивает только соседние ячейки. То же решение применить к `movement/steering.rs::separate_robots` и `ai/systems.rs` (поиск ближайшей цели/структуры).
- **Замер:** добавить bench-сценарий (80 роботов в бою) и замерить до/после.

---

## 3. Важные улучшения (🟡 P2)

### 3.1 Чистка 49 compiler warnings
- **Действие:**
  - `cargo fix --allow-dirty --lib`,
  - ручной проход по `#[allow(dead_code)]` и неиспользуемым полям (`owner_team`, `CommandQueue::current`, `Health::heal`, `world_bounds()`, `ProductionRate`-поля, `CameraTarget`, `IsometricCamera` и т. д.) — либо удалить, либо явно подключить,
  - заменить deprecated `ctx.screen_rect()` → `ctx.content_rect()` (bevy_egui 0.39),
  - цель: `cargo clippy --all-targets -- -D warnings` зелёный в CI.

### 3.2 Локализация UI-строк редактора и builder_ui
- **Где:** [`src/editor/ui.rs`](../src/editor/ui.rs), [`src/ui/builder_ui.rs`](../src/ui/builder_ui.rs)
- **Проблема:** в коде остаются русские литералы вида `"Общий"`, `"Шасси"`, подписи к инструментам редактора — не проходят через `Res<Localization>`. ROADMAP-пункт 11.19 закрыт, но фактически локализация только основного меню и паузы.
- **Действие:** завести `assets/locales/{en,ru}.ron` ключи `ui.resource.general`, `editor.tool.terrain_brush` и т. д., заменить все `"..."` строки в UI на `t!("key")`-хелпер. Завести clippy-lint (ручной grep-тест) на кириллицу в `src/ui/**` и `src/editor/**`.

### 3.3 Диалог «Сохранить изменения?» при выходе из редактора (11.18)
- **Где:** [`src/editor/ui.rs`](../src/editor/ui.rs), [`src/editor/state.rs`](../src/editor/state.rs)
- **Статус:** `dirty`-флаг реализован, но модальный диалог отсутствует — при попытке выйти в главное меню или закрыть окно изменения теряются без подтверждения.
- **Действие:** egui-модальное окно с тремя кнопками: «Сохранить», «Не сохранять», «Отмена». Подключить к:
  - переходу `AppState::Editor → MainMenu`,
  - `ExitApp`-observer при `dirty == true`.

### 3.4 Spatial index + реюз буферов в `separate_robots`
- **Где:** [`src/movement/steering.rs:134-198`](../src/movement/steering.rs)
- **Проблема:** на каждом тике аллоцируется `Vec<(Entity, Vec3)>`; O(n²). После внедрения spatial index (см. 2.6) достаточно запрашивать соседей в радиусе 2 клеток.
- **Действие:** использовать общий `Res<SpatialIndex>` + `Local<Vec<...>>` для амортизации аллокации. Добавить `debug_assert!(robots.len() < 200)` с понятным сообщением.

### 3.5 Integration-тесты save/load и full-cycle
- **Где:** новый `tests/save_roundtrip.rs`
- **Что покрыть:**
  - сохранение → загрузка → состояние идентично (роботы, HP, ресурсы, AI, скаут);
  - загрузка повреждённого RON → `Err`, AppState не меняется;
  - несовместимая версия → миграция;
  - автосохранение не перетирается при failed load.

### 3.6 Разбить `ai/systems.rs` (394 строки) на подмодули
- **Где:** [`src/ai/systems.rs`](../src/ai/systems.rs)
- **Действие:** выделить `ai/build.rs` (`ai_build_robots`, ротация blueprint), `ai/command.rs` (`ai_assign_commands`), `ai/victory.rs` (`check_victory_defeat`). В `mod.rs` только plugin registration.

### 3.7 Разбить `ui/menu.rs` (443 строки)
- **Где:** [`src/ui/menu.rs`](../src/ui/menu.rs)
- **Действие:** `ui/menu/main.rs`, `ui/menu/pause.rs`, `ui/menu/scenario_picker.rs`, `ui/menu/save_slots.rs`. `mod.rs` = plugin + `ScenarioList::load_from_dir`.

### 3.8 Расширить RON hot-reload для конфигов баланса
- **Где:** [`src/robot/registry.rs`](../src/robot/registry.rs), `data/configs/*.ron`
- **Проблема:** конфиги загружаются на Startup, hot-reload не работает → балансировка требует рестарта.
- **Действие:** за feature `dev` подключить `bevy_common_assets::ron::RonAssetPlugin` + AssetEvent-observer, который обновляет `ModuleRegistry`/`GameConfig` на лету.

### 3.9 Валидация границ карты при спавне
- **Где:** [`src/robot/bundle.rs`](../src/robot/bundle.rs), [`src/map/grid.rs`](../src/map/grid.rs)
- **Проблема:** `world_to_grid` возвращает `None` для `pos.x < 0`, но robot может быть заспавнен снаружи карты (через save с багом или сценарий). В бою/pathfinding он станет невидим.
- **Действие:** `spawn_robot()` — `debug_assert!(map.contains_world(pos))`; при release-сборке клампить позицию к ближайшей валидной клетке + `warn!`.

### 3.10 Обработка повреждённого autosave без рассинхрона
- **Где:** [`src/save/systems.rs:231-247`](../src/save/systems.rs) `on_trigger_load`
- **Действие:** перед попыткой чтения сбрасывать `pending.0 = None`; при Err возвращать игрока в главное меню с тостом «Не удалось загрузить сохранение».

---

## 4. Полировка и nice-to-have (🟢 P3)

### 4.1 Debug-логирование ключевых решений
- `combat/targeting` — `debug!` при смене цели,
- `ai/systems` — `debug!` выбранный blueprint и его стоимость,
- `save/systems::apply_pending_load` — поэтапные `info!` с количеством восстановленных сущностей.

### 4.2 Инварианты и `debug_assert!`
- Robot имеет ровно один `Chassis` (проверка в `spawn_robot`).
- `MapGrid::Structure(e)` → `e` жив в `World`.
- `CaptureProgress.progress ∈ [0, required]`.

### 4.3 Graceful degradation аудио
- Если `audio/sfx/*.wav` отсутствует — логировать `warn!` один раз при старте, не spam в каждом кадре выстрела.

### 4.4 Профилирование большого сценария
- Создать `data/scenarios/stress.ron` с 80 стартовыми роботами и 20 фабриками.
- Запустить `cargo run --release`, снять Tracy-профиль до/после внедрения spatial index.

### 4.5 Замена placeholder-мешей (Phase 10.1-10.4)
- glb-ассеты для 4 шасси, структур, эффектов;
- PBR материалы и освещение под изометрию.

### 4.6 Единый хелпер локализации
- Завести макрос/функцию `t!(key)` или `localization.get(key)` с fallback на ключ при отсутствии перевода.

### 4.7 Keymap overlay F1 (10.9) — обновить под orbit (MMB+Z/C) и Editor-режим.

### 4.8 CI-пайплайн
- GitHub Actions: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.

---

## 5. Порядок выполнения (рекомендация)

| Шаг | Пункты | Оценка |
|-----|--------|--------|
| Шаг A — «Гигиена»    | 3.1 (warnings), 3.3 (диалог dirty), 3.2 (локализация) | 0.5–1 день |
| Шаг B — «Save/Load надёжность» | 2.1, 2.3, 2.4, 2.5, 3.5, 3.10 | 1.5–2 дня |
| Шаг C — «UI рефакторинг»      | 2.2, 3.6, 3.7                                 | 1 день |
| Шаг D — «Performance»         | 2.6 (spatial index), 3.4, 4.4 (профилирование) | 1.5 дня |
| Шаг E — «Polish»              | 4.1, 4.2, 4.3, 4.6, 4.7, 3.8, 3.9             | 1–1.5 дня |
| Шаг F — CI + тесты            | 4.8, расширение тестового покрытия             | 0.5 дня |

Итого: ~6–8 рабочих дней на весь план, без учёта Phase 10 (ассеты/анимации).

---

## 6. Незакрытые пункты ROADMAP, связанные с планом

- **11.18** диалог dirty → закрывается в шаге A.3.3.
- **11.19** локализация UI редактора → закрывается в шаге A.3.2.
- **6.13** стартовые условия из scenario RON → следует закрыть параллельно шагу B (чтобы load/save и сценарии сходились по схеме).
- **8.2** settings (громкость/разрешение) → шаг E, использует общий локализационный хелпер.
- **8.8** 3D-превью робота → отдельная задача, вне аудита.
- **10.1–10.7** ассеты/оптимизация → шаг E и позже.

---

## 7. Что оставить как есть

- ECS-модель (`robot/components.rs`, `core/*`) — соответствует спеке v2.0.
- Observer-based события (`core/events.rs`, `save/systems.rs`) — правильный путь.
- Изоляция редактора через маркер `EditorCamera` + `run_if(in_state(Playing))` — корректное решение регрессии 2026-04-11.
- Разделение FixedUpdate/Update/PostUpdate по плагинам — единообразное.

Эти решения менять НЕ нужно — любое «улучшение» здесь сломает совместимость с правилами проекта.

# Nether Earth LB426 — Дорожная карта разработки

**Стек:** Rust (Stable), Bevy 0.18, Serde + RON  
**Тип:** Solo-разработка  
**Дата создания:** 2026-04-04  
**Последнее обновление:** 2026-04-06 — Фаза 9 реализована: save/load (RON, 3 слота + автосохранение), LocalizationPlugin (RU/EN), смена языка в паузе, "Продолжить" в главном меню  
**Авторитетная спецификация:** `1/Nether Earth LB426.md` (v2.0)

---

## 1. Статус модулей

| Модуль | Статус | Описание |
|--------|--------|----------|
| `app` | ✅ Реализован | AppPlugin, AppState (Menu/Playing/Paused/GameOver), настройка окна |
| `core` | ✅ Реализован | GameTime, Health, Team, EntityDamaged/Destroyed observers |
| `player` | ✅ Реализован | Скаут WASD+QE, выбор роботов (LMB/Shift/Ctrl+1-9), команды ПКМ |
| `robot` | ✅ Реализован | Chassis/WeaponSlots/Electronics/Nuclear, ModuleRegistry, RobotBlueprint, spawn |
| `ai` | ✅ Реализован | AICommander, Utility scoring, SeekAndDestroy/Capture/DestroyBase, ядерная стратегия |
| `command` | ✅ Реализован | Idle, MoveTo, SeekAndDestroy, SeekAndCapture, DestroyEnemyBase, Defend, Patrol + queue |
| `movement` | ✅ Реализован | A* по сетке, Velocity, MovementTarget, steering, follow_path, exploration_target |
| `combat` | ✅ Реализован | Targeting, Cannon/Missile/Phasers, projectiles, death, nuclear blast (роботы + структуры) |
| `economy` | ✅ Реализован | 7 ресурсов, производство по фабрикам, очередь постройки, HUD |
| `structure` | ✅ Реализован | Factory/Warbase, захват через VisionRange, прогресс-бар, смена владельца |
| `map` | ✅ Реализован | MapGrid 64×64, RON-загрузка, коллизия скаута, визуальная сетка |
| `ui` | ✅ Реализован | HUD, миникарта, Builder UI, меню, пауза, Game Over, выбор сценария, панель юнитов |
| `camera` | ✅ Реализован | Изометрическая орто-камера, зум скроллом, следование за скаутом |
| `audio` | 🔶 Скелет | AudioSettings ресурс, интеграция без .ogg файлов |
| `save` | ⬛ Заглушка | Только stub-плагин |
| `debug` | ✅ Реализован | Gizmos сетка, overlay (координаты/FPS/GameTime), egui-панель спавна роботов |

---

## 2. Пробелы спецификации (заполнены примерными значениями)

Все значения ниже — начальные заглушки для разработки. Тюнинг при плейтестинге через RON-конфиги без пересборки.

### 2.1 Здоровье робота
**Формула:** `max_hp = base_hp(chassis) + sum(module_weight) * 2`

| Шасси | Base HP | Speed | Mobility | Can Fly |
|-------|---------|-------|----------|---------|
| Wheels | 50 | 1.2 | 0.6 | false |
| Bipod | 40 | 1.5 | 0.8 | false |
| Tracks | 80 | 1.0 | 0.9 | false |
| AntiGrav | 30 | 2.0 | 1.0 | true |

### 2.2 Оружие

| Оружие | Damage | Range | Reload (s) | Множитель | Тип |
|--------|--------|-------|------------|-----------|-----|
| Cannon | 15 | 10 | 1.2 | 1× | Hitscan |
| Missile | 45 | 30 | 3.0 | 3× | Projectile (speed=8.0) |
| Phasers | 8 (per tick) | 6 | 0.15 | 4× DPS | Hitscan (continuous) |

### 2.3 Электроника
- Accuracy bonus: +30%
- Fire rate bonus: +20%
- Radar range: 20 game units

### 2.4 Ядерный заряд
- Радиус взрыва: 8 game units
- Задержка детонации: 2.0 секунды
- Уничтожает ВСЁ в радиусе (включая своего робота)
- Единственный способ уничтожить Warbase

### 2.5 Стоимость модулей

| Модуль | Ресурс | Кол-во | General |
|--------|--------|--------|---------|
| Wheels | Chassis | 15 | 5 |
| Bipod | Chassis | 20 | 10 |
| Tracks | Chassis | 40 | 15 |
| AntiGrav | Chassis | 60 | 25 |
| Cannon | Cannon | 10 | 5 |
| Missile | Missile | 25 | 10 |
| Phasers | Phasers | 30 | 15 |
| Electronics | Electronics | 20 | 10 |
| Nuclear | Nuclear | 50 | 30 |

### 2.6 Время постройки
**Формула:** `build_time = sum(module_costs) * 0.5` секунд

### 2.7 Скорость захвата

Базовое время захвата одинаково для всех шасси: **10 секунд**.  
Электроника: −30% → **7 секунд**.

Робот видит структуры только в радиусе `VisionRange` (8 без электроники, 20 с электроникой).  
При отсутствии видимых целей — исследует карту.

### 2.8 Производство ресурсов
- 1 игровой день = 30 реальных секунд (настраивается в game.ron)
- Каждая захваченная фабрика: +5 специфического ресурса + 2 General за игровой день

---

## 3. Ключевые архитектурные решения

### 3.1 ECS-чистота (из v2.0)
> **ЗАПРЕЩЕНО:** `Vec<Box<dyn Trait>>` внутри компонентов — ломает параллельные запросы и cache-locality.

**Правильный паттерн:**
```rust
#[derive(Component)]
pub struct Chassis { pub chassis_type: ChassisType, pub speed: f32, pub mobility: f32 }

#[derive(Component)]
pub struct WeaponSlots { pub slots: [Option<WeaponData>; 3] }

#[derive(Component)]
pub struct Electronics { pub radar_range: f32, pub accuracy_bonus: f32 }
```

### 3.2 FixedUpdate vs Update
- **FixedUpdate:** AI, движение/pathfinding, combat damage, capture progress
- **Update:** рендеринг, UI, input
- **PostUpdate:** камера

### 3.3 Event-driven коммуникация
Bevy Events для межсистемного общения:
- `EntityDamaged`, `EntityDestroyed`
- `StructureCaptured`, `ResourceChanged`
- `RobotBuilt`, `CommandIssued`

### 3.4 Порядок систем (из v2.0)
1. Input → 2. Player Movement → 3. Command Issuing → 4. AI/Behavior →
5. Movement/Navigation → 6. Combat → 7. Capture → 8. Production →
9. Construction → 10. Health/Death → 11. UI Update

### 3.5 RON Hot-Reload
Использовать Bevy asset hot-reload для RON-конфигов во время разработки — экономия времени при тюнинге баланса.

---

## 4. Фазы разработки

---

### ✅ Фаза 0: Инициализация проекта — ЗАВЕРШЕНА

**Цель:** Пустое Bevy-приложение компилируется, запускается, показывает окно. Структура каталогов создана.

- [x] **0.1** `cargo init`, Cargo.toml с зависимостями (bevy 0.18, serde, ron, bevy_egui 0.39) `[S]`
- [x] **0.2** Создать структуру каталогов: 16 модулей с `mod.rs` `[S]`
- [x] **0.3** `app/state.rs` — `AppState` enum (MainMenu, Playing, Paused, GameOver) `[S]`
- [x] **0.4** `lib.rs` — регистрация всех плагинов-заглушек `[S]`
- [x] **0.5** `main.rs` — `App::new()`, DefaultPlugins, настройка окна 1280×720 `[S]`
- [x] **0.6** Создать `assets/`, `configs/`, `data/` с подпапками `[S]`
- [x] **0.7** `.gitignore`, инициализация git, push на GitHub `[S]`
- [x] **0.8** Скелеты RON-конфигов (chassis.ron, weapons.ron, electronics.ron, nuclear.ron, maps/default.ron) `[S]`
- [x] **0.9** Проверить инициализацию bevy_egui `[S]`

**Критерии проверки:**
- `cargo build` без предупреждений
- `cargo run` открывает окно
- `cargo clippy` — ноль предупреждений
- RON-файлы парсятся (тривиальный тест десериализации)

**Плагины:** bevy 0.18, serde + ron, bevy_egui 0.32

---

### ✅ Фаза 1: Ядро + Полёт скаута — ЗАВЕРШЕНА

**Цель:** Игрок летает на скауте над плоской сеткой. Камера следует. Debug-overlay показывает координаты.

**Зависимости:** Фаза 0

- [x] **1.1** `core/time.rs` — `GameTime`: игровые часы, `game_day` счётчик, `seconds_per_day = 30.0` `[S]`
- [x] **1.2** `core/team.rs` — `Team` enum (Player, Enemy, Neutral), derive Component `[S]`
- [x] **1.3** `core/health.rs` — `Health { current: f32, max: f32 }`, apply_damage, heal `[S]`
- [x] **1.4** `core/events.rs` — EntityDamaged/Destroyed observers, StructureCaptured, ResourceChanged `[S]`
- [ ] **1.5** `core/resources.rs` — `GameConfig` из game.ron, `GameState` (running/paused) `[M]` *(отложено)*
- [x] **1.6** `map/grid.rs` — `MapGrid` 64×64, типы ячеек (Open, Blocked, Structure), конверсия world↔grid `[M]`
- [x] **1.7** `map/loader.rs` — Загрузка карты из RON (default.ron), заполнение MapGrid `[M]`
- [x] **1.8** `map/mod.rs` — MapPlugin: спавн плоскости с видимой сеткой Gizmos `[M]`
- [x] **1.9** `camera/systems.rs` — Изометрическая ортографическая камера, следование за скаутом, зум скроллом `[M]`
- [x] **1.10** `player/components.rs` — `PlayerScout`, `ScoutMovement`, `ScoutMoveIntent` `[S]`
- [x] **1.11** `player/input.rs` — WASD+QE ввод → `ScoutMoveIntent` `[M]`
- [x] **1.12** `player/systems.rs` — Движение скаута, ограничение по границам карты `[M]`
- [x] **1.13** Спавн скаута с placeholder-мешем (цветной куб) `[S]`
- [x] **1.14** `debug/overlay.rs` — egui: позиция скаута, game time, FPS, ячейка под курсором `[M]`
- [x] **1.15** `debug/gizmos.rs` — Переключаемые линии сетки, границы карты `[S]`
- [x] **1.16** Загрузка RON-конфигов через serde (chassis.ron, weapons.ron и др.) `[M]`

**Критерии проверки:**
- Скаут летает плавно с WASD+QE
- Камера следует в изометрической проекции, зум работает
- Скаут не выходит за границы карты 512×512
- Debug-overlay отображает координаты, время, FPS
- GameTime инкрементирует game_day каждые 30 реальных секунд
- Unit-тесты: MapGrid конверсия координат, GameTime, RON-загрузчик

**Плагины:** bevy_common_assets 0.15 (загрузка RON как Bevy-ассетов)

---

### ✅ Фаза 2: Структуры + Населённая карта — ЗАВЕРШЕНА

**Цель:** На карте размещены фабрики и варбейсы из RON-данных. Скаут летает среди них. Цвета команд.

**Зависимости:** Фаза 1

- [x] **2.1** `structure/factory.rs` — `Factory`, `FactoryType` enum (7 типов), `ProductionRate` `[M]`
- [x] **2.2** `structure/warbase.rs` — `Warbase`, `ProductionQueue` (заглушка) `[S]`
- [x] **2.3** `structure/capture.rs` — `CaptureProgress { progress, required }`, `Capturable` marker `[S]`
- [x] **2.4** `structure/mod.rs` — StructurePlugin: спавн структур из данных карты `[M]`
- [x] **2.5** Рендеринг структур: цветные материалы по команде, cuboid-меши (factory 1.4×1.0×1.4, warbase 2.0³) `[M]`
- [x] **2.6** MapGrid: ячейки со структурами помечены `Structure(Entity)` `[S]`
- [x] **2.7** Тестовая карта RON: 1 warbase Player (8,8), 1 warbase Enemy (56,56), 8 нейтральных фабрик `[M]`
- [x] **2.8** `map/collision.rs` — Коллизия скаута (axis-separated) `[M]`
- [x] **2.9** Tooltip при наведении: egui Window с типом/владельцем структуры `[S]`
- [x] **2.10** `economy/resource.rs` — `ResourceType` enum (7 шт.), `PlayerResources` (HashMap), стартовые значения `[M]`
- [x] **2.11** `economy/mod.rs` — EconomyPlugin: регистрация PlayerResources `[S]`

**Критерии проверки:**
- Карта загружается из RON, структуры на правильных позициях
- Фабрики и варбейсы различимы визуально (разные меши/цвета)
- Скаут не пролетает сквозь структуры
- Tooltip при приближении к структуре
- PlayerResources инициализируется со стартовыми значениями
- Unit-тесты: спавн структур из RON, MapGrid разметка

---

### ✅ Фаза 3: Роботы + Базовое движение — ЗАВЕРШЕНА

**Цель:** Роботы спавнятся через debug-панель, двигаются по сетке с A*-pathfinding, сталкиваются с terrain/structures.

**Зависимости:** Фаза 2

- [x] **3.1** `robot/components.rs` — `RobotMarker`, `Chassis`, `WeaponSlots [Option<WeaponData>; 3]`, `Electronics`, `Nuclear`, `RobotStats` `[L]`
- [x] **3.2** `ChassisType` enum, `WeaponType` enum, `WeaponData` struct `[M]`
- [x] **3.3** `robot/registry.rs` — `ModuleRegistry` ресурс, загрузка из RON-конфигов `[M]`
- [x] **3.4** `robot/builder.rs` — `RobotBlueprint`, валидация, расчёт стоимости и build_time `[M]`
- [x] **3.5** `robot/bundle.rs` — `spawn_robot()`: entity со всеми компонентами, цвет команды, 4 типа мешей `[M]`
- [x] **3.6** `robot/systems.rs` — заглушка (stats пересчитывается при спавне в bundle) `[M]`
- [x] **3.7** `movement/velocity.rs` — `Velocity`, `MovementTarget(Vec3)` `[S]`
- [x] **3.8** `movement/pathfinding.rs` — A* BinaryHeap на MapGrid, can_fly для AntiGrav `[L]`
- [x] **3.9** `movement/steering.rs` — `compute_path` (Changed<MovementTarget>), `follow_path` (FixedUpdate) `[M]`
- [x] **3.10** `command/command.rs` — Idle, MoveTo, SeekAndDestroy, SeekAndCapture, Defend, Patrol `[S]`
- [x] **3.11** `command/queue.rs` — `CommandQueue { current, queue: VecDeque }` `[S]`
- [x] **3.12** `command/systems.rs` — dispatch Changed<RobotCommand>, SeekAndDestroy → nearest enemy `[M]`
- [x] **3.13** Debug-панель: спавн робота через egui (chassis, weapons, команда) `[M]`
- [x] **3.14** ПКМ → MoveTo для выбранного робота `[M]`
- [x] **3.15** Terrain-коллизия через A* (Blocked/Structure не проходимы) `[S]`
- [ ] **3.16** Расталкивание роботов при наложении `[M]` *(заглушка `separate_robots`)*
- [x] **3.17** 4 разных Cuboid-меша по типу шасси `[S]`
- [x] **3.18** `Health::apply_damage`, observer EntityDamaged → EntityDestroyed `[M]`

**Критерии проверки:**
- Debug-панель спавнит любой из 4 типов шасси
- Роботы обходят препятствия к цели MoveTo
- Разные шасси двигаются с разной скоростью (визуально заметно)
- AntiGrav игнорирует terrain-препятствия (летает над)
- Роботы не перекрываются друг с другом и структурами
- RobotBlueprint: валидация отклоняет невалидные конфигурации
- Unit-тесты: A* (оптимальный путь, недостижимая цель), валидация blueprint, recalc_stats, стоимость

**Плагины:** Рассмотреть bevy_picking 0.20 для выбора роботов (или ручной raycast)

---

### ✅ Фаза 4: Система команд + Взаимодействие игрока — ЗАВЕРШЕНА

**Цель:** Игрок выбирает роботов, отдаёт все типы приказов через скаут. Роботы реагируют.

**Зависимости:** Фаза 3

- [x] **4.1** MeshPickingPlugin + Observer On<Pointer<Click>> для выбора entity `[M]`
- [x] **4.2** LMB — один робот, Shift+LMB — мульти, Ctrl+1-9 — группы `[L]`
- [x] **4.3** `Selected`, `SelectionGroup(u8)`, желтый gizmo-круг под выбранным `[M]`
- [x] **4.4** ПКМ → MoveTo; P+ПКМ → накопление точек Patrol `[M]`
- [x] **4.5** egui-панель "Выбранный робот": кнопки SeekAndDestroy, SeekAndCapture, Defend, Idle `[M]`
- [x] **4.6** SeekAndDestroy: ближайший враг → MovementTarget `[M]`
- [x] **4.7** SeekAndCapture: stub (лог) `[M]`
- [x] **4.8** Defend(Vec3): InsertMovementTarget к позиции `[S]`
- [x] **4.9** Patrol: `update_patrol` циклически переключает точки `[M]`
- [ ] **4.10** Manual Control (Ctrl+LMB) `[L]` *(не реализовано)*
- [x] **4.11** `draw_command_indicators`: gizmo-линии MoveTo/Patrol/Defend `[S]`
- [x] **4.12** Панель: chassis, team, HP, weapons count, текущая команда `[M]`

**Критерии проверки:**
- Клик выбирает робота (визуальный фидбэк)
- Рамочный выбор для нескольких роботов
- Группы Ctrl+число, вызов по числу
- Все типы команд выдаются и исполняются (движение)
- Manual control переключает управление
- Индикаторы команд видны
- Integration-тест: MoveTo → робот доходит → Idle

**Плагины:** bevy_picking 0.20

---

### ✅ Фаза 5: Боевая система — ЗАВЕРШЕНА (частично)

**Цель:** Роботы стреляют друг в друга, получают урон, уничтожаются. Все 3 типа оружия. Ядерный заряд.

**Зависимости:** Фаза 4

- [x] **5.1** `combat/weapon.rs` — `WeaponCooldowns [f32; 3]`, `CombatTarget`, `MuzzleFlash` `[M]`
- [x] **5.2** `combat/targeting.rs` — `acquire_targets`: ближайший враг в радиусе оружия `[M]`
- [x] **5.3** `combat/fire.rs` — Cannon/Phasers (hitscan), Missile (projectile spawn) `[L]`
- [x] **5.4** EntityDamaged observer в core, Health уменьшается, EntityDestroyed при hp≤0 `[M]`
- [x] **5.5** `combat/death.rs` — `on_entity_destroyed`: despawn сущности `[M]`
- [x] **5.6** `combat/projectile.rs` — `Projectile` с самонаведением, speed=8.0, despawn при попадании `[L]`
- [x] **5.7** Nuclear area damage 8 units при `armed=true` `[L]` *(нет screen flash)*
- [ ] **5.8** Nuclear vs structures (structures не имеют Health) `[M]` *(Phase 6)*
- [ ] **5.9** Бонус электроники к fire rate `[S]` *(Phase 6)*
- [x] **5.10** `combat/visuals.rs` — MuzzleFlash gizmo-линии, оранжевые сферы ракет `[M]`
- [x] **5.11** SeekAndDestroy + targeting = полноценный бой `[M]`
- [ ] **5.12** Combat AI (отступление по порогу здоровья) `[M]` *(Phase 7)*
- [x] **5.13** acquire_targets + fire_weapons + move_projectiles в FixedUpdate `[S]`

**Критерии проверки:**
- Два робота разных команд в радиусе — автоматически сражаются
- Cannon, Missile, Phasers наносят правильный урон
- Missile спавнит projectile с homing
- Роботы уничтожаются при hp=0, despawn с эффектом
- Nuclear уничтожает всё в 8 units (включая своего робота)
- Warbase неуязвима для обычного оружия
- Электроника ускоряет огонь
- Производительность: 20 роботов в бою при 60 FPS
- Unit-тесты: расчёт урона, nuclear radius, friend/foe фильтр

**Плагины:** bevy_rapier3d 0.28 (опционально, для коллизий ракет) или простые distance-проверки

---

### Фаза 6: Экономика + Строительство роботов (14-18 дней)

**Цель:** Полный экономический цикл: фабрики захватываются → производят ресурсы → роботы строятся из ресурсов.

**Зависимости:** Фазы 5 + 2  
**Пререквизит:** Заполнить economy.ron, factories.ron стоимостями из раздела 2.5

- [x] **6.1** `structure/capture.rs` — Полная механика захвата: робот рядом → прогресс → смена владельца → StructureCaptured event `[L]`
- [x] **6.2** Визуал захвата: gizmo прогресс-бар, смена цвета меша, observer on_structure_captured `[M]`
- [x] **6.3** `economy/production.rs` — Тик производства (каждый game_day): +5 специф. ресурса +2 General от каждой фабрики `[M]`
- [x] **6.4** `economy/systems.rs` — ResourceChanged events триггерятся в tick_production `[S]`
- [x] **6.5** `ui/builder_ui.rs` — Экран строительства: chassis, 3 слота ComboBox, электроника, ядерка `[L]`
- [x] **6.6** Builder UI: таблица стоимости (красное при нехватке), оценка времени постройки `[M]`
- [x] **6.7** Валидация: NoWeapons/TooManyWeapons → кнопка Build задизейблена `[M]`
- [x] **6.8** Постройка: списание ресурсов → ProductionQueue → gizmo прогресс-бар → спавн у входа `[L]`
- [x] **6.9** Нажать B рядом с варбейсом → открыть/закрыть Builder UI `[M]`
- [x] **6.10** `ProductionQueue` с VecDeque, `tick_production_queue` (FixedUpdate) `[M]`
- [x] **6.11** `ui/hud.rs` — Панель ресурсов: 7 типов, день, обновление в реалтайме `[M]`
- [x] **6.12** Перезахват: враг захватывает фабрику игрока → потеря производства. Таймер сбрасывается `[M]`
- [ ] **6.13** Стартовые условия из scenario RON: начальные ресурсы, warbase, позиции `[M]`

**Критерии проверки:**
- Робот с приказом Capture подходит к фабрике и захватывает
- Прогресс-бар захвата виден
- Фабрика меняет цвет команды
- Ресурсы накапливаются каждые 30 реальных секунд
- HUD обновляет ресурсы
- Скаут открывает Builder UI на своём warbase
- Невалидные конфигурации не строятся
- Нехватка ресурсов блокирует постройку
- Очередь строительства работает
- Полный цикл: захват → ожидание → постройка → робот в бой
- Unit-тесты: расчёт производства, списание ресурсов, очередь

---

### Фаза 7: ИИ противника (14-20 дней)

**Цель:** ИИ-командир строит роботов, отдаёт приказы, захватывает фабрики, ведёт войну. Игра проходима от начала до конца.

**Зависимости:** Фаза 6

- [x] **7.1** `ai/state.rs` — `AICommander` + `AiConfig` (загрузка из configs/ai.ron) `[L]`
- [x] **7.2** `ai/scoring.rs` — `capture_priority`, `threat_ratio`, `select_blueprint` `[L]`
- [x] **7.3** `ai/systems.rs` — `ai_assign_commands` каждые decision_interval сек `[L]`
- [x] **7.4** ИИ постройка: `ai_build_robots` с ротацией 10 blueprint-вариантов, очередь на Enemy warbase `[M]`
- [x] **7.5** ИИ приказы: idle роботы получают SeekAndCapture или SeekAndDestroy по utility-скорингу `[L]`
- [x] **7.6** `configs/ai.ron` — decision_interval, build_interval, aggression, nuclear_threshold `[M]`
- [x] **7.7** Осведомлённость ИИ: VisionRange (8 без электроники / 20 с), исследование карты при отсутствии видимых целей `[M]`
- [x] **7.8** Ядерная стратегия: `arm_nuclear_on_arrival` + постройка nuclear-робота при ≥N фабриках `[M]`
- [x] **7.9** `check_victory_defeat` — GameResult resource, ориентируется на уничтожение варбейсов `[M]`
- [x] **7.10** `ui/gameover.rs` — экран победы/поражения: дней, фабрик обеих сторон `[M]`
- [x] **7.11** `update_seek_destroy` — динамическая смена цели при уничтожении текущей `[M]`
- [x] **7.12** Все AI-системы в FixedUpdate `[S]`

**Критерии проверки:**
- ИИ строит роботов самостоятельно
- ИИ роботы захватывают нейтральные фабрики
- ИИ атакует роботов и структуры игрока
- ИИ строит разнообразные конфигурации
- ИИ реагирует на угрозы (усиливает оборону)
- ИИ использует nuclear при возможности
- Победа: все warbase врага уничтожены/захвачены
- Поражение: все warbase игрока уничтожены
- Полное прохождение: 15-30 минут
- ИИ не читерит (использует ту же экономику)
- Производительность: 50+ роботов без просадки FPS

---

### Фаза 8: UI + Аудио (10-14 дней)

**Цель:** Полный UI с миникартой, HUD, главное меню, пауза. Звуковые эффекты и музыка.

**Зависимости:** Фазы 6 + 7

- [x] **8.1** `ui/menu.rs` — Главное меню: New Game, Exit; затемнение поверх мира `[M]`
- [ ] **8.2** Settings: громкость, разрешение, отображение клавиш `[M]` *(отложено)*
- [x] **8.3** Пауза: ESC → Continue, Main Menu, Exit; `Time<Virtual>::pause()` замораживает логику `[M]`
- [x] **8.4** `ui/minimap.rs` — 160px canvas: варбейсы, фабрики (по команде), роботы, скаут-крестик `[L]`
- [x] **8.5** `ui/hud.rs` — цвет ресурсов (красный/жёлтый/зелёный), +X/день от фабрик, счётчик фабрик `[M]`
- [x] **8.6** Панель юнита: HP-бар; мульти-выбор — состав шасси, средний HP, разброс приказов `[M]`
- [x] **8.7** Панель команд: кнопки 2×2 (Атака/Захват/Держать/Стоп) работают для всех выбранных `[M]`
- [ ] **8.8** Builder UI: 3D-превью робота при конфигурировании (render-to-texture) `[L]` *(отложено)*
- [x] **8.9** `audio/mod.rs` — `AudioSettings` ресурс, скелет для SFX (без .ogg файлов) `[M]`
- [ ] **8.10** Интеграция звуков: выстрелы (3 типа), взрыв, захват, постройка `[M]` *(нужны .ogg)*
- [ ] **8.11** Фоновая музыка: sci-fi ambient, crossfade при смене состояний `[S]` *(нужны .ogg)*
- [x] **8.12** Выбор сценария: пикер ◀/▶ в меню, сканирует `data/scenarios/*.ron`; 2 карты `[M]`
- [x] **8.13** Game Over → кнопки "Главное меню" / "Выход"; AppState переход `[S]`
- [x] **8.14** `RobotCommand::DestroyEnemyBase` — ядерный робот идёт к варбейсу врага, взрывается вплотную `[M]`
- [x] **8.15** Навигация через VisionRange: SeekAndCapture и DestroyEnemyBase исследуют карту пока цель не в видимости `[M]`
- [x] **8.16** Ядерный взрыв поражает структуры (Factory/Warbase), не только роботов `[S]`
- [x] **8.17** Устранены крэши: `try_insert` для всех `MovementTarget`-команд, `MuzzleFlash` `[S]`
- [x] **8.18** Время захвата унифицировано (10 сек базово, −30% с электроникой) `[S]`

**Критерии проверки:**
- Меню функционально и стилизовано
- Настройки сохраняются между сессиями
- Пауза работает (логика стоит, UI отвечает)
- Миникарта отражает реальное состояние
- Все звуки воспроизводятся, громкость регулируется
- Полный геймплей цикл с UI и аудио

**Плагины:** bevy_audio (встроен), опционально bevy_kira_audio для spatial audio

---

### Фаза 9: Сохранение + Локализация (8-12 дней)

**Цель:** Сохранение/загрузка состояния игры. UI на русском и английском.

**Зависимости:** Фаза 8

- [x] **9.1** `save/types.rs` — `SaveData`: все роботы, структуры, ресурсы, время, AI, позиция скаута `[L]`
- [x] **9.2** Сериализация: обход entity через queries → SaveData → RON (readable) `[L]`
- [x] **9.3** Десериализация: загрузка SaveData → despawn роботов → обновление структур → респавн `[L]`
- [x] **9.4** 3 слота сохранения + автосохранение; UI в меню паузы (Сохранить/Загрузить) `[M]`
- [x] **9.5** Автосохранение при смене игрового дня (`check_autosave` в FixedUpdate) `[S]`
- [x] **9.6** Версионирование формата сохранения (`SAVE_VERSION = 1`) `[S]`
- [x] **9.7** `LocalizationPlugin`: `Res<Localization>`, загрузка из `assets/locales/en.ron` и `ru.ron` `[M]`
- [x] **9.8** Ключи локализации в главном меню, меню паузы; кнопка "Продолжить" (из автосохранения) `[M]`
- [x] **9.9** Выбор языка RU/EN в меню паузы (без перезапуска) `[S]`

**Критерии проверки:**
- Сохранение → выход → загрузка → идентичное состояние
- 3 слота работают независимо
- Размер сохранения < 1MB
- RON-файлы читаемы человеком
- UI корректно на RU и EN
- Смена языка без перезапуска
- Unit-тесты: serialize↔deserialize round-trip, обработка повреждённого файла

**Плагины:** bincode (опционально для компактных сохранений)

---

### Фаза 10: Полировка + Релиз (14-20 дней)

**Цель:** Баланс, финальные ассеты, оптимизация, релизная сборка.

**Зависимости:** Все предыдущие фазы

- [ ] **10.1** Замена placeholder-мешей на финальные .glb модели `[L]`
- [ ] **10.2** PBR-материалы и освещение для изометрии `[M]`
- [ ] **10.3** Particle-эффекты: удары, взрывы, nuclear blast, захват, дым фабрик `[L]`
- [ ] **10.4** Анимации роботов по типу шасси: idle, move, attack, death `[L]`
- [ ] **10.5** Балансировка: плейтест + правка RON-конфигов `[L]`
- [ ] **10.6** Создать 2-3 карты сценариев с разными расстановками `[M]`
- [ ] **10.7** Оптимизация: LOD, пулинг entity, профилирование `[L]`
- [x] **10.8** Debug-инструменты за feature flag `debug_tools`, выключены в release `[S]`
- [x] **10.9** Справка по клавишам (in-game overlay, F1) `[S]`
- [ ] **10.10** Windows release build: `cargo build --release`, тест на чистой системе `[M]`
- [ ] **10.11** README, LICENSE `[S]`
- [x] **10.12** Edge-cases: застревание роботов (StuckDetector), пересчёт пути, дрожание камеры (lerp) `[L]`

**Исправления и доработки (вне плана):**
- [x] Кнопка «Новая игра» сбрасывает мир (`TriggerNewGame`) — роботы, ресурсы, время, ИИ, скаут
- [x] Панель «Юниты»: кнопка «✕ Снять выбор», сворачивается (collapsible)
- [x] Меню строительства автоматически закрывается при отходе от варбейса
- [x] Тип и производство структуры показываются в тултипе (всегда, не только в debug)
- [x] Редизайн экономики: `FactoryType::General` удалён; 12 нейтральных заводов (6 типов × 2); General — от варбейса (+5/день) + от каждого завода (+2/день)
- [x] Захват вражеского завода: сначала → Нейтральный, затем → Игрок (время удвоенное)

**Критерии проверки:**
- Полное прохождение с финальными ассетами и звуком
- 3 сценария играбельны
- 60 FPS при 50+ роботах
- Нет визуальных артефактов и placeholder-ассетов
- Сборка запускается на чистой Windows 11
- Нет крашей при часовой сессии

---

## 5. Критический путь

```
Фаза 0 ──→ Фаза 1 ──→ Фаза 2 ──→ Фаза 3 ──→ Фаза 4 ──→ Фаза 5
                                                               │
                                                               ▼
                         Фаза 8 ◀── Фаза 7 ◀── Фаза 6 ◀──────┘
                           │
                           ▼
                         Фаза 9 ──→ Фаза 10
```

**Критический путь:** 0 → 1 → 3 → 5 → 6 → 7 → 8 → 10

Фаза 2 может частично перекрываться с Фазой 1. Фаза 4 — с Фазой 3.

---

## 6. Интеграция плагинов

| Плагин | Фаза | Назначение |
|--------|------|------------|
| bevy 0.18 | 0 | Движок |
| serde + ron | 0 | Конфиги и сериализация |
| bevy_egui 0.32 | 0 | Debug overlay, позже Builder UI |
| bevy_common_assets 0.15 | 1 | RON как Bevy-ассеты |
| bevy_picking 0.20 | 4 | Выбор роботов/структур raycast |
| bevy_rapier3d 0.28 | 5 (опц.) | Физика ракет, коллизии |
| bevy_asset_loader 0.21 | 8 | Asset loading states |
| bincode | 9 (опц.) | Компактные сохранения |

---

## 7. Итого

| Фаза | Название | Оценка (дни) |
|------|----------|--------------|
| 0 | Инициализация | 3-5 |
| 1 | Ядро + Скаут | 8-12 |
| 2 | Структуры + Карта | 8-12 |
| 3 | Роботы + Движение | 12-18 |
| 4 | Команды + Взаимодействие | 10-14 |
| 5 | Боевая система | 12-16 |
| 6 | Экономика + Строительство | 14-18 |
| 7 | ИИ противника | 14-20 |
| 8 | UI + Аудио | 10-14 |
| 9 | Сохранение + Локализация | 8-12 |
| 10 | Полировка + Релиз | 14-20 |
| | **Итого (solo)** | **~115-161 дней (22-32 недели)** |

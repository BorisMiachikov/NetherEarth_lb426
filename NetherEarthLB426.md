ТЗ: Nether Earth LB426
Целевой стек: Rust (Stable), Bevy 0.15+, Serde (RON)

1. Концепция проекта
Современное переосмысление классической RTS Nether Earth. Изометрическая 3D стратегия. Игрок управляет летающим модулем, строит кастомизируемых роботов и ведет войну за ресурсы (фабрики) с целью уничтожения вражеской базы.

2. Игровые механики (Core Loop)
2.1 Командный модуль (Scout)
Движение: 6 степеней свободы (WASD + Q/E для высоты). Или мышкой, левая клавиша выбрать, правая вверх.

Взаимодействие: При нахождении над своей Warbase или захваченной фабрикой — открытие меню производства.

Приказы: Выдача команд роботам через систему Raycast (выбор робота -> выбор цели/точки).

Manual Control: Возможность «вселиться» в робота (прямое управление огнем и движением).

2.2 Модульная система роботов
Каждый робот собирается из следующих компонентов:

Chassis (Шасси): Определяет тип движения (колеса, гусеницы, ноги, антиграв), скорость и проходимость.

Weapons (Оружие): До 3-х слотов.

Cannon: Высокий урон, средняя дистанция.

Missiles: Дальний бой, самонаведение.

Phasers: Высокая скорострельность, малая дистанция.

Electronics (Электроника): Улучшает радиус обнаружения и точность ИИ.

Nuclear (Ядерный заряд): Спецмодуль для уничтожения Warbase.

2.3 Экономика и Захват
Ресурсы: 7 категорий ресурсов, привязанных к типам модулей.

Фабрики: Захваченная фабрика определенного типа дает прирост соответствующего ресурса каждые 30 секунд.

Механика захвата: Робот с приказом "Capture" должен дойти до входа в здание. Время захвата зависит от типа шасси и наличия электроники.

3. Техническая реализация (Bevy ECS)
3.1 Основные компоненты (Data Structures)
Rust
// Типы приказов
pub enum Order {
    Idle,
    MoveTo(Vec3),
    Attack(Entity),
    Capture(Entity),
    SearchAndDestroy,
}

#[derive(Component)]
pub struct Robot {
    pub health: f32,
    pub owner: PlayerId, // 0 - Player, 1 - AI
}

#[derive(Component)]
pub struct MovementConfig {
    pub speed: f32,
    pub turn_speed: f32,
    pub move_type: LocomotionType, // Wheels, Tracks, Leg, Hover
}
3.2 Ключевые системы (Systems)
production_system: Сбор ресурсов со всех Entity с меткой Factory и OwnedBy(Player).

ai_behavior_system: Обработка очереди приказов (CommandQueue). Использование упрощенного A* для навигации по сетке.

combat_system: Проверка дистанции между роботами разных фракций, расчет урона и спавн эффектов выстрелов.

capture_logic_system: Таймер захвата при коллизии робота с триггером входа в здание.

4. Данные (Data-Driven Design)
Все характеристики модулей должны быть вынесены в файлы настроек (assets/data/balance.ron), чтобы геймдизайнер мог менять баланс без пересборки проекта:

Фрагмент кода
(
    chassis: [
        (id: "Bipod", cost: 20, speed: 1.5, weight: 10),
        (id: "Tracks", cost: 40, speed: 1.0, weight: 30),
    ],
    weapons: [
        (id: "Cannon", damage: 15.0, range: 10.0, reload: 1.2),
    ]
)
6. Общая структура проекта
Цель: чёткое разделение по доменам (feature-based), а не по типам файлов.
/src — исходный код
├── main.rs
├── lib.rs

├── /app                # инициализация приложения
│   ├── mod.rs
│   ├── plugin.rs
│   └── state.rs

├── /core               # базовые вещи (общие для всей игры)
│   ├── mod.rs
│   ├── time.rs
│   ├── transform.rs
│   ├── health.rs
│   ├── team.rs
│   ├── resources.rs              # Глобальные ресурсы (GameTime, Settings)
│   └── events.rs

├── /player             # летающий модуль игрока
│   ├── mod.rs
│   ├── components.rs
│   ├── systems.rs
│   └── input.rs

├── /robot              # роботы
│   ├── mod.rs
│   ├── components.rs
│   ├── bundle.rs
│   ├── builder.rs
│   └── systems.rs

├── /ai                 # AI (FSM + Utility)
│   ├── mod.rs
│   ├── state.rs
│   ├── utility.rs
│   ├── scoring.rs
│   └── systems.rs

├── /command            # система приказов
│   ├── mod.rs
│   ├── command.rs
│   ├── queue.rs
│   └── systems.rs

├── /movement           # движение и навигация
│   ├── mod.rs
│   ├── velocity.rs
│   ├── pathfinding.rs
│   └── steering.rs

├── /combat             # бой
│   ├── mod.rs
│   ├── weapon.rs
│   ├── damage.rs
│   └── systems.rs

├── /economy            # ресурсы
│   ├── mod.rs
│   ├── resource.rs
│   ├── production.rs
│   └── systems.rs

├── /structure          # фабрики и базы
│   ├── mod.rs
│   ├── factory.rs
│   ├── warbase.rs
│   └── capture.rs

├── /map                # карта
│   ├── mod.rs
│   ├── loader.rs
│   ├── grid.rs
│   └── collision.rs

├── /ui                 # интерфейс
│   ├── mod.rs
│   ├── hud.rs
│   ├── minimap.rs
│   ├── builder_ui.rs
│   └── menu.rs

├── /camera             # камера
│   ├── mod.rs
│   └── systems.rs

├── /audio              # звук
│   ├── mod.rs
│   └── systems.rs

├── /save               # сохранения
│   ├── mod.rs
│   ├── serialize.rs
│   └── systems.rs

└── /debug              # отладка
    ├── mod.rs
    ├── gizmos.rs
    └── overlay.rs

/assets — игровые ассеты
├── /models
│   ├── player_module/
│   ├── robots/
│   ├── factories/
│   └── environment/
├── /textures
│   ├── environment/
│   ├── robots/
│   └── ui/
├── /materials
│   └── pbr/
├── /audio
│   ├── music/
│   └── sfx/
├── ui/
│   ├── fonts/
│   ├── icons/
│   └── sprites/
├── /shaders
└── /scenes

/configs — баланс и настройки (важно)
├── game.ron
├── ai.ron
├── economy.ron

├── /modules
│   ├── chassis.ron
│   ├── weapons.ron
│   ├── electronics.ron
│   └── nuclear.ron

├── /units
│   └── robot_defaults.ron

├── /structures
│   ├── factories.ron
│   └── warbases.ron
└── /ui
    └── layout.ron


/data — игровые данные (карты, сценарии)
├── /maps
│   ├── map_01.ron
│   └── map_02.ron

├── /scenarios
│   ├── mission_01.ron
│   └── skirmish.ron


/scripts — утилиты разработки
├── build_release.sh
├── run_dev.sh
└── convert_assets.py


5. План разработки (Roadmap)
MVP 0.1: Настройка Bevy, полет модуля, отображение сетки ландшафта.

Alpha 0.2: Спавн робота из базовых префабов, простое движение по клику.

Beta 0.3: Меню строительства (UI), система ресурсов, стрельба.

Release 1.0: ИИ противника, захват всех типов фабрик, звуковое сопровождение.
use bevy::prelude::*;

use crate::robot::{
    builder::RobotBlueprint,
    components::{ChassisType, WeaponType},
    registry::ModuleRegistry,
};

/// Выбор blueprint для постройки по детерминированному счётчику.
/// Ротирует через несколько тактических конфигураций.
pub fn select_blueprint(counter: u32, _registry: &ModuleRegistry) -> RobotBlueprint {
    match counter % 10 {
        // 30% — Гусеницы + Пушка (тяжёлый атакующий)
        0 | 1 | 2 => RobotBlueprint::new(ChassisType::Tracks).with_weapon(WeaponType::Cannon),
        // 20% — Колёса + Пушка (быстрый базовый)
        3 | 4 => RobotBlueprint::new(ChassisType::Wheels).with_weapon(WeaponType::Cannon),
        // 20% — Бипод + Пушка×2 (сдвоенный огонь)
        5 | 6 => RobotBlueprint::new(ChassisType::Bipod)
            .with_weapon(WeaponType::Cannon)
            .with_weapon(WeaponType::Cannon),
        // 20% — Гусеницы + Ракета (дальнобойный)
        7 | 8 => RobotBlueprint::new(ChassisType::Tracks).with_weapon(WeaponType::Missile),
        // 10% — АнтиГрав + Ракета + Ядерный (диверсант)
        9 => RobotBlueprint::new(ChassisType::AntiGrav)
            .with_weapon(WeaponType::Missile)
            .with_nuclear(),
        _ => unreachable!(),
    }
}

/// Ядерный blueprint — только когда у ИИ достаточно фабрик.
pub fn select_nuclear_blueprint() -> RobotBlueprint {
    RobotBlueprint::new(ChassisType::AntiGrav)
        .with_weapon(WeaponType::Missile)
        .with_nuclear()
}

/// Приоритет захвата фабрики: выше для ближних и нейтральных.
pub fn capture_priority(factory_pos: Vec3, warbase_pos: Vec3, is_neutral: bool) -> f32 {
    let dist = factory_pos.xz().distance(warbase_pos.xz());
    let dist_score = 100.0 / (dist + 10.0);
    let bonus = if is_neutral { 1.5 } else { 1.0 };
    dist_score * bonus
}

/// Соотношение сил: > 1.0 = у врага больше роботов.
pub fn threat_ratio(enemy_count: u32, friendly_count: u32) -> f32 {
    if friendly_count == 0 {
        return 10.0;
    }
    enemy_count as f32 / friendly_count as f32
}

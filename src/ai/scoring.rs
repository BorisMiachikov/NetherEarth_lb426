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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::robot::registry::{ChassisDef, ElectronicsDef, NuclearDef};

    fn stub_registry() -> ModuleRegistry {
        ModuleRegistry {
            chassis: vec![],
            weapons: vec![],
            electronics: ElectronicsDef {
                accuracy_bonus: 0.0,
                fire_rate_bonus: 0.0,
                radar_range: 10.0,
                capture_time_reduction: 0.1,
                cost_electronics: 1,
                cost_general: 1,
            },
            nuclear: NuclearDef {
                blast_radius: 5.0,
                detonation_delay: 3.0,
                cost_nuclear: 5,
                cost_general: 2,
            },
        }
    }

    #[test]
    fn select_blueprint_cycles() {
        let reg = stub_registry();
        for i in 0..10u32 {
            let bp = select_blueprint(i, &reg);
            assert!(bp.validate().is_ok(), "blueprint {i} invalid");
        }
    }

    #[test]
    fn select_blueprint_deterministic() {
        let reg = stub_registry();
        let a = select_blueprint(3, &reg);
        let b = select_blueprint(3, &reg);
        assert_eq!(a.chassis, b.chassis);
    }

    #[test]
    fn capture_priority_neutral_bonus() {
        let wb = Vec3::ZERO;
        let factory = Vec3::new(5.0, 0.0, 0.0);
        let p_neutral  = capture_priority(factory, wb, true);
        let p_enemy    = capture_priority(factory, wb, false);
        assert!(p_neutral > p_enemy);
    }

    #[test]
    fn threat_ratio_no_friendlies() {
        assert_eq!(threat_ratio(5, 0), 10.0);
    }

    #[test]
    fn threat_ratio_equal_forces() {
        assert!((threat_ratio(4, 4) - 1.0).abs() < 1e-6);
    }
}

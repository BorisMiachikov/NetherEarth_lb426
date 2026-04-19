use super::{
    components::{ChassisType, WeaponData, WeaponType},
    registry::ModuleRegistry,
};

/// Описание робота перед спавном. Валидируется перед постройкой.
#[derive(Debug, Clone)]
pub struct RobotBlueprint {
    pub chassis: ChassisType,
    pub weapons: Vec<WeaponType>,
    pub has_electronics: bool,
    pub has_nuclear: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    TooManyWeapons,
    NoWeapons,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::TooManyWeapons => write!(f, "Максимум 3 оружия"),
            ValidationError::NoWeapons => write!(f, "Нужно хотя бы 1 оружие"),
        }
    }
}

#[derive(Debug)]
pub struct BuildCost {
    /// (ResourceType-строка, количество)
    pub items: Vec<(String, u32)>,
    pub general: u32,
    /// Время постройки в секундах.
    pub build_time: f32,
}

impl RobotBlueprint {
    pub fn new(chassis: ChassisType) -> Self {
        Self {
            chassis,
            weapons: vec![],
            has_electronics: false,
            has_nuclear: false,
        }
    }

    pub fn with_weapon(mut self, wt: WeaponType) -> Self {
        self.weapons.push(wt);
        self
    }

    pub fn with_nuclear(mut self) -> Self {
        self.has_nuclear = true;
        self
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.weapons.is_empty() {
            return Err(ValidationError::NoWeapons);
        }
        if self.weapons.len() > 3 {
            return Err(ValidationError::TooManyWeapons);
        }
        Ok(())
    }

    /// Расчёт стоимости постройки по реестру модулей.
    pub fn cost(&self, registry: &ModuleRegistry) -> BuildCost {
        let mut items: Vec<(String, u32)> = vec![];
        let mut general: u32 = 0;
        let mut total_cost: f32 = 0.0;

        // Шасси
        if let Some(c) = registry.chassis(self.chassis) {
            items.push(("Chassis".to_string(), c.cost_chassis));
            general += c.cost_general;
            total_cost += c.cost_chassis as f32;
        }

        // Оружие
        for wt in &self.weapons {
            if let Some(w) = registry.weapon(*wt) {
                items.push((format!("{wt:?}"), w.cost_resource));
                general += w.cost_general;
                total_cost += w.cost_resource as f32;
            }
        }

        // Электроника
        if self.has_electronics {
            items.push(("Electronics".to_string(), registry.electronics.cost_electronics));
            general += registry.electronics.cost_general;
            total_cost += registry.electronics.cost_electronics as f32;
        }

        // Ядерный
        if self.has_nuclear {
            items.push(("Nuclear".to_string(), registry.nuclear.cost_nuclear));
            general += registry.nuclear.cost_general;
            total_cost += registry.nuclear.cost_nuclear as f32;
        }

        items.push(("General".to_string(), general));
        let build_time = total_cost * 0.5;

        BuildCost {
            items,
            general,
            build_time,
        }
    }

    /// Данные оружия для слотов.
    pub fn weapon_data(&self, registry: &ModuleRegistry) -> [Option<WeaponData>; 3] {
        let mut slots: [Option<WeaponData>; 3] = [None, None, None];
        for (i, wt) in self.weapons.iter().enumerate().take(3) {
            if let Some(def) = registry.weapon(*wt) {
                slots[i] = Some(WeaponData {
                    weapon_type: def.weapon_type,
                    damage: def.damage,
                    range: def.range,
                    reload_time: def.reload_time,
                });
            }
        }
        slots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::robot::registry::{ChassisDef, ElectronicsDef, NuclearDef, WeaponDef};

    fn mock_registry() -> ModuleRegistry {
        ModuleRegistry {
            chassis: vec![ChassisDef {
                chassis_type: ChassisType::Wheels,
                base_hp: 50.0,
                speed: 1.2,
                mobility: 0.6,
                can_fly: false,
                cost_chassis: 15,
                cost_general: 5,
            }],
            weapons: vec![
                WeaponDef {
                    weapon_type: WeaponType::Cannon,
                    damage: 15.0,
                    range: 10.0,
                    reload_time: 1.2,
                    cost_resource: 10,
                    cost_general: 5,
                },
                WeaponDef {
                    weapon_type: WeaponType::Missile,
                    damage: 45.0,
                    range: 30.0,
                    reload_time: 3.0,
                    cost_resource: 25,
                    cost_general: 10,
                },
            ],
            electronics: ElectronicsDef {
                accuracy_bonus: 0.3,
                fire_rate_bonus: 0.2,
                radar_range: 20.0,
                capture_time_reduction: 0.3,
                cost_electronics: 20,
                cost_general: 10,
            },
            nuclear: NuclearDef {
                blast_radius: 8.0,
                detonation_delay: 2.0,
                cost_nuclear: 50,
                cost_general: 30,
            },
        }
    }

    #[test]
    fn valid_blueprint_passes() {
        let bp = RobotBlueprint::new(ChassisType::Wheels).with_weapon(WeaponType::Cannon);
        assert!(bp.validate().is_ok());
    }

    #[test]
    fn no_weapons_fails() {
        let bp = RobotBlueprint::new(ChassisType::Wheels);
        assert_eq!(bp.validate(), Err(ValidationError::NoWeapons));
    }

    #[test]
    fn too_many_weapons_fails() {
        let bp = RobotBlueprint::new(ChassisType::Wheels)
            .with_weapon(WeaponType::Cannon)
            .with_weapon(WeaponType::Cannon)
            .with_weapon(WeaponType::Cannon)
            .with_weapon(WeaponType::Cannon);
        assert_eq!(bp.validate(), Err(ValidationError::TooManyWeapons));
    }

    #[test]
    fn cost_calculation() {
        let reg = mock_registry();
        let bp = RobotBlueprint::new(ChassisType::Wheels).with_weapon(WeaponType::Cannon);
        let cost = bp.cost(&reg);
        // chassis=15 + cannon=10 → total_cost=25 → build_time=12.5
        assert!((cost.build_time - 12.5).abs() < 0.01);
        assert_eq!(cost.general, 10); // 5 + 5
    }

    #[test]
    fn stats_recalc() {
        let reg = mock_registry();
        let bp = RobotBlueprint::new(ChassisType::Wheels)
            .with_weapon(WeaponType::Cannon)
            .with_weapon(WeaponType::Missile);
        // max_hp = base_hp + sum(module_weight) * 2 = 50 + (10+25)*2 = 120
        let chassis_def = reg.chassis(ChassisType::Wheels).unwrap();
        let weapon_data = bp.weapon_data(&reg);
        let slots_weight: f32 = weapon_data
            .iter()
            .flatten()
            .map(|w| match w.weapon_type {
                WeaponType::Cannon => 10.0,
                WeaponType::Missile => 25.0,
                WeaponType::Phasers => 30.0,
            })
            .sum();
        let max_hp = chassis_def.base_hp + slots_weight * 2.0;
        assert!((max_hp - 120.0).abs() < 0.01);
    }
}

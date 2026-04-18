use std::{collections::HashMap, ops::{Deref, DerefMut}};

use bevy::prelude::*;

// ResourceType определён в core::events и ре-экспортируется здесь.
pub use crate::core::events::ResourceType;

/// Ресурсы игрока. Хранит запасы по каждому типу.
#[derive(Resource, Debug, Clone)]
pub struct PlayerResources {
    pub stocks: HashMap<ResourceType, i32>,
}

/// Ресурсы ИИ — зеркальный аналог PlayerResources.
#[derive(Resource, Debug, Clone)]
pub struct EnemyResources(pub PlayerResources);

impl Deref for EnemyResources {
    type Target = PlayerResources;
    fn deref(&self) -> &PlayerResources { &self.0 }
}
impl DerefMut for EnemyResources {
    fn deref_mut(&mut self) -> &mut PlayerResources { &mut self.0 }
}

impl PlayerResources {
    pub fn with_starting_values() -> Self {
        let mut stocks = HashMap::new();
        stocks.insert(ResourceType::General, 50);
        stocks.insert(ResourceType::Chassis, 20);
        stocks.insert(ResourceType::Cannon, 15);
        stocks.insert(ResourceType::Missile, 10);
        stocks.insert(ResourceType::Phasers, 10);
        stocks.insert(ResourceType::Electronics, 10);
        stocks.insert(ResourceType::Nuclear, 5);
        Self { stocks }
    }

    pub fn get(&self, rt: ResourceType) -> i32 {
        *self.stocks.get(&rt).unwrap_or(&0)
    }

    pub fn add(&mut self, rt: ResourceType, amount: i32) {
        *self.stocks.entry(rt).or_insert(0) += amount;
    }

    pub fn spend(&mut self, rt: ResourceType, amount: i32) -> bool {
        let current = self.get(rt);
        if current >= amount {
            self.stocks.insert(rt, current - amount);
            true
        } else {
            false
        }
    }

    /// Проверяет, хватает ли ресурсов на постройку по BuildCost.items.
    pub fn can_afford_cost(&self, cost: &crate::robot::builder::BuildCost) -> bool {
        for (name, amount) in &cost.items {
            if *amount == 0 { continue; }
            if let Some(rt) = resource_type_from_name(name) {
                if self.get(rt) < *amount as i32 {
                    return false;
                }
            }
        }
        true
    }

    /// Списывает ресурсы по BuildCost.items. Вызывать только после can_afford_cost.
    pub fn spend_cost(&mut self, cost: &crate::robot::builder::BuildCost) {
        for (name, amount) in &cost.items {
            if *amount == 0 { continue; }
            if let Some(rt) = resource_type_from_name(name) {
                self.spend(rt, *amount as i32);
            }
        }
    }
}

fn resource_type_from_name(name: &str) -> Option<ResourceType> {
    match name {
        "Chassis"     => Some(ResourceType::Chassis),
        "Cannon"      => Some(ResourceType::Cannon),
        "Missile"     => Some(ResourceType::Missile),
        "Phasers"     => Some(ResourceType::Phasers),
        "Electronics" => Some(ResourceType::Electronics),
        "Nuclear"     => Some(ResourceType::Nuclear),
        "General"     => Some(ResourceType::General),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_resources_initialized() {
        let res = PlayerResources::with_starting_values();
        assert_eq!(res.get(ResourceType::General), 50);
        assert_eq!(res.get(ResourceType::Nuclear), 5);
    }

    #[test]
    fn spend_returns_false_if_insufficient() {
        let mut res = PlayerResources::with_starting_values();
        assert!(!res.spend(ResourceType::Nuclear, 100));
        assert_eq!(res.get(ResourceType::Nuclear), 5); // не изменилось
    }

    #[test]
    fn add_and_spend() {
        let mut res = PlayerResources::with_starting_values();
        res.add(ResourceType::Cannon, 10);
        assert_eq!(res.get(ResourceType::Cannon), 25);
        assert!(res.spend(ResourceType::Cannon, 20));
        assert_eq!(res.get(ResourceType::Cannon), 5);
    }
}

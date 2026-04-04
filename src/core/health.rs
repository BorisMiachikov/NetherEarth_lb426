use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    pub fn apply_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_clamps_to_zero() {
        let mut h = Health::new(100.0);
        h.apply_damage(150.0);
        assert_eq!(h.current, 0.0);
        assert!(!h.is_alive());
    }

    #[test]
    fn heal_clamps_to_max() {
        let mut h = Health::new(100.0);
        h.apply_damage(50.0);
        h.heal(200.0);
        assert_eq!(h.current, 100.0);
    }
}

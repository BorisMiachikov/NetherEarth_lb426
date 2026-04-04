use bevy::prelude::*;

/// Маркер: эта сущность — Warbase (главная база).
/// Уничтожается только ядерным зарядом.
#[derive(Component)]
pub struct Warbase;

/// Заглушка очереди производства (заполняется в Фазе 5).
#[derive(Component, Default)]
pub struct ProductionQueue;

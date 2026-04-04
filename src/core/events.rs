use bevy::prelude::*;

use super::{health::Health, team::Team};

/// Урон нанесён сущности. Триггерить: `commands.trigger(EntityDamaged { entity, amount, attacker })`.
#[derive(Event, Debug)]
pub struct EntityDamaged {
    pub entity: Entity,
    pub amount: f32,
    pub attacker: Option<Entity>,
}

/// Сущность уничтожена (HP достигло 0).
#[derive(Event, Debug)]
pub struct EntityDestroyed {
    pub entity: Entity,
    pub team: Team,
}

/// Структура захвачена.
#[derive(Event, Debug)]
pub struct StructureCaptured {
    pub structure: Entity,
    pub new_owner: Team,
    pub old_owner: Team,
}

/// Ресурс изменился.
#[derive(Event, Debug)]
pub struct ResourceChanged {
    pub team: Team,
    pub resource_type: ResourceType,
    pub delta: i32,
    pub new_total: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    General,
    Chassis,
    Cannon,
    Missile,
    Phasers,
    Electronics,
    Nuclear,
}

/// Observer: обрабатывает EntityDamaged → обновляет Health → генерирует EntityDestroyed.
pub fn on_entity_damaged(
    trigger: On<EntityDamaged>,
    mut query: Query<(&mut Health, &Team)>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    if let Ok((mut health, &team)) = query.get_mut(ev.entity) {
        health.apply_damage(ev.amount);
        if !health.is_alive() {
            commands.trigger(EntityDestroyed {
                entity: ev.entity,
                team,
            });
        }
    }
}

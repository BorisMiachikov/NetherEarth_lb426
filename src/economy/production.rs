use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    core::{events::ResourceChanged, time::GameTime, Team},
    structure::{capture::Capturable, factory::Factory, warbase::Warbase, FactType},
};

use super::resource::{EnemyResources, PlayerResources, ResourceType};

/// Последний день, когда производилось накопление ресурсов.
#[derive(Resource, Default)]
pub struct LastProductionDay(pub u32);

/// General-ресурс в день от одного варбейса.
pub const WARBASE_GENERAL_PER_DAY: i32 = 5;
/// General-бонус от каждой захваченной фабрики.
pub const FACTORY_GENERAL_BONUS: i32 = 2;
/// Специфический ресурс от каждой захваченной фабрики.
pub const FACTORY_SPECIFIC_PER_DAY: i32 = 5;

/// Конвертация типа фабрики в тип ресурса.
pub fn factory_type_to_resource(ft: FactType) -> ResourceType {
    match ft {
        FactType::Chassis     => ResourceType::Chassis,
        FactType::Cannon      => ResourceType::Cannon,
        FactType::Missile     => ResourceType::Missile,
        FactType::Phasers     => ResourceType::Phasers,
        FactType::Electronics => ResourceType::Electronics,
        FactType::Nuclear     => ResourceType::Nuclear,
    }
}

/// Начисляет ресурсы раз в игровой день:
/// - Варбейс игрока: +5 General
/// - Каждая захваченная фабрика: +5 специфического + +2 General
pub fn tick_production(
    game_time: Res<GameTime>,
    mut last_day: ResMut<LastProductionDay>,
    factories: Query<(&FactType, &Team), (With<Factory>, With<Capturable>)>,
    warbases: Query<&Team, With<Warbase>>,
    mut player_res: ResMut<PlayerResources>,
    mut enemy_res: ResMut<EnemyResources>,
    mut commands: Commands,
) {
    if game_time.game_day <= last_day.0 {
        return;
    }
    last_day.0 = game_time.game_day;

    let mut deltas: HashMap<ResourceType, i32> = HashMap::new();

    // Варбейс: +5 General за каждый варбейс игрока
    for team in &warbases {
        if *team == Team::Player {
            *deltas.entry(ResourceType::General).or_insert(0) += WARBASE_GENERAL_PER_DAY;
        }
    }

    // Фабрики: +5 специфического + +2 General
    for (factory_type, team) in &factories {
        if *team != Team::Player {
            continue;
        }
        let rt = factory_type_to_resource(*factory_type);
        *deltas.entry(rt).or_insert(0) += FACTORY_SPECIFIC_PER_DAY;
        *deltas.entry(ResourceType::General).or_insert(0) += FACTORY_GENERAL_BONUS;
    }

    for (rt, delta) in deltas {
        player_res.add(rt, delta);
        let new_total = player_res.get(rt);
        commands.trigger(ResourceChanged {
            team: Team::Player,
            resource_type: rt,
            delta,
            new_total,
        });
    }

    // ── Ресурсы ИИ (та же экономика) ──────────────────────────────────────
    let mut enemy_deltas: HashMap<ResourceType, i32> = HashMap::new();

    for team in &warbases {
        if *team == Team::Enemy {
            *enemy_deltas.entry(ResourceType::General).or_insert(0) += WARBASE_GENERAL_PER_DAY;
        }
    }
    for (factory_type, team) in &factories {
        if *team != Team::Enemy {
            continue;
        }
        let rt = factory_type_to_resource(*factory_type);
        *enemy_deltas.entry(rt).or_insert(0) += FACTORY_SPECIFIC_PER_DAY;
        *enemy_deltas.entry(ResourceType::General).or_insert(0) += FACTORY_GENERAL_BONUS;
    }
    for (rt, delta) in enemy_deltas {
        enemy_res.add(rt, delta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_type_mapping_complete() {
        use FactType::*;
        let types = [Chassis, Cannon, Missile, Phasers, Electronics, Nuclear];
        for ft in types {
            let _ = factory_type_to_resource(ft);
        }
    }
}

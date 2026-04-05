use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    core::{events::ResourceChanged, time::GameTime, Team},
    structure::{capture::Capturable, factory::Factory, FactType},
};

use super::resource::{PlayerResources, ResourceType};

/// Последний день, когда производилось накопление ресурсов.
#[derive(Resource, Default)]
pub struct LastProductionDay(pub u32);

/// Конвертация типа фабрики в тип ресурса.
pub fn factory_type_to_resource(ft: FactType) -> ResourceType {
    match ft {
        FactType::General => ResourceType::General,
        FactType::Chassis => ResourceType::Chassis,
        FactType::Cannon => ResourceType::Cannon,
        FactType::Missile => ResourceType::Missile,
        FactType::Phasers => ResourceType::Phasers,
        FactType::Electronics => ResourceType::Electronics,
        FactType::Nuclear => ResourceType::Nuclear,
    }
}

/// Начисляет ресурсы раз в игровой день (+5 специфического +2 General за каждую захваченную фабрику).
pub fn tick_production(
    game_time: Res<GameTime>,
    mut last_day: ResMut<LastProductionDay>,
    factories: Query<(&FactType, &Team), (With<Factory>, With<Capturable>)>,
    mut player_res: ResMut<PlayerResources>,
    mut commands: Commands,
) {
    if game_time.game_day <= last_day.0 {
        return;
    }
    last_day.0 = game_time.game_day;

    // Агрегировать производство по типу ресурса
    let mut deltas: HashMap<ResourceType, i32> = HashMap::new();

    for (factory_type, team) in &factories {
        if *team != Team::Player {
            continue;
        }
        let rt = factory_type_to_resource(*factory_type);
        *deltas.entry(rt).or_insert(0) += 5;
        *deltas.entry(ResourceType::General).or_insert(0) += 2;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_type_mapping_complete() {
        use FactType::*;
        let types = [General, Chassis, Cannon, Missile, Phasers, Electronics, Nuclear];
        for ft in types {
            let _ = factory_type_to_resource(ft);
        }
    }
}

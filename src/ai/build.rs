use bevy::prelude::*;

use crate::{
    core::Team,
    economy::EnemyResources,
    robot::registry::ModuleRegistry,
    structure::{
        factory::Factory,
        warbase::{ProductionQueue, Warbase},
    },
};

use super::{
    scoring::{select_blueprint, select_nuclear_blueprint},
    state::{AICommander, GameResult},
};

/// Периодически добавляет роботов в очередь постройки вражеского варбейса.
pub fn ai_build_robots(
    time: Res<Time>,
    mut ai: ResMut<AICommander>,
    mut warbases: Query<(&mut ProductionQueue, &Transform, &Team), With<Warbase>>,
    registry: Res<ModuleRegistry>,
    factories: Query<&Team, With<Factory>>,
    result: Res<GameResult>,
    mut enemy_res: ResMut<EnemyResources>,
) {
    if result.outcome.is_some() {
        return;
    }

    ai.build_timer += time.delta_secs();
    if ai.build_timer < ai.config.build_interval {
        return;
    }
    ai.build_timer = 0.0;

    let ai_factory_count = factories.iter().filter(|t| **t == Team::Enemy).count() as u32;

    let use_nuclear = ai_factory_count >= ai.config.nuclear_factory_threshold
        && ai.decision_counter % 7 == 0;

    let blueprint = if use_nuclear {
        select_nuclear_blueprint()
    } else {
        select_blueprint(ai.decision_counter, &registry)
    };
    ai.decision_counter = ai.decision_counter.wrapping_add(1);

    if blueprint.validate().is_err() {
        return;
    }
    let build_cost = blueprint.cost(&registry);

    if !enemy_res.can_afford_cost(&build_cost) {
        return;
    }

    for (mut queue, _, team) in &mut warbases {
        if *team != Team::Enemy || queue.queue.len() >= 3 {
            continue;
        }
        enemy_res.spend_cost(&build_cost);
        queue.enqueue(blueprint.clone(), build_cost.build_time);
        ai.robots_built += 1;
        debug!("AI: в очередь {:?} (всего построено: {})", blueprint.chassis, ai.robots_built);
        break;
    }
}

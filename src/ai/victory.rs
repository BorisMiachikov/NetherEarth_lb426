use bevy::prelude::*;

use crate::{
    core::Team,
    structure::{factory::Factory, warbase::Warbase},
};

use super::state::{GameOutcome, GameResult};

/// Проверяет уничтожение варбейсов → устанавливает GameResult.
pub fn check_victory_defeat(
    warbases: Query<&Team, With<Warbase>>,
    mut result: ResMut<GameResult>,
    game_time: Res<crate::core::time::GameTime>,
    factories: Query<&Team, With<Factory>>,
) {
    if result.outcome.is_some() {
        return;
    }

    let player_alive = warbases.iter().any(|t| *t == Team::Player);
    let enemy_alive = warbases.iter().any(|t| *t == Team::Enemy);

    if !player_alive || !enemy_alive {
        let player_factories = factories.iter().filter(|t| **t == Team::Player).count() as u32;
        let enemy_factories = factories.iter().filter(|t| **t == Team::Enemy).count() as u32;

        result.outcome = Some(if !enemy_alive {
            GameOutcome::PlayerWin
        } else {
            GameOutcome::PlayerLose
        });
        result.game_days = game_time.game_day;
        result.player_factories = player_factories;
        result.enemy_factories = enemy_factories;

        info!("Игра окончена: {:?} на день {}", result.outcome, game_time.game_day);
    }
}

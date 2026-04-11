pub mod events;
pub mod health;
pub mod resources;
pub mod team;
pub mod time;

use bevy::prelude::*;

use events::on_entity_damaged;
use resources::{add_pause_sync_systems, load_game_config};
use time::{tick_game_time, GameTime};

use crate::app::state::AppState;

pub use events::{EntityDamaged, EntityDestroyed, ResourceChanged, ResourceType, StructureCaptured};
pub use health::Health;
pub use resources::GameConfig;
pub use team::Team;
pub use time::GameTime as GameTimeClock;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        let config = load_game_config();
        let game_time = GameTime {
            seconds_per_day: config.seconds_per_day,
            ..Default::default()
        };

        app.insert_resource(config)
            .insert_resource(game_time)
            .add_observer(on_entity_damaged)
            .add_systems(FixedUpdate, tick_game_time.run_if(in_state(AppState::Playing)));

        add_pause_sync_systems(app);
    }
}

pub mod events;
pub mod health;
pub mod team;
pub mod time;

use bevy::prelude::*;

use events::on_entity_damaged;
use time::{tick_game_time, GameTime};

pub use events::{EntityDamaged, EntityDestroyed, ResourceChanged, ResourceType, StructureCaptured};
pub use health::Health;
pub use team::Team;
pub use time::GameTime as GameTimeClock;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameTime>()
            .add_observer(on_entity_damaged)
            .add_systems(FixedUpdate, tick_game_time);
    }
}

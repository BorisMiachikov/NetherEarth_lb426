pub mod io;
pub mod systems;
pub mod types;

use bevy::prelude::*;

use systems::{
    apply_pending_load, check_autosave, on_trigger_autosave, on_trigger_load,
    on_trigger_load_autosave, on_trigger_new_game, on_trigger_save, LastAutoSaveDay, PendingLoad,
};

use crate::app::state::AppState;

pub use systems::{TriggerLoad, TriggerLoadAutosave, TriggerNewGame, TriggerSave};
pub use types::SAVE_SLOT_COUNT;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PendingLoad>()
            .init_resource::<LastAutoSaveDay>()
            .add_observer(on_trigger_save)
            .add_observer(on_trigger_autosave)
            .add_observer(on_trigger_load)
            .add_observer(on_trigger_load_autosave)
            .add_observer(on_trigger_new_game)
            .add_systems(
                Update,
                apply_pending_load.run_if(|p: Res<PendingLoad>| p.0.is_some()),
            )
            .add_systems(FixedUpdate, check_autosave.run_if(in_state(AppState::Playing)));
    }
}

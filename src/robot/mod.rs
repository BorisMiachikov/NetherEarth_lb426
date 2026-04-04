pub mod builder;
pub mod bundle;
pub mod components;
pub mod registry;
pub mod systems;

use bevy::prelude::*;

use registry::{load_module_registry, ModuleRegistry};
use systems::recalc_stats;

pub use builder::RobotBlueprint;
pub use components::ChassisType;
pub use registry::ModuleRegistry as Registry;

pub struct RobotPlugin;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        let registry = load_module_registry().unwrap_or_else(|e| {
            warn!("Не удалось загрузить ModuleRegistry: {e}. Использую заглушку.");
            ModuleRegistry {
                chassis: vec![],
                weapons: vec![],
                electronics: registry::ElectronicsDef {
                    accuracy_bonus: 0.3,
                    fire_rate_bonus: 0.2,
                    radar_range: 20.0,
                    capture_time_reduction: 0.3,
                    cost_electronics: 20,
                    cost_general: 10,
                },
                nuclear: registry::NuclearDef {
                    blast_radius: 8.0,
                    detonation_delay: 2.0,
                    cost_nuclear: 50,
                    cost_general: 30,
                },
            }
        });

        app.insert_resource(registry)
            .add_systems(FixedUpdate, recalc_stats);
    }
}

use bevy::prelude::*;

use crate::core::{Health, Team};

use super::{
    builder::RobotBlueprint,
    components::{
        Chassis, ChassisType, Electronics, Nuclear, RobotMarker, RobotStats, WeaponSlots,
    },
    registry::ModuleRegistry,
};

/// Меш по типу шасси.
pub fn chassis_mesh(ct: ChassisType, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    match ct {
        ChassisType::Wheels => meshes.add(Cuboid::new(0.9, 0.5, 1.2)),
        ChassisType::Bipod => meshes.add(Cuboid::new(0.7, 0.8, 0.7)),
        ChassisType::Tracks => meshes.add(Cuboid::new(1.1, 0.5, 1.4)),
        ChassisType::AntiGrav => meshes.add(Cuboid::new(0.8, 0.3, 0.8)),
    }
}

/// Цвет команды.
fn team_material(
    team: Team,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let color = match team {
        Team::Player => Color::srgb(0.1, 0.6, 1.0),
        Team::Enemy => Color::srgb(1.0, 0.2, 0.2),
        Team::Neutral => Color::srgb(0.7, 0.7, 0.2),
    };
    materials.add(StandardMaterial {
        base_color: color,
        ..default()
    })
}

/// Спавн робота из blueprint. Возвращает Entity.
pub fn spawn_robot(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    blueprint: &RobotBlueprint,
    registry: &ModuleRegistry,
    team: Team,
    position: Vec3,
) -> Option<Entity> {
    blueprint.validate().ok()?;

    let chassis_def = registry.chassis(blueprint.chassis)?;

    // RobotStats
    let weapon_data = blueprint.weapon_data(registry);
    let slots = WeaponSlots { slots: weapon_data };
    let max_hp = chassis_def.base_hp + slots.total_weight() * 2.0;
    let speed = chassis_def.speed;
    let mut capture_time = chassis_def.capture_time;

    let chassis = Chassis {
        chassis_type: blueprint.chassis,
        base_hp: chassis_def.base_hp,
        speed: chassis_def.speed,
        mobility: chassis_def.mobility,
        capture_time: chassis_def.capture_time,
    };

    let stats = RobotStats {
        max_hp,
        speed,
        capture_time,
    };

    let mesh = chassis_mesh(blueprint.chassis, meshes);
    let mat = team_material(team, materials);

    let altitude = if chassis_def.can_fly { 2.0 } else { 0.3 };
    let pos = position.with_y(altitude);

    let mut entity = commands.spawn((
        Name::new(format!("Robot {:?} {:?}", team, blueprint.chassis)),
        RobotMarker,
        chassis,
        slots,
        stats,
        Health::new(max_hp),
        team,
        crate::movement::velocity::Velocity::default(),
        crate::command::command::RobotCommand::Idle,
        crate::command::queue::CommandQueue::default(),
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::from_translation(pos),
    ));

    // Опциональные модули
    if blueprint.has_electronics {
        let elec = registry.electronics.clone();
        capture_time *= 1.0 - elec.capture_time_reduction;
        entity.insert(Electronics {
            radar_range: elec.radar_range,
            accuracy_bonus: elec.accuracy_bonus,
            fire_rate_bonus: elec.fire_rate_bonus,
            capture_time_reduction: elec.capture_time_reduction,
        });
    }

    if blueprint.has_nuclear {
        let nuc = &registry.nuclear;
        entity.insert(Nuclear {
            blast_radius: nuc.blast_radius,
            detonation_delay: nuc.detonation_delay,
            armed: false,
        });
    }

    Some(entity.id())
}

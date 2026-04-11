use bevy::prelude::*;
use crate::combat::WeaponCooldowns;

use crate::core::{Health, Team};

use super::{
    builder::RobotBlueprint,
    components::{
        Chassis, ChassisType, Electronics, Nuclear, RobotMarker, RobotStats, VisionRange,
        WeaponSlots, BASE_VISION_RANGE,
    },
    registry::ModuleRegistry,
};

/// Основной меш корпуса робота по типу шасси.
pub fn chassis_mesh(ct: ChassisType, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    match ct {
        ChassisType::Wheels => meshes.add(Cuboid::new(0.9, 0.4, 1.2)),
        ChassisType::Bipod => meshes.add(Cuboid::new(0.6, 0.6, 0.5)),
        ChassisType::Tracks => meshes.add(Cuboid::new(0.9, 0.45, 1.3)),
        ChassisType::AntiGrav => meshes.add(Cuboid::new(0.8, 0.3, 0.8)),
    }
}

/// Материал для декоративных деталей (колёса, гусеницы, ноги).
fn dark_material(materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.12, 0.14),
        perceptual_roughness: 0.8,
        ..default()
    })
}

/// Светящийся материал (для антиграв-диска, куполов).
fn glow_material(
    materials: &mut Assets<StandardMaterial>,
    color: Color,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        emissive: LinearRgba::from(color) * 2.0,
        ..default()
    })
}

/// Спавнит декоративные дочерние меши для шасси. Вызывается внутри with_children.
/// Дети НЕ имеют Pickable, чтобы клик попадал в основной корпус (раскаст сквозь них).
pub fn spawn_chassis_details(
    parent: &mut ChildSpawnerCommands,
    ct: ChassisType,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let dark = dark_material(materials);

    match ct {
        ChassisType::Wheels => {
            // 4 колеса по углам корпуса (0.9 × 0.4 × 1.2)
            let wheel_mesh = meshes.add(Cylinder::new(0.2, 0.18));
            let wheel_rot = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
            for (x, z) in [(-0.5, -0.45), (0.5, -0.45), (-0.5, 0.45), (0.5, 0.45)] {
                parent.spawn((
                    Mesh3d(wheel_mesh.clone()),
                    MeshMaterial3d(dark.clone()),
                    Transform::from_xyz(x, -0.2, z).with_rotation(wheel_rot),
                ));
            }
        }
        ChassisType::Bipod => {
            // 2 ноги вниз + "голова" сверху (корпус = торс 0.6 × 0.6 × 0.5)
            let leg_mesh = meshes.add(Cuboid::new(0.18, 0.5, 0.18));
            for x in [-0.18, 0.18] {
                parent.spawn((
                    Mesh3d(leg_mesh.clone()),
                    MeshMaterial3d(dark.clone()),
                    Transform::from_xyz(x, -0.55, 0.0),
                ));
            }
            let head_mesh = meshes.add(Sphere::new(0.18));
            parent.spawn((
                Mesh3d(head_mesh),
                MeshMaterial3d(dark.clone()),
                Transform::from_xyz(0.0, 0.45, 0.0),
            ));
        }
        ChassisType::Tracks => {
            // 2 длинных гусеницы по бокам корпуса (0.9 × 0.45 × 1.3)
            let track_mesh = meshes.add(Cuboid::new(0.25, 0.35, 1.5));
            for x in [-0.55, 0.55] {
                parent.spawn((
                    Mesh3d(track_mesh.clone()),
                    MeshMaterial3d(dark.clone()),
                    Transform::from_xyz(x, -0.1, 0.0),
                ));
            }
        }
        ChassisType::AntiGrav => {
            // Светящийся диск снизу
            let disc_mesh = meshes.add(Cylinder::new(0.5, 0.05));
            let glow = glow_material(materials, Color::srgb(0.3, 0.9, 1.0));
            parent.spawn((
                Mesh3d(disc_mesh),
                MeshMaterial3d(glow),
                Transform::from_xyz(0.0, -0.2, 0.0),
            ));
        }
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

    let weapon_data = blueprint.weapon_data(registry);
    let slots = WeaponSlots { slots: weapon_data };
    let max_hp = chassis_def.base_hp + slots.total_weight() * 2.0;
    let speed = chassis_def.speed;

    let electronics_opt = if blueprint.has_electronics {
        Some(registry.electronics.clone())
    } else {
        None
    };

    // Дальность обнаружения: с электроникой — radar_range, без — базовая
    let vision_range = VisionRange(
        electronics_opt
            .as_ref()
            .map_or(BASE_VISION_RANGE, |e| e.radar_range),
    );
    let capture_time = if let Some(ref elec) = electronics_opt {
        crate::structure::capture::BASE_CAPTURE_TIME * (1.0 - elec.capture_time_reduction)
    } else {
        crate::structure::capture::BASE_CAPTURE_TIME
    };

    let chassis = Chassis {
        chassis_type: blueprint.chassis,
        base_hp: chassis_def.base_hp,
        speed: chassis_def.speed,
        mobility: chassis_def.mobility,
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
        crate::movement::steering::CurrentPath::default(),
        Pickable::default(),
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::from_translation(pos),
    ));
    entity.insert((
        WeaponCooldowns::default(),
        vision_range,
        crate::movement::steering::StuckDetector::default(),
        crate::editor::GameWorldEntity,
    ));

    // Декоративные дочерние меши (колёса, ноги, гусеницы, антиграв-диск)
    entity.with_children(|parent| {
        spawn_chassis_details(parent, blueprint.chassis, meshes, materials);
    });

    if let Some(elec) = electronics_opt {
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

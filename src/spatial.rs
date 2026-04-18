use std::collections::HashMap;

use bevy::prelude::*;

use crate::{app::state::AppState, core::Team, robot::components::RobotMarker};

/// Размер ячейки пространственного хэша (в мировых единицах).
const BUCKET_SIZE: f32 = 4.0;

/// Метка системного набора — все системы, читающие индекс, должны идти после него.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpatialSet;

/// Равномерный пространственный хэш роботов.
/// Обновляется раз в FixedUpdate перед combat и steering.
#[derive(Resource, Default)]
pub struct SpatialIndex {
    buckets: HashMap<(i32, i32), Vec<(Entity, Vec3, Team)>>,
}

impl SpatialIndex {
    fn key(pos: Vec3) -> (i32, i32) {
        ((pos.x / BUCKET_SIZE) as i32, (pos.z / BUCKET_SIZE) as i32)
    }

    pub fn clear(&mut self) {
        for v in self.buckets.values_mut() {
            v.clear();
        }
    }

    pub fn insert(&mut self, entity: Entity, pos: Vec3, team: Team) {
        self.buckets
            .entry(Self::key(pos))
            .or_default()
            .push((entity, pos, team));
    }

    /// Вызывает `f` для каждой сущности в радиусе `radius` от `center`.
    /// Без аллокаций на горячем пути.
    pub fn query_radius(&self, center: Vec3, radius: f32, mut f: impl FnMut(Entity, Vec3, Team)) {
        let r = (radius / BUCKET_SIZE).ceil() as i32 + 1;
        let (cx, cz) = Self::key(center);
        let r2 = radius * radius;
        for bx in (cx - r)..=(cx + r) {
            for bz in (cz - r)..=(cz + r) {
                if let Some(bucket) = self.buckets.get(&(bx, bz)) {
                    for &(entity, pos, team) in bucket {
                        if pos.distance_squared(center) <= r2 {
                            f(entity, pos, team);
                        }
                    }
                }
            }
        }
    }
}

/// Перестраивает SpatialIndex по актуальным позициям роботов.
pub fn update_spatial_index(
    mut index: ResMut<SpatialIndex>,
    robots: Query<(Entity, &Transform, &Team), With<RobotMarker>>,
) {
    index.clear();
    for (entity, tf, team) in &robots {
        index.insert(entity, tf.translation, *team);
    }
}

pub struct SpatialPlugin;

impl Plugin for SpatialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpatialIndex>().add_systems(
            FixedUpdate,
            update_spatial_index
                .in_set(SpatialSet)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

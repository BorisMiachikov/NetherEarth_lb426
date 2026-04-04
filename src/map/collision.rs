use bevy::prelude::*;

use crate::player::components::PlayerScout;

use super::grid::{MapGrid, CELL_SIZE};

/// Axis-separated коллизия скаута с непроходимыми ячейками карты.
/// Запускается в FixedUpdate после move_scout.
pub fn scout_collision(map: Res<MapGrid>, mut query: Query<&mut Transform, With<PlayerScout>>) {
    let Ok(mut tf) = query.single_mut() else {
        return;
    };

    let max_x = map.width as f32 * CELL_SIZE;
    let max_z = map.height as f32 * CELL_SIZE;

    // Зажимаем в границы карты
    let cx = tf.translation.x.clamp(0.0, max_x);
    let cz = tf.translation.z.clamp(0.0, max_z);

    // Проверяем проходимость текущей ячейки
    if let Some((gx, gy)) = map.world_to_grid(Vec3::new(cx, 0.0, cz)) {
        if !map.is_passable(gx, gy) {
            // Откатываем X — пробуем только Z
            let only_z = Vec3::new(tf.translation.x, tf.translation.y, cz);
            if let Some((gx2, gy2)) = map.world_to_grid(Vec3::new(tf.translation.x, 0.0, cz)) {
                if map.is_passable(gx2, gy2) {
                    tf.translation = only_z;
                    return;
                }
            }
            // Откатываем Z — пробуем только X
            let only_x = Vec3::new(cx, tf.translation.y, tf.translation.z);
            if let Some((gx3, gy3)) =
                map.world_to_grid(Vec3::new(cx, 0.0, tf.translation.z))
            {
                if map.is_passable(gx3, gy3) {
                    tf.translation = only_x;
                    return;
                }
            }
            // Оба заблокированы — не двигаемся (позиция остаётся прежней)
            // (не применяем cx/cz)
        } else {
            tf.translation.x = cx;
            tf.translation.z = cz;
        }
    }
}

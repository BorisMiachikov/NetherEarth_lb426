use bevy::prelude::*;

use crate::robot::components::ChassisType;

pub const CELL_SIZE: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Open,
    /// Устаревший тип — использовался до добавления рельефа. Аналог Rock.
    Blocked,
    /// Скала: непроходима для ВСЕХ шасси, блокирует LOS.
    Rock,
    /// Яма / расщелина: непроходима для Wheels/Bipod/Tracks, проходима для AntiGrav.
    Pit,
    /// Песок: проходим для всех, замедляет Wheels и Bipod (cost = 2).
    Sand,
    /// Занята структурой (entity-владелец). Непроходима, блокирует LOS.
    Structure(Entity),
}

impl CellType {
    /// Может ли шасси пройти через эту ячейку?
    pub fn is_passable_for(self, chassis: ChassisType) -> bool {
        match self {
            CellType::Open | CellType::Sand => true,
            CellType::Blocked | CellType::Rock => false,
            CellType::Pit => chassis == ChassisType::AntiGrav,
            CellType::Structure(_) => false,
        }
    }

    /// Стоимость прохода ячейки для A* (None = непроходимо).
    pub fn movement_cost(self, chassis: ChassisType) -> Option<u32> {
        match self {
            CellType::Open => Some(1),
            CellType::Sand => match chassis {
                ChassisType::Wheels | ChassisType::Bipod => Some(2),
                _ => Some(1),
            },
            CellType::Pit => {
                if chassis == ChassisType::AntiGrav {
                    Some(1)
                } else {
                    None
                }
            }
            CellType::Blocked | CellType::Rock => None,
            CellType::Structure(_) => None,
        }
    }

    /// Блокирует ли ячейка линию обзора?
    pub fn blocks_los(self) -> bool {
        matches!(self, CellType::Rock | CellType::Blocked | CellType::Structure(_))
    }

    /// Обратная совместимость: любая проходимость (для скаута/коллизий).
    pub fn is_passable(self) -> bool {
        matches!(self, CellType::Open | CellType::Sand | CellType::Pit)
    }
}

/// Игровая сетка. Ячейки лежат в плоскости XZ.
/// Координата ячейки (gx, gy) → центр мира: (gx + 0.5, 0, gy + 0.5).
#[derive(Resource)]
pub struct MapGrid {
    pub width: u32,
    pub height: u32,
    cells: Vec<CellType>,
}

impl MapGrid {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            cells: vec![CellType::Open; (width * height) as usize],
        }
    }

    pub fn get(&self, x: u32, y: u32) -> Option<CellType> {
        if x < self.width && y < self.height {
            Some(self.cells[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    pub fn set(&mut self, x: u32, y: u32, cell: CellType) {
        if x < self.width && y < self.height {
            self.cells[(y * self.width + x) as usize] = cell;
        }
    }

    /// Мировые координаты → ячейка сетки. Возвращает None за пределами карты.
    pub fn world_to_grid(&self, pos: Vec3) -> Option<(u32, u32)> {
        let gx = (pos.x / CELL_SIZE).floor();
        let gy = (pos.z / CELL_SIZE).floor();
        if gx >= 0.0 && gy >= 0.0 {
            let gx = gx as u32;
            let gy = gy as u32;
            if gx < self.width && gy < self.height {
                return Some((gx, gy));
            }
        }
        None
    }

    /// Ячейка сетки → центр ячейки в мировых координатах (Y = 0).
    pub fn grid_to_world(&self, x: u32, y: u32) -> Vec3 {
        Vec3::new(
            x as f32 * CELL_SIZE + CELL_SIZE * 0.5,
            0.0,
            y as f32 * CELL_SIZE + CELL_SIZE * 0.5,
        )
    }

    pub fn is_passable(&self, x: u32, y: u32) -> bool {
        self.get(x, y).map(|c| c.is_passable()).unwrap_or(false)
    }

    pub fn world_bounds(&self) -> (Vec3, Vec3) {
        let min = Vec3::ZERO;
        let max = Vec3::new(
            self.width as f32 * CELL_SIZE,
            0.0,
            self.height as f32 * CELL_SIZE,
        );
        (min, max)
    }

    /// Итератор по всем ячейкам: ((x, y), CellType).
    pub fn iter_cells(&self) -> impl Iterator<Item = ((u32, u32), CellType)> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).map(move |x| ((x, y), self.cells[(y * self.width + x) as usize]))
        })
    }

    /// Проверка линии обзора между двумя ячейками сетки (алгоритм Брезенхэма).
    /// Возвращает false если на пути есть ячейка blocks_los().
    pub fn has_line_of_sight(&self, from: (u32, u32), to: (u32, u32)) -> bool {
        let (mut x0, mut y0) = (from.0 as i32, from.1 as i32);
        let (x1, y1) = (to.0 as i32, to.1 as i32);

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            // Не проверяем начальную и конечную ячейки (там стоят сами юниты/структуры)
            if (x0, y0) != (from.0 as i32, from.1 as i32)
                && (x0, y0) != (x1, y1)
            {
                if x0 < 0 || y0 < 0 {
                    return false;
                }
                match self.get(x0 as u32, y0 as u32) {
                    Some(cell) if cell.blocks_los() => return false,
                    None => return false,
                    _ => {}
                }
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::robot::components::ChassisType;

    fn dummy_entity() -> Entity {
        Entity::from_raw_u32(42).unwrap()
    }

    #[test]
    fn grid_world_roundtrip() {
        let grid = MapGrid::new(64, 64);
        let world = grid.grid_to_world(10, 20);
        let (gx, gy) = grid.world_to_grid(world).unwrap();
        assert_eq!((gx, gy), (10, 20));
    }

    #[test]
    fn out_of_bounds_returns_none() {
        let grid = MapGrid::new(64, 64);
        assert!(grid.world_to_grid(Vec3::new(-1.0, 0.0, 0.0)).is_none());
        assert!(grid.world_to_grid(Vec3::new(64.0, 0.0, 0.0)).is_none());
        assert!(grid.get(64, 0).is_none());
    }

    #[test]
    fn set_and_get_cell() {
        let mut grid = MapGrid::new(64, 64);
        grid.set(5, 7, CellType::Blocked);
        assert_eq!(grid.get(5, 7), Some(CellType::Blocked));
        assert_eq!(grid.get(5, 8), Some(CellType::Open));
    }

    #[test]
    fn structure_cell_not_passable() {
        let mut grid = MapGrid::new(64, 64);
        grid.set(3, 3, CellType::Structure(dummy_entity()));
        assert!(!grid.is_passable(3, 3));
        assert!(grid.is_passable(3, 4));
    }

    #[test]
    fn blocked_not_passable() {
        let mut grid = MapGrid::new(64, 64);
        grid.set(1, 1, CellType::Blocked);
        assert!(!grid.is_passable(1, 1));
    }

    #[test]
    fn pit_passable_only_for_antigrav() {
        let mut grid = MapGrid::new(10, 10);
        grid.set(3, 3, CellType::Pit);
        assert!(!CellType::Pit.is_passable_for(ChassisType::Wheels));
        assert!(!CellType::Pit.is_passable_for(ChassisType::Bipod));
        assert!(!CellType::Pit.is_passable_for(ChassisType::Tracks));
        assert!(CellType::Pit.is_passable_for(ChassisType::AntiGrav));
    }

    #[test]
    fn sand_costs_2_for_wheels_bipod() {
        assert_eq!(CellType::Sand.movement_cost(ChassisType::Wheels), Some(2));
        assert_eq!(CellType::Sand.movement_cost(ChassisType::Bipod), Some(2));
        assert_eq!(CellType::Sand.movement_cost(ChassisType::Tracks), Some(1));
        assert_eq!(CellType::Sand.movement_cost(ChassisType::AntiGrav), Some(1));
    }

    #[test]
    fn rock_blocks_los() {
        let mut grid = MapGrid::new(10, 10);
        grid.set(5, 0, CellType::Rock);
        // Прямая линия через скалу
        assert!(!grid.has_line_of_sight((0, 0), (9, 0)));
    }

    #[test]
    fn open_path_has_los() {
        let grid = MapGrid::new(10, 10);
        assert!(grid.has_line_of_sight((0, 0), (9, 9)));
    }
}

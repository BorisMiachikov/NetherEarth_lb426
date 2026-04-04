use bevy::prelude::*;

pub const CELL_SIZE: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Open,
    Blocked,
    Structure,
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
        matches!(self.get(x, y), Some(CellType::Open | CellType::Structure))
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

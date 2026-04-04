use std::collections::{BinaryHeap, HashMap};

use crate::map::grid::MapGrid;

/// Ячейка пути.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: u32,
    pub y: u32,
}

impl GridCell {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[derive(Eq, PartialEq)]
struct Node {
    cost: u32,
    cell: GridCell,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost) // min-heap
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn heuristic(a: GridCell, b: GridCell) -> u32 {
    a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
}

/// A* на MapGrid. `can_fly` — AntiGrav игнорирует заблокированные ячейки.
/// Возвращает путь от start (не включительно) до goal (включительно),
/// или None если цель недостижима.
pub fn find_path(
    map: &MapGrid,
    start: GridCell,
    goal: GridCell,
    can_fly: bool,
) -> Option<Vec<GridCell>> {
    if start == goal {
        return Some(vec![]);
    }

    let is_walkable = |c: GridCell| -> bool {
        if can_fly {
            // AntiGrav летит над Blocked, но не через Structure (здания)
            matches!(
                map.get(c.x, c.y),
                Some(crate::map::grid::CellType::Open)
                    | Some(crate::map::grid::CellType::Blocked)
            )
        } else {
            map.is_passable(c.x, c.y)
        }
    };

    if !is_walkable(goal) {
        return None;
    }

    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<GridCell, GridCell> = HashMap::new();
    let mut g_score: HashMap<GridCell, u32> = HashMap::new();

    g_score.insert(start, 0);
    open.push(Node {
        cost: heuristic(start, goal),
        cell: start,
    });

    while let Some(Node { cell: current, .. }) = open.pop() {
        if current == goal {
            // Восстанавливаем путь
            let mut path = vec![goal];
            let mut cur = goal;
            while let Some(&prev) = came_from.get(&cur) {
                path.push(prev);
                cur = prev;
            }
            path.reverse();
            path.remove(0); // убираем start
            return Some(path);
        }

        let cur_g = *g_score.get(&current).unwrap_or(&u32::MAX);

        // 4 соседа (без диагоналей)
        let neighbors = [
            (current.x.wrapping_sub(1), current.y),
            (current.x + 1, current.y),
            (current.x, current.y.wrapping_sub(1)),
            (current.x, current.y + 1),
        ];

        for (nx, ny) in neighbors {
            let neighbor = GridCell::new(nx, ny);
            if !is_walkable(neighbor) {
                continue;
            }
            let tentative_g = cur_g + 1;
            if tentative_g < *g_score.get(&neighbor).unwrap_or(&u32::MAX) {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g);
                open.push(Node {
                    cost: tentative_g + heuristic(neighbor, goal),
                    cell: neighbor,
                });
            }
        }
    }

    None // недостижимо
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::grid::{CellType, MapGrid};

    #[test]
    fn straight_path() {
        let map = MapGrid::new(10, 10);
        let path = find_path(&map, GridCell::new(0, 0), GridCell::new(3, 0), false).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path.last().unwrap(), &GridCell::new(3, 0));
    }

    #[test]
    fn path_around_wall() {
        let mut map = MapGrid::new(10, 10);
        // Стена по x=2, y=0..4
        for y in 0..4 {
            map.set(2, y, CellType::Blocked);
        }
        let path = find_path(&map, GridCell::new(0, 0), GridCell::new(4, 0), false).unwrap();
        assert!(!path.is_empty());
        // Путь не проходит через заблокированные ячейки
        for cell in &path {
            assert!(map.is_passable(cell.x, cell.y));
        }
    }

    #[test]
    fn unreachable_goal_returns_none() {
        let mut map = MapGrid::new(5, 5);
        // Окружаем цель со всех сторон
        map.set(3, 2, CellType::Blocked);
        map.set(2, 3, CellType::Blocked);
        map.set(4, 3, CellType::Blocked);
        map.set(3, 4, CellType::Blocked);
        // Сама цель открыта, но окружена
        let result = find_path(&map, GridCell::new(0, 0), GridCell::new(3, 3), false);
        assert!(result.is_none());
    }

    #[test]
    fn same_start_and_goal() {
        let map = MapGrid::new(10, 10);
        let path = find_path(&map, GridCell::new(2, 2), GridCell::new(2, 2), false).unwrap();
        assert!(path.is_empty());
    }
}

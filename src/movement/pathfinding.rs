use std::collections::{BinaryHeap, HashMap};

use crate::{map::grid::MapGrid, robot::components::ChassisType};

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

/// A* на MapGrid с учётом типа шасси.
///
/// - Rock/Blocked: непроходимо для всех.
/// - Pit: непроходимо для Wheels/Bipod/Tracks, проходимо для AntiGrav.
/// - Sand: стоимость 2 для Wheels/Bipod, 1 для остальных.
/// - Structure: непроходима, но смежные ячейки используются как fallback-цель.
///
/// Возвращает путь от start (не включительно) до goal (включительно),
/// или None если цель недостижима.
pub fn find_path(
    map: &MapGrid,
    start: GridCell,
    goal: GridCell,
    chassis: ChassisType,
) -> Option<Vec<GridCell>> {
    if start == goal {
        return Some(vec![]);
    }

    let cell_cost = |c: GridCell| -> Option<u32> {
        map.get(c.x, c.y)?.movement_cost(chassis)
    };

    let is_walkable = |c: GridCell| cell_cost(c).is_some();

    // Если цель непроходима (стоит структура или скала), берём ближайшую соседнюю клетку.
    let goal = if !is_walkable(goal) {
        let adj = [
            GridCell::new(goal.x.wrapping_sub(1), goal.y),
            GridCell::new(goal.x + 1, goal.y),
            GridCell::new(goal.x, goal.y.wrapping_sub(1)),
            GridCell::new(goal.x, goal.y + 1),
        ];
        match adj.into_iter().find(|&c| is_walkable(c)) {
            Some(c) => c,
            None => return None,
        }
    } else {
        goal
    };

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
            let Some(cost) = cell_cost(neighbor) else {
                continue;
            };
            let tentative_g = cur_g.saturating_add(cost);
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
        let path = find_path(&map, GridCell::new(0, 0), GridCell::new(3, 0), ChassisType::Wheels).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path.last().unwrap(), &GridCell::new(3, 0));
    }

    #[test]
    fn path_around_wall() {
        let mut map = MapGrid::new(10, 10);
        for y in 0..4 {
            map.set(2, y, CellType::Rock);
        }
        let path = find_path(&map, GridCell::new(0, 0), GridCell::new(4, 0), ChassisType::Wheels).unwrap();
        assert!(!path.is_empty());
        for cell in &path {
            assert!(CellType::Rock != map.get(cell.x, cell.y).unwrap());
        }
    }

    #[test]
    fn pit_blocks_wheels_not_antigrav() {
        let mut map = MapGrid::new(10, 10);
        // Ряд ям поперёк пути
        for x in 0..10 {
            map.set(x, 3, CellType::Pit);
        }
        // Wheels не может пересечь
        assert!(find_path(&map, GridCell::new(5, 0), GridCell::new(5, 6), ChassisType::Wheels).is_none());
        // AntiGrav может
        assert!(find_path(&map, GridCell::new(5, 0), GridCell::new(5, 6), ChassisType::AntiGrav).is_some());
    }

    #[test]
    fn sand_prefers_open_for_wheels() {
        let mut map = MapGrid::new(10, 10);
        // Полоса песка по x=1..8, y=5 — прямой путь дороже
        for x in 0..10 {
            map.set(x, 5, CellType::Sand);
        }
        // Путь должен найтись (Sand проходим)
        let path = find_path(&map, GridCell::new(5, 0), GridCell::new(5, 9), ChassisType::Wheels);
        assert!(path.is_some());
    }

    #[test]
    fn unreachable_goal_returns_none() {
        let mut map = MapGrid::new(5, 5);
        map.set(3, 2, CellType::Rock);
        map.set(2, 3, CellType::Rock);
        map.set(4, 3, CellType::Rock);
        map.set(3, 4, CellType::Rock);
        let result = find_path(&map, GridCell::new(0, 0), GridCell::new(3, 3), ChassisType::Wheels);
        assert!(result.is_none());
    }

    #[test]
    fn same_start_and_goal() {
        let map = MapGrid::new(10, 10);
        let path = find_path(&map, GridCell::new(2, 2), GridCell::new(2, 2), ChassisType::Tracks).unwrap();
        assert!(path.is_empty());
    }
}

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::cmp::Ordering;

use crate::tilemap::{TileKind, Tilemap};

/// A position on the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

/// A node in the A* open set, ordered by cost (lowest f first).
#[derive(Debug, Clone, Eq, PartialEq)]
struct Node {
    pos: Pos,
    f: i32, // g + h (total estimated cost)
    g: i32, // cost from start to this node
}

// BinaryHeap is a max-heap, but we want the lowest f first.
// So we reverse the ordering.
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f) // Reversed: lower f = higher priority
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Manhattan distance heuristic (good for 4-directional grid movement).
fn heuristic(a: Pos, b: Pos) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

/// The 4 cardinal directions: up, down, left, right.
const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

/// Find the shortest path from `start` to `goal` using A*.
/// Returns the path as a Vec of positions (excluding `start`, including `goal`),
/// or None if no path exists.
pub fn find_path(start: Pos, goal: Pos, tilemap: &Tilemap) -> Option<Vec<Pos>> {
    // If start == goal, no path needed
    if start == goal {
        return Some(vec![]);
    }

    // If goal is not walkable, no path exists
    let goal_tile = tilemap.get(goal.x, goal.y);
    match goal_tile {
        TileKind::Wall | TileKind::Water => return None,
        _ => {}
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<Pos, Pos> = HashMap::new();
    let mut g_scores: HashMap<Pos, i32> = HashMap::new();

    g_scores.insert(start, 0);
    open_set.push(Node {
        pos: start,
        f: heuristic(start, goal),
        g: 0,
    });

    while let Some(current) = open_set.pop() {
        // Reached the goal — reconstruct the path
        if current.pos == goal {
            let mut path = vec![];
            let mut pos = goal;
            while pos != start {
                path.push(pos);
                pos = came_from[&pos];
            }
            path.reverse();
            return Some(path);
        }

        // Skip if we already found a better path to this node
        let current_g = g_scores.get(&current.pos).copied().unwrap_or(i32::MAX);
        if current.g > current_g {
            continue;
        }

        // Explore neighbors
        for (dx, dy) in &DIRECTIONS {
            let neighbor = Pos {
                x: current.pos.x + dx,
                y: current.pos.y + dy,
            };

            // Check bounds
            if neighbor.x < 0 || neighbor.x >= tilemap.cols
                || neighbor.y < 0 || neighbor.y >= tilemap.rows
            {
                continue;
            }

            // Check if walkable
            let tile = tilemap.get(neighbor.x, neighbor.y);
            match tile {
                TileKind::Wall | TileKind::Water => continue,
                _ => {}
            }

            let new_g = current_g + 1; // Each step costs 1
            let prev_g = g_scores.get(&neighbor).copied().unwrap_or(i32::MAX);

            if new_g < prev_g {
                g_scores.insert(neighbor, new_g);
                came_from.insert(neighbor, current.pos);
                open_set.push(Node {
                    pos: neighbor,
                    f: new_g + heuristic(neighbor, goal),
                    g: new_g,
                });
            }
        }
    }

    // No path found
    None
}

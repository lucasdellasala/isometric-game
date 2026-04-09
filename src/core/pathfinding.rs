use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::cmp::Ordering;

use crate::core::tilemap::Tilemap;

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

/// Octile distance heuristic (8-directional with diagonal cost ~1.4).
/// Uses integer approximation: cardinal=10, diagonal=14.
fn heuristic(a: Pos, b: Pos) -> i32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    let diag = dx.min(dy);
    let straight = dx.max(dy) - diag;
    diag * 14 + straight * 10
}

/// 8 directions: 4 cardinal + 4 diagonal.
const DIRECTIONS: [(i32, i32); 8] = [
    (0, -1), (0, 1), (-1, 0), (1, 0),   // cardinal
    (-1, -1), (1, -1), (-1, 1), (1, 1),  // diagonal
];

/// Debug info captured during A* execution. Render-side only — never stored in GameState.
pub struct PathDebugInfo {
    /// Tiles that were fully explored (popped from open set).
    pub closed_set: Vec<Pos>,
    /// Final path from start to goal (same as find_path return value).
    pub path: Vec<Pos>,
    /// Start and goal positions.
    pub start: Pos,
    pub goal: Pos,
    /// Whether a path was found.
    pub found: bool,
}

/// Find the shortest path from `start` to `goal` using A*.
/// Returns the path as a Vec of positions (excluding `start`, including `goal`),
/// or None if no path exists.
pub fn find_path(start: Pos, goal: Pos, tilemap: &Tilemap, blocked: &std::collections::HashSet<(i32, i32)>) -> Option<Vec<Pos>> {
    // If start == goal, no path needed
    if start == goal {
        return Some(vec![]);
    }

    // If goal is not walkable or blocked by an object, no path exists
    if !tilemap.get(goal.x, goal.y).is_walkable() || blocked.contains(&(goal.x, goal.y)) {
        return None;
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

            // Check if walkable and not blocked by objects
            if !tilemap.get(neighbor.x, neighbor.y).is_walkable() || blocked.contains(&(neighbor.x, neighbor.y)) {
                continue;
            }

            // Cardinal moves cost 10, diagonal moves cost 14 (~√2 × 10)
            let is_diagonal = dx.abs() + dy.abs() == 2;
            let step_cost = if is_diagonal { 14 } else { 10 };
            let new_g = current_g + step_cost;
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

/// Same as find_path but captures debug info (closed set, path).
/// Used by the renderer for visualization — not by gameplay code.
pub fn find_path_with_debug(
    start: Pos,
    goal: Pos,
    tilemap: &Tilemap,
    blocked: &std::collections::HashSet<(i32, i32)>,
) -> PathDebugInfo {
    let mut debug = PathDebugInfo {
        closed_set: Vec::new(),
        path: Vec::new(),
        start,
        goal,
        found: false,
    };

    if start == goal {
        debug.found = true;
        return debug;
    }

    if !tilemap.get(goal.x, goal.y).is_walkable() || blocked.contains(&(goal.x, goal.y)) {
        return debug;
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
        let current_g = g_scores.get(&current.pos).copied().unwrap_or(i32::MAX);
        if current.g > current_g {
            continue;
        }

        debug.closed_set.push(current.pos);

        if current.pos == goal {
            let mut path = vec![];
            let mut pos = goal;
            while pos != start {
                path.push(pos);
                pos = came_from[&pos];
            }
            path.reverse();
            debug.path = path;
            debug.found = true;
            return debug;
        }

        for (dx, dy) in &DIRECTIONS {
            let neighbor = Pos {
                x: current.pos.x + dx,
                y: current.pos.y + dy,
            };

            if neighbor.x < 0 || neighbor.x >= tilemap.cols
                || neighbor.y < 0 || neighbor.y >= tilemap.rows
            {
                continue;
            }

            if !tilemap.get(neighbor.x, neighbor.y).is_walkable() || blocked.contains(&(neighbor.x, neighbor.y)) {
                continue;
            }

            let is_diagonal = dx.abs() + dy.abs() == 2;
            let step_cost = if is_diagonal { 14 } else { 10 };
            let new_g = current_g + step_cost;
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

    debug
}

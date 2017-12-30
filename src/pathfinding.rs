use std::cmp::Ordering;
use std::u32;
use std::collections::{BinaryHeap, HashMap, HashSet};
use terrain::Terrain;
use num::Rational;
use num::rational::Ratio;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Direction {
    N,
    S,
    E,
    W,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct DaState {
    dist: Rational,
    pos: (u32, u32),
}

impl DaState {
    fn new(dist: Rational, pos: (u32, u32)) -> Self {
        DaState { dist, pos }
    }
}

impl PartialOrd for DaState {
    fn partial_cmp(&self, other: &DaState) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DaState {
    fn cmp(&self, other: &DaState) -> Ordering {
        other
            .dist
            .cmp(&self.dist)
            .then_with(|| self.pos.cmp(&other.pos))
    }
}

fn is_point_valid((point_x, point_y): (u32, u32), max_w: u32, max_h: u32) -> bool {
    point_x < max_w && point_y < max_h
}

// Returns vec of ((coord_x, coord_y), terrain movement cost).
fn neighbor_costs(
    (point_x, point_y): (u32, u32),
    terrain: &Vec<Vec<Terrain>>,
    max_w: u32,
    max_h: u32,
) -> Vec<((u32, u32), Rational)> {
    let mut neighbors = vec![];
    if point_x >= 1 && is_point_valid((point_x - 1, point_y), max_w, max_h)
        && terrain[(point_x - 1) as usize][point_y as usize] != Terrain::Wall
    {
        let terr = &terrain[(point_x - 1) as usize][point_y as usize];
        neighbors.push(((point_x - 1, point_y), terr.cost()));
    }
    if is_point_valid((point_x + 1, point_y), max_w, max_h)
        && terrain[(point_x + 1) as usize][point_y as usize] != Terrain::Wall
    {
        let terr = &terrain[(point_x + 1) as usize][point_y as usize];
        neighbors.push(((point_x + 1, point_y), terr.cost()));
    }
    if point_y >= 1 && is_point_valid((point_x, point_y - 1), max_w, max_h)
        && terrain[point_x as usize][(point_y - 1) as usize] != Terrain::Wall
    {
        let terr = &terrain[point_x as usize][(point_y - 1) as usize];
        neighbors.push(((point_x, point_y - 1), terr.cost()));
    }
    if is_point_valid((point_x, point_y + 1), max_w, max_h)
        && terrain[point_x as usize][(point_y + 1) as usize] != Terrain::Wall
    {
        let terr = &terrain[point_x as usize][(point_y + 1) as usize];
        neighbors.push(((point_x, point_y + 1), terr.cost()));
    }
    neighbors
}

// Gets all valid neighbor coordinates (doesn't look at terrain)
fn valid_neighbor_coords(
    (point_x, point_y): (u32, u32),
    max_w: u32,
    max_h: u32,
) -> Vec<(u32, u32)> {
    let mut neighbors = vec![];
    if point_x >= 1 && is_point_valid((point_x - 1, point_y), max_w, max_h) {
        neighbors.push((point_x - 1, point_y));
    }
    if is_point_valid((point_x + 1, point_y), max_w, max_h) {
        neighbors.push((point_x + 1, point_y));
    }
    if point_y >= 1 && is_point_valid((point_x, point_y - 1), max_w, max_h) {
        neighbors.push((point_x, point_y - 1));
    }
    if is_point_valid((point_x, point_y + 1), max_w, max_h) {
        neighbors.push((point_x, point_y + 1));
    }
    neighbors
}

// Dijkstra's algorithm
// Params:
// src: (x, y) coords of source and destination
// terrain: 2d vec (x, y) of terrain features
// max_w, max_h: grid width, height
// max_dist: max movement of unit (algorithm stops considering neighbors when it
// encounters a total cost > max_dist)
//
// Returns:
// 0: map of backpointers indicating best paths to each coord
// 1: map of costs to each coord
// 2: set of boundary coords
// 3: set of all reachable coords
//
// At the src point, came_from points to src (i.e., to find the end of the path,
// the condition is that the backpointer for a point points to the point
// itself).
pub fn compute_path_costs(
    src: (u32, u32),
    terrain: &Vec<Vec<Terrain>>,
    max_w: u32,
    max_h: u32,
    max_dist: u32,
) -> (
    HashMap<(u32, u32), (u32, u32)>,
    HashMap<(u32, u32), Rational>,
    HashSet<(u32, u32)>,
    HashSet<(u32, u32)>,
) {
    let mut frontier = BinaryHeap::new();
    frontier.push(DaState::new(Ratio::from_integer(0), src));
    let mut came_from: HashMap<(u32, u32), (u32, u32)> = HashMap::new();
    came_from.insert(src, src);
    let mut cost_so_far = HashMap::new();
    let max_dist = Ratio::from_integer(max_dist as isize);
    cost_so_far.insert(src, Ratio::from_integer(0));

    let mut max_boundary = HashSet::new();
    let mut reachable_coords = HashSet::new();

    while !frontier.is_empty() {
        let current = frontier.pop().unwrap();
        max_boundary.remove(&came_from[&current.pos]);
        max_boundary.insert(current.pos);
        reachable_coords.insert(current.pos);
        for (neighbor_coord, cost) in neighbor_costs(current.pos, terrain, max_w, max_h) {
            let new_cost = cost_so_far[&current.pos] + cost;
            if new_cost <= max_dist
                && (!cost_so_far.contains_key(&neighbor_coord)
                    || new_cost < cost_so_far[&neighbor_coord])
            {
                cost_so_far.insert(neighbor_coord.clone(), new_cost);
                frontier.push(DaState::new(new_cost, neighbor_coord.clone()));
                came_from.insert(neighbor_coord.clone(), current.pos);
            }
        }
    }
    (came_from, cost_so_far, max_boundary, reachable_coords)
}

// Reads from the map of backpointers to get the best path to dest.
pub fn get_path(dest: (u32, u32), paths: &HashMap<(u32, u32), (u32, u32)>) -> Vec<(u32, u32)> {
    if paths.get(&dest) == Some(&dest) {
        // src == dest
        return vec![dest];
    }
    let mut path = vec![];
    let mut cur = dest;
    while paths.contains_key(&cur) && paths[&cur] != cur {
        path.push(cur);
        cur = paths[&cur];
    }
    if path.len() > 0 {
        path.push(cur);
    }
    path.reverse();
    path
}

// Combines path from get_path into endpoints for line segments that can be
// passed to the draw function
pub fn consolidate_path(path: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    if path.len() <= 1 {
        return path;
    }
    let mut consolidated_path = vec![path[0]];
    let mut prev_horizontal = path[0].0 == path[1].0;
    for window in path.windows(2).skip(1) {
        let (prev_x, prev_y) = (*window)[0];
        let (step_x, _) = (*window)[1];
        let cur_horizontal = step_x == prev_x;
        if cur_horizontal != prev_horizontal {
            consolidated_path.push((prev_x, prev_y));
        }
        prev_horizontal = cur_horizontal;
    }
    if consolidated_path[consolidated_path.len() - 1] != path[path.len() - 1] {
        consolidated_path.push(path[path.len() - 1]);
    }
    consolidated_path
}

// Gets the direction to `to`, starting from `from`. `from` and `to` must be
// adjacent (otherwise panics).
fn relative_adj_dir(from: (u32, u32), to: (u32, u32)) -> Direction {
    let from = tuple_as!(from, (x, i32), (y, i32));
    let to = tuple_as!(to, (x, i32), (y, i32));
    if from.0 == to.0 {
        if from.1 - to.1 == 1 {
            Direction::N
        } else if from.1 - to.1 == -1 {
            Direction::S
        } else {
            unreachable!();
        }
    } else if from.1 == to.1 {
        if from.0 - to.0 == 1 {
            Direction::W
        } else if from.0 - to.0 == -1 {
            Direction::E
        } else {
            unreachable!();
        }
    } else {
        unreachable!();
    }
}

// For each grid coord in the boundary, look at its neighbors and tag each coord
// with the direction of each neighbor. We need to do this because when we draw
// the boundary, we need to determine on which sides of the grid cell to draw
// the border.
pub fn find_boundary_neighbor_directions(
    boundary: &HashSet<(u32, u32)>,
    reachable_coords: &HashSet<(u32, u32)>,
    max_w: u32,
    max_h: u32,
) -> Vec<((u32, u32), Vec<Direction>)> {
    boundary
        .iter()
        .map(|&(x, y)| {
            let mut directions = vec![Direction::N, Direction::S, Direction::E, Direction::W];
            let neighbors = valid_neighbor_coords((x, y), max_w, max_h);
            for neighbor in neighbors.iter() {
                if reachable_coords.contains(&neighbor) {
                    directions.remove_item(&relative_adj_dir((x, y), *neighbor));
                }
            }
            ((x, y), directions)
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_path_src() {
        let paths = hashmap! {
            (2, 1) => (1, 1),
            (1, 1) => (0, 0),
            (0, 0) => (0, 0),
        };
        assert_eq!(get_path((0, 0), &paths), vec![(0, 0)]);
    }

    #[test]
    fn test_get_path_1() {
        let paths = hashmap! {
            (2, 1) => (1, 1),
            (1, 1) => (0, 0),
            (0, 0) => (0, 0),
        };
        assert_eq!(get_path((1, 1), &paths), vec![(0, 0), (1, 1)]);
    }

    #[test]
    fn test_get_path_2() {
        let paths = hashmap! {
            (2, 1) => (1, 1),
            (1, 1) => (0, 0),
            (0, 0) => (0, 0),
        };
        assert_eq!(get_path((2, 1), &paths), vec![(0, 0), (1, 1), (2, 1)]);
    }

    #[test]
    fn test_consolidate_path_simple_1() {
        let path = vec![];
        let cpath = consolidate_path(path);
        assert_eq!(cpath, vec![]);
    }

    #[test]
    fn test_consolidate_path_simple_2() {
        let path = vec![(0, 0)];
        let cpath = consolidate_path(path);
        assert_eq!(cpath, vec![(0, 0)]);
    }

    #[test]
    fn test_consolidate_path_simple_3() {
        let path = vec![(0, 0), (1, 0)];
        let cpath = consolidate_path(path);
        assert_eq!(cpath, vec![(0, 0), (1, 0)]);
    }

    #[test]
    fn test_consolidate_path_simple_4() {
        let path = vec![(0, 0), (1, 0), (2, 0)];
        let cpath = consolidate_path(path);
        assert_eq!(cpath, vec![(0, 0), (2, 0)]);
    }

    #[test]
    fn test_consolidate_path_with_turn_1() {
        let path = vec![(0, 0), (1, 0), (2, 0), (2, 1)];
        let cpath = consolidate_path(path);
        assert_eq!(cpath, vec![(0, 0), (2, 0), (2, 1)]);
    }

    #[test]
    fn test_consolidate_path_with_turn_2() {
        let path = vec![
            (1, 0),
            (2, 0),
            (3, 0),
            (4, 0),
            (5, 0),
            (6, 0),
            (6, 1),
            (6, 2),
            (5, 2),
            (4, 2),
        ];
        let cpath = consolidate_path(path);
        assert_eq!(cpath, vec![(1, 0), (6, 0), (6, 2), (4, 2)]);
    }
}

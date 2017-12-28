use std::cmp::Ordering;
use std::u32;
use std::collections::{HashMap, BinaryHeap};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct DaState {
    dist: u32,
    pos: (u32, u32)
}

impl DaState {
    fn new(dist: u32, pos: (u32, u32)) -> Self {
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
        other.dist.cmp(&self.dist).then_with(|| self.pos.cmp(&other.pos))
    }
}

fn is_point_valid((point_x, point_y): (u32, u32), max_w: u32, max_h: u32) -> bool {
    point_x < max_w && point_y < max_h
}

// Returns vec of ((coord_x, coord_y), cost)
// Right now, all tile movement costs are 1.
fn get_neighbors((point_x, point_y): (u32, u32), walls: &Vec<Vec<bool>>, max_w: u32, max_h: u32) -> Vec<((u32, u32), u32)> {
    let mut neighbors = vec![];
    if point_x >= 1 &&
       is_point_valid((point_x - 1, point_y), max_w, max_h) &&
       !walls[(point_x - 1) as usize][point_y as usize] {
        neighbors.push(((point_x - 1, point_y), 1));
    }
    if is_point_valid((point_x + 1, point_y), max_w, max_h) &&
       !walls[(point_x + 1) as usize][point_y as usize] {
       neighbors.push(((point_x + 1, point_y), 1));
    }
    if point_y >= 1 &&
       is_point_valid((point_x, point_y - 1), max_w, max_h) &&
       !walls[point_x as usize][(point_y - 1) as usize] {
        neighbors.push(((point_x, point_y - 1), 1));
    }
    if is_point_valid((point_x, point_y + 1), max_w, max_h) &&
       !walls[point_x as usize][(point_y + 1) as usize] {
       neighbors.push(((point_x, point_y + 1), 1));
    }
    neighbors
}

// Dijkstra's algorithm
// Params: (x, y) coords of source and destination
pub fn compute_path_costs(src: (u32, u32), walls: &Vec<Vec<bool>>, max_w: u32, max_h: u32) -> (HashMap<(u32, u32), (u32, u32)>, HashMap<(u32, u32), u32>) {
    let mut frontier = BinaryHeap::new();
    frontier.push(DaState::new(0, src));
    let mut came_from = HashMap::new();
    let mut cost_so_far = HashMap::new();
    cost_so_far.insert(src.clone(), 0);

    while !frontier.is_empty() {
        let current = frontier.pop().unwrap();
        for (neighbor_coord, cost) in get_neighbors(current.pos, walls, max_w, max_h) {
            let new_cost = cost_so_far[&current.pos] + cost;
            if !cost_so_far.contains_key(&neighbor_coord) || new_cost < cost_so_far[&neighbor_coord] {
                cost_so_far.insert(neighbor_coord.clone(), new_cost);
                frontier.push(DaState::new(new_cost, neighbor_coord.clone()));
                came_from.insert(neighbor_coord.clone(), current.pos);
            }
        }
    }
    (came_from, cost_so_far)
}

pub fn get_path(dest: (u32, u32), paths: HashMap<(u32, u32), (u32, u32)>) -> Vec<(u32, u32)> {
    let mut path = vec![];
    let mut cur = dest;
    while paths.contains_key(&cur) {
        path.push(cur);
        cur = paths[&cur];
    }
    path.reverse();
    path
}

pub fn consolidate_path(path: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    if path.len() <= 1 {
        return path.clone();
    }
    let mut consolidated_path = vec![path[0]];
    let mut prev_horizontal = path[0].0 == path[1].0;
    let (mut prev_x, mut prev_y) = path[0];
    for &(step_x, step_y) in path.iter().skip(1) {
        let cur_horizontal = step_x == prev_x;
        if cur_horizontal != prev_horizontal {
            consolidated_path.push((prev_x, prev_y));
        }
        prev_x = step_x;
        prev_y = step_y;
        prev_horizontal = cur_horizontal;
    }
    if consolidated_path[consolidated_path.len() - 1] != path[path.len() - 1] {
        consolidated_path.push(path[path.len() - 1]);
    }
    consolidated_path
}
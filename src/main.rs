#![feature(use_nested_groups)]
extern crate ggez;
#[macro_use] extern crate indoc;

mod dataparser;

use ggez::*;
use ggez::event::*;
use ggez::graphics::{Drawable, DrawMode, Point2, DrawParam, Rect, Image, spritebatch::*};
use ggez::conf::{WindowMode, WindowSetup};
use std::env;
use std::path;
use std::cmp::Ordering;
use std::u32;
use std::collections::{HashMap, BinaryHeap};

#[derive(Debug)]
struct MainState {
    mouse_coords: (u32, u32),
    font: graphics::Font,
    konrad_imgs: Vec<Image>,
    konrad_tick: f32,
    walls: Vec<Vec<bool>>,
    walls_sb: SpriteBatch,
}

const FPS: u32 = 60;
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 800;
const VERTICAL_PADDING: u32 = 30;
const HORIZONTAL_PADDING: u32 = 270;
const GRID_LINE_WIDTH: u32 = 2;  // should be even
const GRID_N_CELL_WIDTH: u32 = 10;  // number of horizontal grid cells
const GRID_N_CELL_HEIGHT: u32 = 10;  // number of verical grid cells
const GRID_CELL_DIM: u32 = (WINDOW_HEIGHT - 2 * VERTICAL_PADDING) / GRID_N_CELL_HEIGHT; // 74

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let konrad_imgs = vec![
            Image::new(ctx, "/konrad-commander.png")?,
            Image::new(ctx, "/konrad-commander-attack-1.png")?,
            Image::new(ctx, "/konrad-commander-attack-2.png")?,
            Image::new(ctx, "/konrad-commander-attack-3.png")?,
            Image::new(ctx, "/konrad-commander-attack-4.png")?,
        ];
        let walls = dataparser::parse_walls("walls.txt", 10, 10)?;
        let mut walls_sb = SpriteBatch::new(graphics::Image::new(ctx, "/wall.png")?);

        for (x, row) in walls.iter().enumerate() {
            for (y, is_wall) in row.iter().enumerate() {
                if *is_wall {
                    let (rect_x, rect_y) = grid_to_screen_coord((x as u32, y as u32));
                    walls_sb.add(DrawParam {
                        dest: Point2::new(rect_x as f32, rect_y as f32),
                        ..DrawParam::default()
                    });
                }
            }
        }

        Ok(MainState {
            mouse_coords: (0, 0),
            font: graphics::Font::new(ctx, "/DejaVuSerif.ttf", 10)?,
            konrad_imgs,
            konrad_tick: 0.0,
            walls,
            walls_sb
        })
    }
}

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
fn get_neighbors((point_x, point_y): (u32, u32), walls: &Vec<Vec<bool>>) -> Vec<((u32, u32), u32)> {
    let mut neighbors = vec![];
    if point_x >= 1 &&
       is_point_valid((point_x - 1, point_y), GRID_N_CELL_WIDTH, GRID_N_CELL_HEIGHT) &&
       !walls[(point_x - 1) as usize][point_y as usize] {
        neighbors.push(((point_x - 1, point_y), 1));
    }
    if is_point_valid((point_x + 1, point_y), GRID_N_CELL_WIDTH, GRID_N_CELL_HEIGHT) &&
       !walls[(point_x + 1) as usize][point_y as usize] {
       neighbors.push(((point_x + 1, point_y), 1));
    }
    if point_y >= 1 &&
       is_point_valid((point_x, point_y - 1), GRID_N_CELL_WIDTH, GRID_N_CELL_HEIGHT) &&
       !walls[point_x as usize][(point_y - 1) as usize] {
        neighbors.push(((point_x, point_y - 1), 1));
    }
    if is_point_valid((point_x, point_y + 1), GRID_N_CELL_WIDTH, GRID_N_CELL_HEIGHT) &&
       !walls[point_x as usize][(point_y + 1) as usize] {
       neighbors.push(((point_x, point_y + 1), 1));
    }
    neighbors
}

// Dijkstra's algorithm
// Params: (x, y) coords of source and destination
fn compute_path_costs(src: (u32, u32), walls: &Vec<Vec<bool>>) -> (HashMap<(u32, u32), (u32, u32)>, HashMap<(u32, u32), u32>) {
    let mut frontier = BinaryHeap::new();
    frontier.push(DaState::new(0, src));
    let mut came_from = HashMap::new();
    let mut cost_so_far = HashMap::new();
    cost_so_far.insert(src.clone(), 0);

    while !frontier.is_empty() {
        let current = frontier.pop().unwrap();
        for (neighbor_coord, cost) in get_neighbors(current.pos, walls) {
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

fn get_path(dest: (u32, u32), paths: HashMap<(u32, u32), (u32, u32)>) -> Vec<(u32, u32)> {
    let mut path = vec![];
    let mut cur = dest;
    while paths.contains_key(&cur) {
        path.push(cur);
        cur = paths[&cur];
    }
    path.reverse();
    path
}

fn consolidate_path(path: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
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

fn draw_grid(ctx: &mut Context) -> GameResult<()> {
    for i in 0..GRID_N_CELL_HEIGHT + 1 {
        graphics::line(ctx,
                       &[Point2::new((HORIZONTAL_PADDING + i * GRID_CELL_DIM) as f32, VERTICAL_PADDING as f32),
                         Point2::new((HORIZONTAL_PADDING + i * GRID_CELL_DIM) as f32, (WINDOW_HEIGHT - VERTICAL_PADDING) as f32)],
                       GRID_LINE_WIDTH as f32)?;
    }
    for i in 0..GRID_N_CELL_WIDTH + 1 {
        graphics::line(ctx,
                       &[Point2::new(HORIZONTAL_PADDING as f32, (VERTICAL_PADDING + i * GRID_CELL_DIM) as f32),
                         Point2::new((WINDOW_WIDTH - HORIZONTAL_PADDING) as f32, (VERTICAL_PADDING + i * GRID_CELL_DIM) as f32)],
                       GRID_LINE_WIDTH as f32)?;
    }
    Ok(())
}

fn screen_to_grid_coord((screen_x, screen_y): (u32, u32)) -> Option<(u32, u32)> {
    if screen_x < HORIZONTAL_PADDING || screen_x > WINDOW_WIDTH - HORIZONTAL_PADDING ||
       screen_y < VERTICAL_PADDING || screen_y > WINDOW_HEIGHT - VERTICAL_PADDING {
        return None;
    }
    Some(((screen_x - HORIZONTAL_PADDING) / GRID_CELL_DIM, (screen_y - VERTICAL_PADDING) / GRID_CELL_DIM))
}

// Screen coord is the top left hand corner of the cell, not including line width
fn grid_to_screen_coord((grid_x, grid_y): (u32, u32)) -> (u32, u32) {
    (HORIZONTAL_PADDING + grid_x * GRID_CELL_DIM + GRID_LINE_WIDTH / 2,
     VERTICAL_PADDING + grid_y * GRID_CELL_DIM + GRID_LINE_WIDTH / 2)
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, FPS) {
            self.konrad_tick += 2.0 / (FPS as f32);
            if self.konrad_tick >= 5.0 {
                self.konrad_tick -= 5.0;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        draw_grid(ctx)?;

        // Draw fps counter
        let fps = (ggez::timer::get_fps(ctx) as u32).to_string();
        let fps_txt = graphics::Text::new(ctx, &fps, &self.font)?;
        fps_txt.draw(ctx, Point2::new(WINDOW_WIDTH as f32 - 40.0, 20.0), 0.0)?;

        // Draw walls
        self.walls_sb.draw(ctx, Point2::new(0.0, 0.0), 0.0)?;

        // Draw highlighted grid cell
        match screen_to_grid_coord(self.mouse_coords) {
            Some((grid_x, grid_y)) => {
                let (rect_x, rect_y) = grid_to_screen_coord((grid_x, grid_y));
                let old_color = graphics::get_color(ctx);
                graphics::set_color(ctx, graphics::Color::from_rgb(234, 152, 174))?;
                graphics::rectangle(ctx,
                                    DrawMode::Fill,
                                    Rect::new(rect_x as f32, rect_y as f32,
                                              (GRID_CELL_DIM - GRID_LINE_WIDTH) as f32,
                                              (GRID_CELL_DIM - GRID_LINE_WIDTH) as f32))?;
                graphics::set_color(ctx, old_color)?;
            }
            None => ()
        }

        // Draw animated sprite
        let screen_coord = grid_to_screen_coord((0, 0));
        self.konrad_imgs[self.konrad_tick as usize].draw_ex(ctx, DrawParam {
            dest: Point2::new(screen_coord.0 as f32, screen_coord.1 as f32),
            ..DrawParam::default()
        })?;
        graphics::present(ctx);
        timer::yield_now();
        Ok(())
    }

    fn mouse_motion_event(&mut self,
                          _ctx: &mut Context,
                          _state: MouseState,
                          x: i32,
                          y: i32,
                          _xrel: i32,
                          _yrel: i32) {
        self.mouse_coords = (x as u32, y as u32);
    }
}

fn main() {
    let mut cb = ContextBuilder::new("feg", "feg")
        .window_setup(WindowSetup::default().title("FEG"))
        .window_mode(WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT));

    println!("{}", env!("CARGO_MANIFEST_DIR"));
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    } else {
        println!("Missing CARGO_MANIFEST_DIR environment variable");
        std::process::exit(1);
    }

    let ctx = &mut cb.build().unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    let (paths, costs) = compute_path_costs((0, 0), &state.walls);
    println!("{:?}", get_path((4, 2), paths));
    event::run(ctx, state).unwrap();
}
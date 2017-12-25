extern crate ggez;

use ggez::*;
use ggez::event::*;
use ggez::graphics::{DrawMode, Point2};
use ggez::conf::{WindowMode, WindowSetup};
use std::env;
use std::path;

fn world_to_screen_coords(screen_width: u32, screen_height: u32, point: Point2) -> Point2 {
    let width = screen_width as f32;
    let height = screen_height as f32;
    let x = point.x + width / 2.0;
    let y = height - (point.y + height / 2.0);
    Point2::new(x, y)
}

struct MainState {
    mouse_coords: (u32, u32),
    font: graphics::Font
}

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 800;
const VERTICAL_PADDING: u32 = 50;
const HORIZONTAL_PADDING: u32 = 290;
const GRID_LINE_WIDTH: u32 = 2;  // should be even
const GRID_N_CELL_WIDTH: u32 = 10;  // number of horizontal grid cells
const GRID_N_CELL_HEIGHT: u32 = 10;  // number of verical grid cells
const GRID_CELL_DIM: u32 = (WINDOW_HEIGHT - 2 * VERTICAL_PADDING) / GRID_N_CELL_HEIGHT;

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(MainState {
            mouse_coords: (0, 0),
            font: graphics::Font::new(ctx, "/DejaVuSerif.ttf", 10)?
        })
    }

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

fn grid_to_screen_coord((grid_x, grid_y): (u32, u32)) -> (u32, u32) {
    (HORIZONTAL_PADDING + grid_x * GRID_CELL_DIM + GRID_LINE_WIDTH / 2,
     VERTICAL_PADDING + grid_y * GRID_CELL_DIM + GRID_LINE_WIDTH / 2)
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        draw_grid(ctx)?;
        let fps = (ggez::timer::get_fps(ctx) as u32).to_string();
        let fps_txt = graphics::Text::new(ctx, &fps, &self.font)?;
        graphics::draw(ctx, &fps_txt, Point2::new(WINDOW_WIDTH as f32 - 40.0, 20.0), 0.0)?;
        match screen_to_grid_coord(self.mouse_coords) {
            Some((grid_x, grid_y)) => {
                let (rect_x, rect_y) = grid_to_screen_coord((grid_x, grid_y));
                let old_color = graphics::get_color(ctx);
                graphics::set_color(ctx, graphics::Color::from_rgba(255, 255, 255, 100))?;
                graphics::rectangle(ctx,
                                    DrawMode::Fill,
                                    graphics::Rect::new(rect_x as f32, rect_y as f32,
                                                        (GRID_CELL_DIM - GRID_LINE_WIDTH) as f32,
                                                        (GRID_CELL_DIM - GRID_LINE_WIDTH) as f32))?;
                graphics::set_color(ctx, old_color);
            }
            None => ()
        }
        graphics::present(ctx);
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

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    }

    let ctx = &mut cb.build().unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
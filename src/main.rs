extern crate ggez;
#[macro_use]
extern crate indoc;

mod dataparser;
mod pathfinding;
mod mainstate;

use ggez::{event, graphics, timer, Context, ContextBuilder, GameResult};
use ggez::event::{EventHandler, MouseState};
use ggez::graphics::{DrawMode, DrawParam, Drawable, Point2, Rect};
use ggez::conf::{WindowMode, WindowSetup};
use std::env;
use std::path;

use mainstate::*;

fn draw_grid(ctx: &mut Context, main_state: &MainState) -> GameResult<()> {
    for i in 0..main_state.grid_n_cell_height + 1 {
        graphics::line(
            ctx,
            &[
                Point2::new(
                    (main_state.horizontal_padding + i * main_state.grid_cell_dim) as f32,
                    main_state.vertical_padding as f32,
                ),
                Point2::new(
                    (main_state.horizontal_padding + i * main_state.grid_cell_dim) as f32,
                    (main_state.window_height - main_state.vertical_padding) as f32,
                ),
            ],
            main_state.grid_line_width as f32,
        )?;
    }
    for i in 0..main_state.grid_n_cell_width + 1 {
        graphics::line(
            ctx,
            &[
                Point2::new(
                    main_state.horizontal_padding as f32,
                    (main_state.vertical_padding + i * main_state.grid_cell_dim) as f32,
                ),
                Point2::new(
                    (main_state.window_width - main_state.horizontal_padding) as f32,
                    (main_state.vertical_padding + i * main_state.grid_cell_dim) as f32,
                ),
            ],
            main_state.grid_line_width as f32,
        )?;
    }
    Ok(())
}


impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, self.fps) {
            self.konrad_tick += 2.0 / (self.fps as f32);
            if self.konrad_tick >= 5.0 {
                self.konrad_tick -= 5.0;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        draw_grid(ctx, self)?;

        // Draw fps counter
        let fps = (ggez::timer::get_fps(ctx) as u32).to_string();
        let fps_txt = graphics::Text::new(ctx, &fps, &self.font)?;
        fps_txt.draw(ctx, Point2::new(self.window_width as f32 - 40.0, 20.0), 0.0)?;

        // Draw walls
        self.walls_sb.draw(ctx, Point2::new(0.0, 0.0), 0.0)?;

        // Draw highlighted grid cell
        match self.screen_to_grid_coord(self.mouse_coords) {
            Some((grid_x, grid_y)) => {
                let mut cpath_segments = self.cpath_to_segments(pathfinding::consolidate_path(
                    pathfinding::get_path((grid_x, grid_y), &self.paths),
                ));
                for segment in cpath_segments.chunks(2) {
                    let start = segment[0];
                    let end = segment[1];
                    graphics::line(
                        ctx,
                        &[
                            Point2::new(start.0 as f32, start.1 as f32),
                            Point2::new(end.0 as f32, end.1 as f32),
                        ],
                        self.path_line_width as f32,
                    )?;
                }
                let (rect_x, rect_y) = self.grid_to_screen_coord((grid_x, grid_y));
                let old_color = graphics::get_color(ctx);
                graphics::set_color(ctx, graphics::Color::from_rgb(234, 152, 174))?;
                graphics::rectangle(
                    ctx,
                    DrawMode::Fill,
                    Rect::new(
                        rect_x as f32,
                        rect_y as f32,
                        (self.grid_cell_dim - self.grid_line_width) as f32,
                        (self.grid_cell_dim - self.grid_line_width) as f32,
                    ),
                )?;
                graphics::set_color(ctx, old_color)?;
            }
            None => (),
        }

        // Draw animated sprite
        let screen_coord = self.grid_to_screen_coord((0, 0));
        self.konrad_imgs[self.konrad_tick as usize].draw_ex(
            ctx,
            DrawParam {
                dest: Point2::new(screen_coord.0 as f32, screen_coord.1 as f32),
                ..DrawParam::default()
            },
        )?;
        graphics::present(ctx);
        timer::yield_now();
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        _state: MouseState,
        x: i32,
        y: i32,
        _xrel: i32,
        _yrel: i32,
    ) {
        self.mouse_coords = (x as u32, y as u32);
    }
}

fn main() {
    let window_width = 1280;
    let window_height = 800;
    let mut cb = ContextBuilder::new("feg", "feg")
        .window_setup(WindowSetup::default().title("FEG"))
        .window_mode(WindowMode::default().dimensions(window_width, window_height));

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
    let state = &mut MainState::new(ctx, window_width, window_height).unwrap();
    event::run(ctx, state).unwrap();
}

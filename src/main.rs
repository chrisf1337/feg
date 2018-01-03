#![feature(vec_remove_item)]
extern crate ggez;
#[cfg(test)]
#[macro_use]
extern crate indoc;
#[cfg(test)]
#[macro_use]
extern crate maplit;
extern crate num;

#[macro_use]
mod utils;
mod dataparser;
mod pathfinding;
mod mainstate;
mod terrain;
mod unit;

use ggez::{event, graphics, timer, Context, ContextBuilder, GameResult};
use ggez::event::{EventHandler, MouseButton, MouseState};
use ggez::graphics::{Color, DrawMode, DrawParam, Drawable, Image, Point2};
use ggez::conf::{WindowMode, WindowSetup};
use std::env;
use std::path;
use num::rational::Ratio;
use num::Zero;

use mainstate::*;

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, self.fps) {
            let unit = self.units[&(3, 3)].clone();
            let mut unit = unit.borrow_mut();
            unit.animation_tick += 2.0 / (self.fps as f32);

            if unit.animation_tick >= 5.0 {
                unit.animation_tick -= 5.0;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        self.draw_grid(ctx)?;

        // Draw fps counter
        let fps = (ggez::timer::get_fps(ctx) as u32).to_string();
        let fps_txt = graphics::Text::new(ctx, &fps, &self.font)?;
        fps_txt.draw(ctx, Point2::new(self.window_width as f32 - 40.0, 20.0), 0.0)?;

        // Draw terrain
        self.wall_sb.draw(ctx, Point2::new(0.0, 0.0), 0.0)?;
        self.sand_sb.draw(ctx, Point2::new(0.0, 0.0), 0.0)?;

        let unit = self.units[&(3, 3)].clone();
        let unit = unit.borrow();

        for &coord in unit.reachable_coords.iter() {
            let old_color = graphics::get_color(ctx);
            graphics::set_color(ctx, Color::from_rgba(255, 84, 163, 60))?;
            let (x, y) = self.grid_to_screen_coord(coord);
            graphics::rectangle(
                ctx,
                DrawMode::Fill,
                graphics::Rect {
                    x: x as f32,
                    y: y as f32,
                    w: (self.grid_cell_dim - self.grid_line_width) as f32,
                    h: (self.grid_cell_dim - self.grid_line_width) as f32,
                },
            )?;
            graphics::set_color(ctx, old_color)?;
        }

        // Draw selection
        if let Some((grid_x, grid_y)) = self.selection {
            if self.selection != self.screen_to_grid_coord(self.mouse_coords) {
                let (screen_x, screen_y) = self.grid_to_screen_coord((grid_x, grid_y));
                let old_color = graphics::get_color(ctx);
                graphics::set_color(ctx, graphics::Color::from_rgb(255, 255, 0))?;
                self.cursor_img
                    .draw(ctx, Point2::new(screen_x as f32, screen_y as f32), 0.0)?;
                graphics::set_color(ctx, old_color)?;
            }
        }

        // Draw highlighted grid cell and path
        match self.screen_to_grid_coord(self.mouse_coords) {
            Some((grid_x, grid_y)) => {
                let mut cpath_segments = self.cpath_to_segments(pathfinding::consolidate_path(
                    pathfinding::get_path((grid_x, grid_y), &(unit.paths)),
                ));
                if cpath_segments.len() > 1 {
                    // Don't draw path when cursor is on the unit itself
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
                }
                let (rect_x, rect_y) = self.grid_to_screen_coord((grid_x, grid_y));
                let old_color = graphics::get_color(ctx);
                graphics::set_color(ctx, graphics::Color::from_rgb(255, 0, 0))?;
                self.cursor_img
                    .draw(ctx, Point2::new(rect_x as f32, rect_y as f32), 0.0)?;
                graphics::set_color(ctx, old_color)?;
            }
            None => (),
        }

        // Draw animated sprite
        let screen_coord = self.grid_to_screen_coord((3, 3));
        unit.animation_sprites[unit.animation_tick as usize].draw_ex(
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

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: i32, y: i32) {
        match button {
            MouseButton::Left => {
                self.selection = self.screen_to_grid_coord((x as u32, y as u32));
                match self.selection {
                    Some(grid_coord) => match self.units.get(&grid_coord) {
                        Some(unit) => self.selected_unit = Some(unit.clone()),
                        None => self.selected_unit = None,
                    },
                    None => self.selected_unit = None,
                }
            }
            _ => (),
        }
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
    state.add_unit(
        1,
        5,
        (3, 3),
        vec![
            Image::new(ctx, "/konrad-commander.png"),
            Image::new(ctx, "/konrad-commander-attack-1.png"),
            Image::new(ctx, "/konrad-commander-attack-2.png"),
            Image::new(ctx, "/konrad-commander-attack-3.png"),
            Image::new(ctx, "/konrad-commander-attack-4.png"),
        ].into_iter()
            .map(|x| x.unwrap())
            .collect(),
    );

    {
        let unit = state.units[&(3, 3)].clone();
        let unit = unit.borrow();
        for y in 0..10 {
            for x in 0..10 {
                match unit.costs.get(&(x, y)) {
                    Some(dist) => {
                        if dist == &Ratio::zero() {
                            print!("  S  ")
                        } else {
                            print!("{:.2} ", utils::rat_to_f32(dist))
                        }
                    }
                    None => print!("---- "),
                }
            }
            println!("");
        }
        println!("{:?}", unit.boundary);
        println!(
            "{:?}",
            pathfinding::find_boundary_neighbor_directions(
                &(unit.boundary),
                &(unit.reachable_coords),
                state.grid_n_cell_width,
                state.grid_n_cell_height
            )
        );
    }
    event::run(ctx, state).unwrap();
}

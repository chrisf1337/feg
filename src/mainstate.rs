use ggez::{graphics, Context, GameResult};
use ggez::graphics::{DrawParam, Drawable, Font, Image, Point2, Text};
use ggez::graphics::spritebatch::*;
use std::collections::HashMap;
use dataparser;
use pathfinding;
use terrain::Terrain;
use unit::Unit;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct MainState {
    pub mouse_coords: (u32, u32),
    pub font: Font,
    pub terrain: Vec<Vec<Terrain>>,
    pub wall_sb: SpriteBatch,
    pub sand_sb: SpriteBatch,
    pub fps: u32,
    pub window_width: u32,
    pub window_height: u32,
    pub vertical_padding: u32,
    pub horizontal_padding: u32,
    pub grid_line_width: u32,
    pub grid_n_cell_width: u32,
    pub grid_n_cell_height: u32,
    pub grid_cell_dim: u32,
    pub path_line_width: u32,
    pub cursor_img: Image,
    pub selection: Option<(u32, u32)>,
    pub grid_coord_to_unit_map: HashMap<(u32, u32), ()>,
    pub number_texts: Vec<Text>,
    pub units: HashMap<(u32, u32), Rc<RefCell<Unit>>>,
    pub selected_unit: Option<Rc<RefCell<Unit>>>,
}

impl MainState {
    pub fn new(ctx: &mut Context, window_width: u32, window_height: u32) -> GameResult<Self> {
        let wall_sb = SpriteBatch::new(Image::new(ctx, "/wall.png")?);
        let sand_sb = SpriteBatch::new(Image::new(ctx, "/sand.png")?);
        let terrain = dataparser::parse_walls("terrain.txt", 10, 10)?;

        let vertical_padding = 30;
        let grid_n_cell_width = 10; // number of horizontal grid cells
        let grid_n_cell_height = 10; // number of verical grid cells

        let font = Font::new(ctx, "/DejaVuSerif.ttf", 10)?;
        // Precompile Texts because Text::new() is expensive.
        let number_texts = (0..10)
            .into_iter()
            .map(|i| Text::new(ctx, &i.to_string(), &font).unwrap())
            .collect();

        let mut main_state = MainState {
            mouse_coords: (0, 0),
            font,
            terrain: terrain.clone(),
            wall_sb,
            sand_sb,

            fps: 60,
            window_width: window_width,
            window_height: window_height,
            horizontal_padding: 270,
            vertical_padding,
            grid_line_width: 2, // should be even
            grid_n_cell_width,
            grid_n_cell_height,

            // 74. This includes the line width.
            grid_cell_dim: (window_height - 2 * vertical_padding) / grid_n_cell_height,

            // Width of the line used to draw the path indicator.
            path_line_width: 10,
            cursor_img: Image::new(ctx, "/cursor.png")?,
            selection: None,
            grid_coord_to_unit_map: HashMap::new(),
            number_texts,

            units: HashMap::new(),
            selected_unit: None,
        };

        for (x, col) in terrain.iter().enumerate() {
            for (y, terrain_type) in col.iter().enumerate() {
                let (rect_x, rect_y) = main_state.grid_to_screen_coord((x as u32, y as u32));
                match terrain_type {
                    &Terrain::Wall => {
                        main_state.wall_sb.add(DrawParam {
                            dest: Point2::new(rect_x as f32, rect_y as f32),
                            ..DrawParam::default()
                        });
                    }
                    &Terrain::Sand => {
                        main_state.sand_sb.add(DrawParam {
                            dest: Point2::new(rect_x as f32, rect_y as f32),
                            ..DrawParam::default()
                        });
                    }
                    _ => (),
                }
            }
        }

        Ok(main_state)
    }

    pub fn add_unit(
        &mut self,
        id: u32,
        movement_range: u32,
        location: (u32, u32),
        animation_sprites: Vec<Image>,
    ) {
        let (paths, costs, boundary, reachable_coords) = pathfinding::compute_path_costs(
            location,
            &self.terrain,
            self.grid_n_cell_width,
            self.grid_n_cell_height,
            movement_range,
        );
        let unit = Unit::new(
            id,
            movement_range,
            location,
            animation_sprites,
            paths,
            costs,
            boundary,
            reachable_coords,
        );
        self.units.insert(location, Rc::new(RefCell::new(unit)));
    }

    pub fn draw_grid(&self, ctx: &mut Context) -> GameResult<()> {
        for i in 0..self.grid_n_cell_height + 1 {
            graphics::line(
                ctx,
                &[
                    Point2::new(
                        (self.horizontal_padding + i * self.grid_cell_dim) as f32,
                        self.vertical_padding as f32,
                    ),
                    Point2::new(
                        (self.horizontal_padding + i * self.grid_cell_dim) as f32,
                        (self.window_height - self.vertical_padding) as f32,
                    ),
                ],
                self.grid_line_width as f32,
            )?;
        }
        for i in 0..self.grid_n_cell_width + 1 {
            graphics::line(
                ctx,
                &[
                    Point2::new(
                        self.horizontal_padding as f32,
                        (self.vertical_padding + i * self.grid_cell_dim) as f32,
                    ),
                    Point2::new(
                        (self.window_width - self.horizontal_padding) as f32,
                        (self.vertical_padding + i * self.grid_cell_dim) as f32,
                    ),
                ],
                self.grid_line_width as f32,
            )?;
        }

        // Draw x and y labels
        for x in 0..10 {
            self.number_texts[x].draw(
                ctx,
                Point2::new(
                    (self.horizontal_padding + self.grid_cell_dim / 2 - 3
                        + x as u32 * self.grid_cell_dim) as f32,
                    (self.vertical_padding - 20) as f32,
                ),
                0.0,
            )?;
        }
        for y in 0..10 {
            self.number_texts[y].draw(
                ctx,
                Point2::new(
                    (self.horizontal_padding - 20) as f32,
                    (self.vertical_padding + self.grid_cell_dim / 2 - 2
                        + y as u32 * self.grid_cell_dim) as f32,
                ),
                0.0,
            )?;
        }

        Ok(())
    }

    // Includes pixels in the line of the grid
    pub fn screen_to_grid_coord(&self, (screen_x, screen_y): (u32, u32)) -> Option<(u32, u32)> {
        if screen_x < self.horizontal_padding
            || screen_x > self.window_width - self.horizontal_padding
            || screen_y < self.vertical_padding
            || screen_y > self.window_height - self.vertical_padding
        {
            return None;
        }
        Some((
            (screen_x - self.horizontal_padding) / self.grid_cell_dim,
            (screen_y - self.vertical_padding) / self.grid_cell_dim,
        ))
    }

    // Screen coord is the top left hand corner of the cell, not including line
    // width (i.e. if we start drawing at the coord returned by this function,
    // we will not overlap with the grid line)
    pub fn grid_to_screen_coord(&self, (grid_x, grid_y): (u32, u32)) -> (u32, u32) {
        (
            self.horizontal_padding + grid_x * self.grid_cell_dim + self.grid_line_width / 2,
            self.vertical_padding + grid_y * self.grid_cell_dim + self.grid_line_width / 2,
        )
    }

    // Screen coord is the center of the cell
    pub fn grid_to_screen_coord_center(&self, (grid_x, grid_y): (u32, u32)) -> (u32, u32) {
        let (x, y) = self.grid_to_screen_coord((grid_x, grid_y));
        (
            x + (self.grid_cell_dim - self.grid_line_width) / 2,
            y + (self.grid_cell_dim - self.grid_line_width) / 2,
        )
    }

    // Converts a consolidated path returned by pathfinding::consolidate_path()
    // into segments that can be drawn by iterating over pairs of elements in
    // the returned vec. We need to account for the width of the line used to
    // draw the path indicator.
    pub fn cpath_to_segments(&self, cpath: Vec<(u32, u32)>) -> Vec<(f32, f32)> {
        if cpath.len() < 2 {
            return cpath
                .into_iter()
                .map(|p| tuple_as!(self.grid_to_screen_coord_center(p), (x, f32), (y, f32)))
                .collect();
        }
        let mut segments = vec![
            tuple_as!(
                self.grid_to_screen_coord_center(cpath[0]),
                (x, f32),
                (y, f32)
            ),
        ];
        for window in cpath.windows(2).take(cpath.windows(2).len() - 1) {
            let prev = (*window)[0];
            let cur = (*window)[1];
            let (x, y) = self.grid_to_screen_coord_center(cur);
            // If y coords are equal, then the line is horizontal
            if cur.1 == prev.1 {
                if cur.0 < prev.0 {
                    segments.push(((x - self.path_line_width / 2) as f32, y as f32));
                } else {
                    segments.push(((x + self.path_line_width / 2) as f32, y as f32));
                }
                segments.push((x as f32, y as f32));
            } else {
                if cur.1 < prev.1 {
                    segments.push((x as f32, (y - self.path_line_width / 2) as f32));
                } else {
                    segments.push((x as f32, (y + self.path_line_width / 2) as f32));
                }
                segments.push((x as f32, y as f32));
            }
        }
        segments.push(tuple_as!(
            self.grid_to_screen_coord_center(cpath[cpath.len() - 1]),
            (x, f32),
            (y, f32)
        ));
        segments
    }
}

use ggez::{Context, GameResult};
use ggez::graphics::{DrawParam, Font, Image, Point2};
use ggez::graphics::spritebatch::*;
use std::collections::HashMap;
use dataparser;
use pathfinding;

#[derive(Debug)]
pub struct MainState {
    pub mouse_coords: (u32, u32),
    pub font: Font,
    pub konrad_imgs: Vec<Image>,
    pub konrad_tick: f32,
    pub walls: Vec<Vec<bool>>,
    pub walls_sb: SpriteBatch,
    pub fps: u32,
    pub window_width: u32,
    pub window_height: u32,
    pub vertical_padding: u32,
    pub horizontal_padding: u32,
    pub grid_line_width: u32,
    pub grid_n_cell_width: u32,
    pub grid_n_cell_height: u32,
    pub grid_cell_dim: u32,
    pub paths: HashMap<(u32, u32), (u32, u32)>,
    pub costs: HashMap<(u32, u32), u32>,
}

impl MainState {
    pub fn new(ctx: &mut Context, window_width: u32, window_height: u32) -> GameResult<Self> {
        let konrad_imgs = vec![
            Image::new(ctx, "/konrad-commander.png")?,
            Image::new(ctx, "/konrad-commander-attack-1.png")?,
            Image::new(ctx, "/konrad-commander-attack-2.png")?,
            Image::new(ctx, "/konrad-commander-attack-3.png")?,
            Image::new(ctx, "/konrad-commander-attack-4.png")?,
        ];
        let walls_sb = SpriteBatch::new(Image::new(ctx, "/wall.png")?);
        let walls = dataparser::parse_walls("walls.txt", 10, 10)?;

        let vertical_padding = 30;
        let grid_n_cell_width = 10; // number of horizontal grid cells
        let grid_n_cell_height = 10; // number of verical grid cells

        let (paths, costs) =
            pathfinding::compute_path_costs((0, 0), &walls, grid_n_cell_width, grid_n_cell_height);

        let mut main_state = MainState {
            mouse_coords: (0, 0),
            font: Font::new(ctx, "/DejaVuSerif.ttf", 10)?,
            konrad_imgs,
            konrad_tick: 0.0,
            walls: walls.clone(),
            walls_sb,
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
            paths,
            costs,
        };

        for (x, row) in walls.iter().enumerate() {
            for (y, is_wall) in row.iter().enumerate() {
                if *is_wall {
                    let (rect_x, rect_y) = main_state.grid_to_screen_coord((x as u32, y as u32));
                    main_state.walls_sb.add(DrawParam {
                        dest: Point2::new(rect_x as f32, rect_y as f32),
                        ..DrawParam::default()
                    });
                }
            }
        }

        Ok(main_state)
    }

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
}

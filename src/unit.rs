use ggez::graphics::Image;
use std::collections::{HashMap, HashSet};
use num::Rational;

#[derive(Debug)]
pub struct Unit {
    pub id: u32,
    pub movement_range: u32,
    pub location: (u32, u32),
    pub animation_sprites: Vec<Image>,
    pub animation_tick: f32,
    pub paths: HashMap<(u32, u32), (u32, u32)>,
    pub costs: HashMap<(u32, u32), Rational>,
    pub boundary: HashSet<(u32, u32)>,
    pub reachable_coords: HashSet<(u32, u32)>,
}

impl Unit {
    pub fn new(
        id: u32,
        movement_range: u32,
        location: (u32, u32),
        animation_sprites: Vec<Image>,
        paths: HashMap<(u32, u32), (u32, u32)>,
        costs: HashMap<(u32, u32), Rational>,
        boundary: HashSet<(u32, u32)>,
        reachable_coords: HashSet<(u32, u32)>,
    ) -> Self {
        Unit {
            id,
            movement_range,
            location,
            animation_sprites,
            animation_tick: 0.0,
            paths,
            costs,
            boundary,
            reachable_coords,
        }
    }
}

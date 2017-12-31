#[derive(Eq, PartialEq, Hash, Debug)]
pub struct Unit {
    pub id: u32,
    pub movement_range: u32,
}

impl Unit {
    pub fn new() -> Self {
        Unit {
            id: 1,
            movement_range: 5,
        }
    }
}

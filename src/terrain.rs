use decimal;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Terrain {
    Wall,
    Sand,
    None,
}

impl Terrain {
    pub fn cost(&self) -> decimal::d128 {
        d128!(1.0)
    }
}

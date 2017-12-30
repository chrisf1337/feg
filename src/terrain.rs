use num::{One, Rational};
use num::rational::Ratio;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Terrain {
    Wall,
    Sand,
    None,
}

impl Terrain {
    pub fn cost(&self) -> Rational {
        match self {
            &Terrain::Sand => Ratio::new(25, 10),
            &Terrain::None => Ratio::one(),
            &Terrain::Wall => unreachable!(),
        }
    }
}

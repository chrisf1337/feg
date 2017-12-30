use num::Rational;
use num::rational::Ratio;
macro_rules! tuple_as {
    ($t: expr, $(($var: ident, $ty: ty)),*) => {
        {
            let ($($var,)*) = $t;
            ($($var as $ty,)*)
        }
    }
}

pub fn rat_to_f32(rat: Rational) -> f32 {
    *rat.numer() as f32 / *rat.denom() as f32
}

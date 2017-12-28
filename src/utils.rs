macro_rules! tuple_as {
    ($t: expr, $(($var: ident, $ty: ty)),*) => {
        {
            let ($($var,)*) = $t;
            ($($var as $ty,)*)
        }
    }
}

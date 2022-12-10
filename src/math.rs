use num_traits::real::Real;

use crate::Coords;

pub(crate) fn is_left_of_line<C: Real>(c_min: Coords<C>, c_max: Coords<C>, c: Coords<C>) -> bool {
    if c.y() == c_max.y() {
        c.x() < c_max.x()
    } else if c.y() == c_min.y() {
        c.x() < c_min.x()
    } else {
        (c_min.x() - c_max.x()) * (c.y() - c_max.y()) < (c_min.y() - c_max.y()) * (c.x() - c_max.x())
    }
}

pub(crate) fn math_n(n: usize, h: usize) -> usize {
    let mut nf = n as f64;
    for _ in 0..h {
        nf = nf.log2();
    }
    ((n as f64) / nf).ceil() as usize
}

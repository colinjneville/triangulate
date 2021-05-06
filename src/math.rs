use crate::{Vertex, VertexExt};
use num_traits::Zero;

pub(crate) fn is_left_of_line<V: Vertex>(v_min: &VertexExt<V>, v_max: &VertexExt<V>, v: &VertexExt<V>) -> bool {
    if v.y() == v_max.y() {
        v.x() < v_max.x()
    } else if v.y() == v_min.y() {
        v.x() < v_min.x()
    } else {
        v_max.cross(v_min, v) < V::Coordinate::zero()
    }
}

pub(crate) fn math_n(n: usize, h: usize) -> usize {
    let mut nf = n as f64;
    for _ in 0..h {
        nf = nf.log2();
    }
    ((n as f64) / nf).ceil() as usize
}

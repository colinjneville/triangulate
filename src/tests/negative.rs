use crate::{builders, Triangulate};

use super::util::VTest;

#[test]
#[should_panic]
fn incomplete_polygon() {
    let polygon: Vec<VTest> = vec![(0., 0.).into(), (1., 1.).into()];
    polygon.triangulate::<builders::VecVecFanBuilder<_>>(&mut Vec::new()).unwrap();
}

#[test]
#[should_panic]
fn overlapping_vertex() {
    // ___
    // \ /
    //  x
    // / \
    // ---
    let polygon: Vec<VTest> = vec![(-1., 1.).into(), (1., 1.).into(), (0., 0.).into(), (1., -1.).into(), (-1., -1.).into(), (0., 0.).into()];
    polygon.triangulate::<builders::VecVecFanBuilder<_>>(&mut Vec::new()).unwrap();
}

#[test]
#[should_panic]
fn overlapping_polygons() {
    // +------+
    // |    +---+
    // |    | | |
    // |    +---+
    // +------+
    let polygon_a: Vec<VTest> = vec![(0., 0.).into(), (0., 1.).into(), (1., 1.).into(), (1., 0.).into()];
    let polygon_b: Vec<VTest> = vec![(0.75, 0.25).into(), (0.75, 0.75).into(), (1.25, 0.75).into(), (1.25, 0.25).into()];
    vec![polygon_a, polygon_b].triangulate::<builders::VecVecFanBuilder<_>>(&mut Vec::new()).unwrap();
}
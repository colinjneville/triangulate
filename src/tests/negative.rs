use crate::{formats, Polygon, PolygonList};

#[test]
#[should_panic]
fn incomplete_polygon() {
    let polygon: Vec<[f32; 2]> = vec![[0., 0.], [1., 1.]];
    polygon.triangulate(formats::IndexedFanFormat::new(&mut Vec::<Vec<_>>::new())).unwrap();
}

#[test]
#[should_panic]
fn overlapping_vertex() {
    // ___
    // \ /
    //  x
    // / \
    // ---
    let polygon: Vec<[f32; 2]> = vec![[-1., 1.], [1., 1.], [0., 0.], [1., -1.], [-1., -1.], [0., 0.]];
    polygon.triangulate(formats::IndexedFanFormat::new(&mut Vec::<Vec<_>>::new())).unwrap();
}

#[test]
#[should_panic]
fn overlapping_polygons() {
    // +------+
    // |    +---+
    // |    | | |
    // |    +---+
    // +------+
    let polygon_a: Vec<[f32; 2]> = vec![[0., 0.], [0., 1.], [1., 1.], [1., 0.]];
    let polygon_b: Vec<[f32; 2]> = vec![[0.75, 0.25], [0.75, 0.75], [1.25, 0.75], [1.25, 0.25]];
    vec![polygon_a, polygon_b].triangulate(formats::IndexedFanFormat::new(&mut Vec::<Vec<_>>::new())).unwrap();
}
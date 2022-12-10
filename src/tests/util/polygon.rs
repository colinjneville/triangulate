//! Sample polygons

/// A square polygon
pub fn square() -> Vec<[f32; 2]> {
    vec![
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
    ]
}

/// A 4-pointed star polygon
pub fn star() -> Vec<[f32; 2]> {
    vec![
        [1.0, 0.0],
        [2.0, 2.0],
        [0.0, 1.0],
        [-2.0, 2.0],
        [-1.0, 0.0],
        [-2.0, -2.0],
        [0.0, -1.0],
        [2.0, -2.0],
    ]
}

/// A concave half frame polygon
pub fn half_frame() -> Vec<[f32; 2]> {
    vec![
        [0., 0.], 
        [0.05, 0.05], 
        [0.95, 0.05],
        [0.95, 0.95],
        [1., 1.],
        [1., 0.],
    ]
}

/// All polygons in this module
pub fn all() -> Vec<Vec<[f32; 2]>> {
    vec![square(), star(), half_frame()]
}
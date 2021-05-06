use super::vtest::VTest;

pub fn square() -> Vec<VTest> {
    vec![
        (0.0, 0.0).into(),
        (0.0, 1.0).into(),
        (1.0, 1.0).into(),
        (1.0, 0.0).into(),
    ]
}

pub fn star() -> Vec<VTest> {
    vec![
        (1.0, 0.0).into(),
        (2.0, 2.0).into(),
        (0.0, 1.0).into(),
        (-2.0, 2.0).into(),
        (-1.0, 0.0).into(),
        (-2.0, -2.0).into(),
        (0.0, -1.0).into(),
        (2.0, -2.0).into(),
    ]
}

pub fn half_frame() -> Vec<VTest> {
    vec![
        (0., 0.).into(), (0.05, 0.05).into(), (0.95, 0.05).into(), (0.95, 0.95).into(), (1., 1.).into(), (1., 0.).into()
    ]
}

pub fn all() -> Vec<Vec<VTest>> {
    vec![square(), star(), half_frame()]
}
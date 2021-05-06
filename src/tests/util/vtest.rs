use crate::inputs::Vertex;

#[derive(Default, Copy, Clone, PartialEq, PartialOrd)]
pub struct VTest {
    x: f32,
    y: f32,
}

impl VTest {
    pub fn new(x: f32, y: f32) -> Self { VTest {x, y} }
}

impl std::fmt::Debug for VTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl std::fmt::Display for VTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Vertex for VTest {
    type Coordinate = f32;

    #[inline(always)]
    fn x(&self) -> Self::Coordinate { self.x }

    #[inline(always)]
    fn y(&self) -> Self::Coordinate { self.y }
}

impl Into<VTest> for (f32, f32) {
    fn into(self) -> VTest {
        VTest::new(self.0, self.1)
    }
}
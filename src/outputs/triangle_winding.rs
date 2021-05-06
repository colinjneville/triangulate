/// The order the vertices in a triangle are listed in
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TriangleWinding {
    Counterclockwise,
    Clockwise,
}

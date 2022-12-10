/// The order the vertices in a polygon are listed in
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TriangleWinding {
    /// Counter-clockwise ordering
    Counterclockwise,
    /// Clockwise ordering
    Clockwise,
}

impl TriangleWinding {
    /// Reverse the winding order
    pub const fn reverse(self) -> Self {
        match self {
            TriangleWinding::Counterclockwise => TriangleWinding::Clockwise,
            TriangleWinding::Clockwise => TriangleWinding::Counterclockwise,
        }
    }
}

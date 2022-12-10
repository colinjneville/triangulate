use std::fmt;

use crate::{Vertex, VertexIndex, idx::{Idx, IdxDisplay}, math::is_left_of_line, nexus::Nexus, Coords};

#[derive(Clone)]
pub(crate) struct Segment<V: Vertex, Index: VertexIndex> {
    ni_min: Idx<Nexus<V, Index>>,
    ni_max: Idx<Nexus<V, Index>>,
    c_min: Coords<V::Coordinate>,
    c_max: Coords<V::Coordinate>,
}

impl<V: Vertex, Index: VertexIndex> fmt::Debug for Segment<V, Index> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Segment").field("ni_min", &self.ni_min).field("ni_max", &self.ni_max).field("c_min", &self.c_min).field("c_max", &self.c_max).finish()
    }
}

impl<V: Vertex, Index: VertexIndex> IdxDisplay for Segment<V, Index> {
    fn fmt(f: &mut std::fmt::Formatter<'_>, idx: usize) -> std::fmt::Result {
        write!(f, "s{}", idx)
    }
}

impl<V: Vertex, Index: VertexIndex> std::fmt::Display for Segment<V, Index>
where V::Coordinate: std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.ni_min, self.ni_max)
    }
}

impl<V: Vertex, Index: VertexIndex> Segment<V, Index> {
    pub fn new(ni_min: Idx<Nexus<V, Index>>, ni_max: Idx<Nexus<V, Index>>, c_min: Coords<V::Coordinate>, c_max: Coords<V::Coordinate>) -> Self {
        Self {
            ni_min,
            ni_max,
            c_min,
            c_max,
        }
    }

    pub fn ni_min(&self) -> Idx<Nexus<V, Index>> { self.ni_min }
    pub fn ni_max(&self) -> Idx<Nexus<V, Index>> { self.ni_max }

    pub fn is_on_left(&self, c: Coords<V::Coordinate>) -> bool {
        is_left_of_line(self.c_min, self.c_max, c)
    }
}

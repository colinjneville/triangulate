use crate::{PolygonList, PolygonListExt, Vertex, VertexIndex, idx::{Idx, IdxDisplay}, math::is_left_of_line, nexus::Nexus};

#[derive(Debug, Clone)]
pub(crate) struct Segment<V: Vertex, Index: VertexIndex> {
    ni_max: Idx<Nexus<V, Index>>,
    ni_min: Idx<Nexus<V, Index>>,
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
    pub fn new(ni_min: Idx<Nexus<V, Index>>, ni_max: Idx<Nexus<V, Index>>) -> Self {
        Self {
            ni_min,
            ni_max,
        }
    }

    pub fn ni_min(&self) -> Idx<Nexus<V, Index>> { self.ni_min }
    pub fn ni_max(&self) -> Idx<Nexus<V, Index>> { self.ni_max }

    pub fn is_on_left<'a, P: PolygonList<'a, Index=Index>>(&self, ps: PolygonListExt<'a, P>, ns: &[Nexus<V, Index>], vi: Index) -> bool {
        let n_min = &ns[self.ni_min];
        let n_max = &ns[self.ni_max];
        let v_min = &ps[n_min.vertex()];
        let v_max = &ps[n_max.vertex()];
        let v = &ps[vi];
        is_left_of_line(v_min, v_max, v)
    }
}

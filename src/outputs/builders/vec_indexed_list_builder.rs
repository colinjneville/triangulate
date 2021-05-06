use std::convert;

use crate::{ListBuilder, PolygonList, TriangulationError};


pub struct VecIndexedListBuilder<'f, 'a, P: PolygonList<'a>> 
where P::Vertex: Clone {
    list: &'f mut Vec<P::Index>,
    initial_vertex_count: usize,
}

impl<'f, 'a, P: PolygonList<'a>> ListBuilder<'a, P> for VecIndexedListBuilder<'f, 'a, P> 
where P::Vertex: Clone {
    type Initializer = &'f mut Vec<P::Index>;
    type Output = &'f mut [P::Index];
    type Error = convert::Infallible;

    fn new(list: Self::Initializer, _polygon_list: P) -> Result<Self, Self::Error> {
        let initial_vertex_count = list.len();
        let lb = Self {
            list,
            initial_vertex_count,
        };
        Ok(lb)
    }

    fn add_triangle(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error> {
        self.list.push(vi0);
        self.list.push(vi1);
        self.list.push(vi2);
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        Ok(&mut self.list[self.initial_vertex_count..])
    }

    fn fail(self, _error: &TriangulationError<Self::Error>) {
        self.list.truncate(self.initial_vertex_count);
    }
}

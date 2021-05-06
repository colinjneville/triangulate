use std::convert;

use crate::{ListBuilder, PolygonList, TriangulationError};


pub struct VecListBuilder<'f, 'a, P: PolygonList<'a>> 
where P::Vertex: Clone {
    polygon_list: P,
    list: &'f mut Vec<P::Vertex>,
    initial_vertex_count: usize,
}

impl<'f, 'a, P: PolygonList<'a>> ListBuilder<'a, P> for VecListBuilder<'f, 'a, P> 
where P::Vertex: Clone {
    type Initializer = &'f mut Vec<P::Vertex>;
    type Output = &'f mut [P::Vertex];
    type Error = convert::Infallible;

    fn new(list: Self::Initializer, polygon_list: P) -> Result<Self, Self::Error> {
        let initial_vertex_count = list.len();
        let lb = Self {
            polygon_list,
            list,
            initial_vertex_count,
        };
        Ok(lb)
    }

    fn add_triangle(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error> {
        self.list.push(self.polygon_list.get_vertex(vi0).clone());
        self.list.push(self.polygon_list.get_vertex(vi1).clone());
        self.list.push(self.polygon_list.get_vertex(vi2).clone());
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        Ok(&mut self.list[self.initial_vertex_count..])
    }

    fn fail(self, _error: &TriangulationError<Self::Error>) {
        self.list.truncate(self.initial_vertex_count);
    }
}

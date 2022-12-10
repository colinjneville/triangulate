use std::marker::PhantomData;

use crate::{ListFormat, PolygonList, TriangulationError, List, ListBuilder};

use super::GenericList;

/// A [ListFormat] which outputs the triangle list by their actual vertex values, not their indices
#[derive(Debug)]
pub struct DeindexedListFormat<'p, P: PolygonList<'p> + ?Sized, L: List<P::Vertex>>
where P::Vertex: Clone {
    list: L,
    _phantom: PhantomData<&'p P>,
}

impl<'p, P: PolygonList<'p> + ?Sized, L: List<P::Vertex>> DeindexedListFormat<'p, P, L> 
where P::Vertex: Clone {
    /// Create a deindexed format which stores its output in the given [List]
    pub fn new(list: L) -> Self {
        Self { list, _phantom: PhantomData, }
    }
}

impl <'p, P: PolygonList<'p> + ?Sized, L: List<P::Vertex>> ListFormat<'p, P> for DeindexedListFormat<'p, P, L> 
where P::Vertex: Clone {
    type Builder = DeindexedListBuilder<'p, P, L>;

    fn initialize(self, polygon_list: &'p P) -> Result<Self::Builder, <Self::Builder as ListBuilder<'p, P>>::Error> {
        DeindexedListBuilder::new(self.list, polygon_list)
    }
}

pub struct DeindexedListBuilder<'p, P: PolygonList<'p> + ?Sized, L: List<P::Vertex>> 
where P::Vertex: Clone {
    list: GenericList<L, P::Vertex>,
    polygon_list: &'p P,
}

impl<'p, P: PolygonList<'p> + ?Sized, L: List<P::Vertex>> DeindexedListBuilder<'p, P, L> 
where P::Vertex: Clone {
    fn new(list: L, polygon_list: &'p P) -> Result<Self, <Self as ListBuilder<'p, P>>::Error> {
        let list = GenericList::new(list);
        let l = Self {
            list,
            polygon_list,
        };

        Ok(l)
    }
}

impl<'p, P: PolygonList<'p> + ?Sized, L: List<P::Vertex>> ListBuilder<'p, P> for DeindexedListBuilder<'p, P, L> 
where P::Vertex: Clone {
    type Output = L;
    type Error = std::convert::Infallible;

    fn add_triangle(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error> {
        let (v0, v1, v2) = (self.polygon_list.get_vertex(vi0).clone(), self.polygon_list.get_vertex(vi1).clone(), self.polygon_list.get_vertex(vi2).clone());
        self.list.new_triangle(v0, v1, v2);
        Ok(())
    }
    
    fn build(self) -> Result<Self::Output, Self::Error> {
        Ok(self.list.build())
    }

    fn fail(self, _error: &TriangulationError<Self::Error>) {
        self.list.fail();
    }
}

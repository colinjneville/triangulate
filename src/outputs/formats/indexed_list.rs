use std::marker::PhantomData;

use crate::{ListFormat, PolygonList, TriangulationError, List, ListBuilder};

use super::GenericList;

/// A [ListFormat] which outputs the triangle list by its indices
#[derive(Debug)]
pub struct IndexedListFormat<'p, P: PolygonList<'p> + ?Sized, L: List<P::Index>> {
    list: L,
    _phantom: PhantomData<&'p P>,
}

impl<'p, P: PolygonList<'p> + ?Sized, L: List<P::Index>> IndexedListFormat<'p, P, L> {
    /// Create an indexed format which stores its output in the given [List]
    pub fn new(list: L) -> Self {
        Self { list, _phantom: PhantomData, }
    }
}

impl <'p, P: PolygonList<'p> + ?Sized, L: List<P::Index>> ListFormat<'p, P> for IndexedListFormat<'p, P, L> {
    type Builder = IndexedListBuilder<'p, P, L>;

    fn initialize(self, polygon_list: &'p P) -> Result<Self::Builder, <Self::Builder as ListBuilder<'p, P>>::Error> {
        IndexedListBuilder::new(self.list, polygon_list)
    }
}

pub struct IndexedListBuilder<'p, P: PolygonList<'p> + ?Sized, L: List<P::Index>> {
    list: GenericList<L, P::Index>,
}

impl<'p, P: PolygonList<'p> + ?Sized, L: List<P::Index>> IndexedListBuilder<'p, P, L> {
    fn new(list: L, _polygon_list: &'p P) -> Result<Self, <Self as ListBuilder<'p, P>>::Error> {
        let list = GenericList::new(list);
        let l = Self {
            list,
        };

        Ok(l)
    }
}

impl<'p, P: PolygonList<'p> + ?Sized, L: List<P::Index>> ListBuilder<'p, P> for IndexedListBuilder<'p, P, L> {
    type Output = L;
    type Error = std::convert::Infallible;

    fn add_triangle(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error> {
        self.list.new_triangle(vi0, vi1, vi2);
        Ok(())
    }
    
    fn build(self) -> Result<Self::Output, Self::Error> {
        Ok(self.list.build())
    }

    fn fail(self, _error: &TriangulationError<Self::Error>) {
        self.list.fail();
    }
}

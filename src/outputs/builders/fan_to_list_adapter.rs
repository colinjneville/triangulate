use std::marker::PhantomData;

use crate::{FanBuilder, ListBuilder, PolygonList, TriangleWinding, TriangulationError};

pub struct FanToListAdapter<'a, P: PolygonList<'a>, LB: ListBuilder<'a, P>> {
    list_builder: LB,
    vi0: P::Index,
    vi1: P::Index,
    _a: PhantomData<&'a ()>,
}

impl<'a, P: PolygonList<'a>, LB: ListBuilder<'a, P>> FanBuilder<'a, P> for FanToListAdapter<'a, P, LB> {
    type Initializer = LB::Initializer;
    type Output = LB::Output;
    type Error = LB::Error;

    const WINDING: TriangleWinding = LB::WINDING;

    fn new(initializer: Self::Initializer, polygon_list: P, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<Self, Self::Error>
    where Self: Sized {
        let mut list_builder = LB::new(initializer, polygon_list)?;
        list_builder.add_triangle(vi0.clone(), vi1, vi2.clone())?;
        Ok(Self {
            list_builder,
            vi0,
            vi1: vi2,
            _a: PhantomData,
        })
    }

    fn new_fan(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error> {
        self.vi0 = vi0.clone();
        self.vi1 = vi2.clone();
        self.list_builder.add_triangle(vi0, vi1, vi2)
    }

    fn extend_fan(&mut self, vi: P::Index) -> Result<(), Self::Error> {
        let vi1 = std::mem::replace(&mut self.vi1, vi.clone());
        self.list_builder.add_triangle(self.vi0.clone(), vi1, vi)
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        self.list_builder.build()
    }

    fn fail(self, error: &TriangulationError<Self::Error>) {
        self.list_builder.fail(error);
    }
}

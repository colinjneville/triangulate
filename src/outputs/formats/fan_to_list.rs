use std::marker::PhantomData;

use crate::{PolygonList, ListFormat, FanFormat, FanBuilder, ListBuilder, TriangleWinding};

/// A wrapper which adapts a [ListFormat] to work as a [FanFormat], which is required for [PolygonList::triangulate]
/// 
/// Constructed with [ListFormat::into_fan_format].
pub struct FanToListFormat<'p, P: PolygonList<'p> + ?Sized, LB: ListFormat<'p, P>>(LB, PhantomData<&'p P>);

impl<'p, P: PolygonList<'p> + ?Sized, LB: ListFormat<'p, P>> FanToListFormat<'p, P, LB> {
    pub(crate) fn new(list_builder: LB) -> Self {
        Self(list_builder, PhantomData)
    }
}

impl<'p, P: PolygonList<'p> + ?Sized, LB: ListFormat<'p, P>> FanFormat<'p, P> for FanToListFormat<'p, P, LB> {
    type Builder = FanToListBuilder<'p, P, LB::Builder>;

    fn initialize(self, polygon_list: &'p P, vi0: <P as PolygonList<'p>>::Index, vi1: <P as PolygonList<'p>>::Index, vi2: <P as PolygonList<'p>>::Index) -> Result<Self::Builder, <Self::Builder as FanBuilder<'p, P>>::Error> {
        let mut list_builder = self.0.initialize(polygon_list)?;
        list_builder.add_triangle(vi0.clone(), vi1, vi2.clone())?;
        
        let fan_builder = FanToListBuilder::new(list_builder, vi0, vi2);
        Ok(fan_builder)
    }
}

pub struct FanToListBuilder<'p, P: PolygonList<'p> + ?Sized, LB: ListBuilder<'p, P>> {
    list_builder: LB,
    vi0: P::Index,
    vi1: P::Index,
    _phantom: PhantomData<&'p P>,
}

impl<'p, P: PolygonList<'p> + ?Sized, LB: ListBuilder<'p, P>> FanToListBuilder<'p, P, LB> {
    fn new(list_builder: LB, vi0: P::Index, vi1: P::Index) -> Self {
        Self {
            list_builder,
            vi0,
            vi1,
            _phantom: PhantomData,
        }
    }
}

impl<'p, P: PolygonList<'p> + ?Sized, LB: ListBuilder<'p, P>> FanBuilder<'p, P> for FanToListBuilder<'p, P, LB> {
    type Output = LB::Output;
    type Error = LB::Error;

    const WINDING: TriangleWinding = <LB as ListBuilder<'p, P>>::WINDING;

    fn new_fan(&mut self, vi0: <P as PolygonList<'p>>::Index, vi1: <P as PolygonList<'p>>::Index, vi2: <P as PolygonList<'p>>::Index) -> Result<(), Self::Error> {
        self.vi0 = vi0.clone();
        self.vi1 = vi2.clone();
        self.list_builder.add_triangle(vi0, vi1, vi2)
    }

    fn extend_fan(&mut self, vi: <P as PolygonList<'p>>::Index) -> Result<(), Self::Error> {
        let vi1 = std::mem::replace(&mut self.vi1, vi.clone());
        self.list_builder.add_triangle(self.vi0.clone(), vi1, vi)
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        self.list_builder.build()
    }

    fn fail(self, error: &crate::TriangulationError<Self::Error>) {
        self.list_builder.fail(error);
    }
}

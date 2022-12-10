use std::marker::PhantomData;

use crate::{PolygonList, FanFormat, FanBuilder, TriangleWinding};

/// Wrapper which reverses the winding of a [FanFormat]
pub struct ReverseFanFormat<'p, P: PolygonList<'p> + ?Sized, FB: FanFormat<'p, P>>(FB, PhantomData<&'p P>);

impl<'p, P: PolygonList<'p> + ?Sized, FB: FanFormat<'p, P>> ReverseFanFormat<'p, P, FB> {
    pub(crate) fn new(fan_builder: FB) -> Self {
        Self(fan_builder, PhantomData)
    }
}

impl<'p, P: PolygonList<'p> + ?Sized, FB: FanFormat<'p, P>> FanFormat<'p, P> for ReverseFanFormat<'p, P, FB> {
    type Builder = ReverseFanBuilder<'p, P, FB::Builder>;

    fn initialize(self, polygon_list: &'p P, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<Self::Builder, <Self::Builder as FanBuilder<'p, P>>::Error> {
        let fb = self.0.initialize(polygon_list, vi0, vi1, vi2)?;
        Ok(ReverseFanBuilder(fb, PhantomData))
    }
}

/// Wrapper which reverses the winding of a [FanBuilder]
pub struct ReverseFanBuilder<'p, P: PolygonList<'p> + ?Sized, FB: FanBuilder<'p, P>>(FB, PhantomData<&'p P>);

impl<'p, P: PolygonList<'p> + ?Sized, FB: FanBuilder<'p, P>> FanBuilder<'p, P> for ReverseFanBuilder<'p, P, FB> {
    type Output = FB::Output;
    type Error = FB::Error;

    const WINDING: TriangleWinding = FB::WINDING.reverse();
    
    fn new_fan(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error> {
        self.0.new_fan(vi0, vi1, vi2)
    }

    fn extend_fan(&mut self, vi: P::Index) -> Result<(), Self::Error> {
        self.0.extend_fan(vi)
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        self.0.build()
    }

    fn fail(self, error: &crate::TriangulationError<Self::Error>) {
        self.0.fail(error);
    }
}
use std::marker::PhantomData;

use crate::{FanFormat, PolygonList, TriangulationError, outputs::fan_format::FanBuilder, Fan, Fans};

use super::GenericFans;

/// A [FanFormat] which outputs the triangle fans by their indices
#[derive(Debug)]
pub struct IndexedFanFormat<'p, P: PolygonList<'p> + ?Sized, FS: Fans>
where FS::Fan: Fan<P::Index> {
    fans: FS,
    _phantom: PhantomData<&'p P>,
}

impl<'p, P: PolygonList<'p> + ?Sized, FS: Fans> IndexedFanFormat<'p, P, FS>
where FS::Fan: Fan<P::Index> {
    /// Create an indexed format which stores its output in the given [Fans]
    pub fn new(fans: FS) -> Self {
        Self { fans, _phantom: PhantomData, }
    }
}

impl <'p, P: PolygonList<'p> + ?Sized, FS: Fans> FanFormat<'p, P> for IndexedFanFormat<'p, P, FS>
where FS::Fan: Fan<P::Index> {
    type Builder = IndexedFanBuilder<'p, P, FS>;

    fn initialize(self, polygon_list: &'p P, vi0: <P as PolygonList<'p>>::Index, vi1: <P as PolygonList<'p>>::Index, vi2: <P as PolygonList<'p>>::Index) -> Result<Self::Builder, <Self::Builder as FanBuilder<'p, P>>::Error> {
        IndexedFanBuilder::new(self.fans, polygon_list, vi0, vi1, vi2)
    }
}

pub struct IndexedFanBuilder<'p, P: PolygonList<'p> + ?Sized, FS: Fans> 
where FS::Fan: Fan<P::Index> {
    fans: GenericFans<FS, P::Index>,
}

impl<'p, P: PolygonList<'p> + ?Sized, FS: Fans> IndexedFanBuilder<'p, P, FS> 
where FS::Fan: Fan<P::Index> {
    fn new(fans: FS, _polygon_list: &'p P, vi0: <P as PolygonList<'p>>::Index, vi1: <P as PolygonList<'p>>::Index, vi2: <P as PolygonList<'p>>::Index) -> Result<Self, <Self as FanBuilder<'p, P>>::Error> {
        let fans = GenericFans::new(fans, vi0, vi1, vi2);
        let fb = Self {
            fans,
        };

        Ok(fb)
    }
}

impl<'p, P: PolygonList<'p> + ?Sized, FS: Fans> FanBuilder<'p, P> for IndexedFanBuilder<'p, P, FS>
where FS::Fan: Fan<P::Index> {
    type Output = FS;
    type Error = std::convert::Infallible;

    fn new_fan(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error> {
        self.fans.new_fan(vi0, vi1, vi2);
        Ok(())
    }

    fn extend_fan(&mut self, vi: P::Index) -> Result<(), Self::Error> {
        self.fans.extend_fan(vi);
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        Ok(self.fans.build())
    }

    fn fail(self, _error: &TriangulationError<Self::Error>) {
        self.fans.fail();
    }
}

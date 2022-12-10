use std::marker::PhantomData;

use crate::{Fan, PolygonList, Fans, FanFormat, FanBuilder, TriangulationError};

use super::GenericFans;

/// A [FanFormat] which outputs the triangle fans by their actual vertex values, not their indices
#[derive(Debug)]
pub struct DeindexedFanFormat<'p, P: PolygonList<'p> + ?Sized, FS: Fans>
where FS::Fan: Fan<P::Vertex>,
      P::Vertex: Clone {
    fans: FS,
    _phantom: PhantomData<&'p P>,
}

impl<'p, P: PolygonList<'p> + ?Sized, FS: Fans> DeindexedFanFormat<'p, P, FS>
where FS::Fan: Fan<P::Vertex>,
      P::Vertex: Clone {
    /// Create a deindexed format which stores its output in the given [Fans]
    pub fn new(fans: FS) -> Self {
        Self { fans , _phantom: PhantomData }
    }
}

impl <'p, P: PolygonList<'p> + ?Sized, FS: Fans> FanFormat<'p, P> for DeindexedFanFormat<'p, P, FS>
where FS::Fan: Fan<P::Vertex>,
      P::Vertex: Clone {
    type Builder = DeindexedFanBuilder<'p, P, FS>;

    fn initialize(self, polygon_list: &'p P, vi0: <P as PolygonList<'p>>::Index, vi1: <P as PolygonList<'p>>::Index, vi2: <P as PolygonList<'p>>::Index) -> Result<Self::Builder, <Self::Builder as FanBuilder<'p, P>>::Error> {
        DeindexedFanBuilder::new(self.fans, polygon_list, vi0, vi1, vi2)
    }
}

pub struct DeindexedFanBuilder<'p, P: PolygonList<'p> + ?Sized, FS: Fans>
where FS::Fan: Fan<P::Vertex>,
      P::Vertex: Clone {
    fans: GenericFans<FS, P::Vertex>,
    polygon_list: &'p P,
}

impl<'p, P: PolygonList<'p> + ?Sized, FS: Fans> DeindexedFanBuilder<'p, P, FS> 
where FS::Fan: Fan<P::Vertex>,
      P::Vertex: Clone {
    fn new(fans: FS, polygon_list: &'p P, vi0: <P as PolygonList<'p>>::Index, vi1: <P as PolygonList<'p>>::Index, vi2: <P as PolygonList<'p>>::Index) -> Result<Self, <Self as FanBuilder<'p, P>>::Error> {
        let (v0, v1, v2) = (polygon_list.get_vertex(vi0).clone(), polygon_list.get_vertex(vi1).clone(), polygon_list.get_vertex(vi2).clone());
        let fans = GenericFans::new(fans, v0, v1, v2);
        let fb = Self {
            fans,
            polygon_list
        };

        Ok(fb)
    }
}

impl<'p, P: PolygonList<'p> + ?Sized, FS: Fans> FanBuilder<'p, P> for DeindexedFanBuilder<'p, P, FS>
where FS::Fan: Fan<P::Vertex>,
      P::Vertex: Clone {
    type Output = FS;
    type Error = std::convert::Infallible;

    fn new_fan(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error> {
        let (v0, v1, v2) = (self.polygon_list.get_vertex(vi0).clone(), self.polygon_list.get_vertex(vi1).clone(), self.polygon_list.get_vertex(vi2).clone());
        self.fans.new_fan(v0, v1, v2);
        Ok(())
    }

    fn extend_fan(&mut self, vi: P::Index) -> Result<(), Self::Error> {
        self.fans.extend_fan(self.polygon_list.get_vertex(vi).clone());
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        Ok(self.fans.build())
    }

    fn fail(self, _error: &TriangulationError<Self::Error>) {
        self.fans.fail();
    }
}

use std::error;

use crate::{PolygonList, TriangleWinding, TriangulationError};

pub trait FanBuilder<'a, P: PolygonList<'a>> {
    type Initializer;
    type Output;
    type Error: error::Error;

    const WINDING: TriangleWinding = TriangleWinding::Counterclockwise;

    fn new(initializer: Self::Initializer, polygon_list: P, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<Self, Self::Error>
    where Self: Sized;

    fn new_fan(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error>;
    fn extend_fan(&mut self, vi: P::Index) -> Result<(), Self::Error>;

    fn build(self) -> Result<Self::Output, Self::Error>;
    fn fail(self, error: &TriangulationError<Self::Error>);
}

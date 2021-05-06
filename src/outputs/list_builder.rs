use std::error;

use crate::{PolygonList, TriangleWinding, TriangulationError};

pub trait ListBuilder<'a, P: PolygonList<'a>> {
    type Initializer;
    type Output;
    type Error: error::Error;

    const WINDING: TriangleWinding = TriangleWinding::Counterclockwise;

    fn new(initializer: Self::Initializer, polygon_list: P) -> Result<Self, Self::Error>
    where Self: Sized;

    fn add_triangle(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error>;

    fn build(self) -> Result<Self::Output, Self::Error>;
    fn fail(self, error: &TriangulationError<Self::Error>);
}
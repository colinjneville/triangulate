use std::error;

use crate::{PolygonList, TriangleWinding, TriangulationError, formats::ReverseFanFormat};

/// Describes the construction and layout of a triangle fans
pub trait FanFormat<'p, P: PolygonList<'p> + ?Sized> {
    /// The type responsible for constructing triangle fans.
    /// 
    /// This type can be `Self`, if you choose to implement both [FanFormat] and [FanBuilder] on the same type.
    type Builder: FanBuilder<'p, P> + Sized;

    /// Constructs a [FanFormat::Builder] with an initial triangle, optionally using a reference to the [PolygonList] being triangulated.
    fn initialize(self, polygon_list: &'p P, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<Self::Builder, <Self::Builder as FanBuilder<'p, P>>::Error>;

    /// Constructs a [FanFormat] with the opposite [TriangleWinding]
    fn reverse_winding(self) -> ReverseFanFormat<'p, P, Self>
    where Self: Sized {
        ReverseFanFormat::new(self)
    }
}

/// Performs the construction of triangle fans
pub trait FanBuilder<'p, P: PolygonList<'p> + ?Sized>: Sized {
    /// The triangle fans output type
    type Output;
    /// The error type when the builder fails
    type Error: error::Error;

    /// The winding direction this builder expects for triangles
    const WINDING: TriangleWinding = TriangleWinding::Counterclockwise;

    /// Starts a new fan with the given triangle
    fn new_fan(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error>;
    /// Extends the current fan with a triangle containing the given vertex
    fn extend_fan(&mut self, vi: P::Index) -> Result<(), Self::Error>;

    /// Called when triangulation has completed to get the resulting output
    fn build(self) -> Result<Self::Output, Self::Error>;

    /// Called when triangulation encounters an error.
    /// 
    /// Any required cleanup (e.g. removing the partial triangulation added to an existing [Vec]) should be done here
    fn fail(self, error: &TriangulationError<Self::Error>);
}

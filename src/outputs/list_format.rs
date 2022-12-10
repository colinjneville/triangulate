use std::error;

use crate::{PolygonList, TriangleWinding, TriangulationError, formats};

/// Describes the construction and layout of a triangle list
pub trait ListFormat<'p, P: PolygonList<'p> + ?Sized> {
    /// The type responsible for constructing the triangle list.
    /// 
    /// This type can be `Self`, if you choose to implement both [ListFormat] and [ListBuilder] on the same type.
    type Builder: ListBuilder<'p, P> + Sized;

    /// Constructs a [ListFormat::Builder], optionally using a reference to the [PolygonList] being triangulated.
    fn initialize(self, polygon_list: &'p P) -> Result<Self::Builder, <Self::Builder as ListBuilder<'p, P>>::Error>;

    /// Converts this [ListFormat] into a [FanFormat](crate::FanFormat), the format type required for triangulation.
    fn into_fan_format(self) -> formats::FanToListFormat<'p, P, Self>
    where Self: Sized {
        formats::FanToListFormat::new(self)
    }
}

/// Performs the construction of a triangle list
pub trait ListBuilder<'p, P: PolygonList<'p> + ?Sized> {
    /// The triangle list output type
    type Output;
    /// The error type when the builder fails
    type Error: error::Error;

    /// The winding direction this builder expects for triangles
    const WINDING: TriangleWinding = TriangleWinding::Counterclockwise;

    /// Adds a triangle with the given indices
    fn add_triangle(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error>;

    /// Called when triangulation has completed to get the resulting output
    fn build(self) -> Result<Self::Output, Self::Error>;

    /// Called when triangulation encounters an error.
    /// 
    /// Any required cleanup (e.g. removing the partial triangulation added to an existing [Vec]) should be done here
    fn fail(self, error: &TriangulationError<Self::Error>);
}
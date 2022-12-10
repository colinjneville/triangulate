/// A type which can be used to index a specific [Vertex](crate::Vertex).
/// Automatically implemented for all [Eq] + [Clone] types
#[cfg(not(feature = "_debugging"))]
pub trait VertexIndex: Eq + Clone { }
#[cfg(feature = "_debugging")]
pub trait VertexIndex: Eq + Clone + std::fmt::Debug { }

#[cfg(not(feature = "_debugging"))]
impl<T> VertexIndex for T 
where T: Eq + Clone
{ }

#[cfg(feature = "_debugging")]
impl<T> VertexIndex for T 
where T: Eq + Clone + std::fmt::Debug
{ }
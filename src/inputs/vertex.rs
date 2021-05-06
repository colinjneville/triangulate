use std::fmt::Debug;

use num_traits::real::Real;

use crate::idx::IdxDisplay;

#[cfg(not(feature = "debugging"))]
pub trait Vertex: Debug {
    type Coordinate: Real;

    fn x(&self) -> Self::Coordinate;
    fn y(&self) -> Self::Coordinate;
}

#[cfg(feature = "debugging")]
pub trait Vertex: Debug {
    type Coordinate: Real + Debug;
    
    fn x(&self) -> Self::Coordinate;
    fn y(&self) -> Self::Coordinate;
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub(crate) struct VertexExt<V: Vertex>(pub V);

impl<V: Vertex> VertexExt<V> {
    pub fn to_newtype_ref(base: &V) -> &VertexExt<V> {
        unsafe {
            &*(base as *const V as *const VertexExt<V>)
        }
    }
}

impl<V: Vertex> std::fmt::Display for VertexExt<V>
where V::Coordinate: std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

impl<V: Vertex> IdxDisplay for VertexExt<V> {
    fn fmt(f: &mut std::fmt::Formatter<'_>, idx: usize) -> std::fmt::Result {
        write!(f, "v{}", idx)
    }
}

impl<V: Vertex> VertexExt<V> {
    #[inline(always)]
    pub fn x(&self) -> V::Coordinate {
        self.0.x()
    }

    #[inline(always)]
    pub fn y(&self) -> V::Coordinate {
        self.0.y()
    }

    pub fn cross(&self, a: &Self, b: &Self) -> V::Coordinate {
        (a.x() - self.x()) * (b.y() - self.y()) - (a.y() - self.y()) * (b.x() - self.x())
    }
}

impl<V: Vertex> From<V> for VertexExt<V> {
    #[inline(always)]
    fn from(v: V) -> Self {
        Self(v)
    }
}

impl<V: Vertex> PartialOrd for VertexExt<V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.y().partial_cmp(&other.y()) {
            Some(y_ord) => {
                if y_ord == std::cmp::Ordering::Equal {
                    self.x().partial_cmp(&other.x())
                } else {
                    Some(y_ord)
                }
            }
            None => None,
        }
    }
}

impl<V: Vertex> PartialEq for VertexExt<V> {
    fn eq(&self, other: &Self) -> bool {
        self.x() == other.x() && self.y() == other.y()
    }
}

#[cfg(not(feature = "debugging"))]
pub trait VertexIndex: Eq + Clone { }
#[cfg(feature = "debugging")]
pub trait VertexIndex: Eq + Clone + Debug { }

#[cfg(not(feature = "debugging"))]
impl<T> VertexIndex for T 
where T: Eq + Clone
{ }

#[cfg(feature = "debugging")]
impl<T> VertexIndex for T 
where T: Eq + Clone + Debug
{ }
use core::fmt;
use std::{fmt::Debug, cmp};

use num_traits::real::Real;

use crate::idx::IdxDisplay;

/// A two-dimensional point. 
/// 
/// The coordinate type must implement [num_traits::real::Real], reexported as [crate::Real].
pub trait Vertex {
    /// The type of the individual `x` and `y` coordinates
    type Coordinate: Real;

    /// The x [Vertex::Coordinate] value
    fn x(&self) -> Self::Coordinate;
    /// The y [Vertex::Coordinate] value
    fn y(&self) -> Self::Coordinate;
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Coords<C: Real>([C; 2]);

impl<C: Real> Coords<C> {
    pub fn x(&self) -> C { self.0[0] }
    pub fn y(&self) -> C { self.0[1] }

    pub fn zero() -> Self { Self([C::zero(), C::zero()]) }
}

impl<C: Real> fmt::Debug for Coords<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tuple = f.debug_tuple("Coords");
        if let Some(x) = self.x().to_f64() {
            tuple.field(&x);
        }
        if let Some(y) = self.y().to_f64() {
            tuple.field(&y);
        }
        tuple.finish()
    }
}

impl<C: Real> fmt::Display for Coords<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let (Some(x), Some(y)) = (self.x().to_f64(), self.y().to_f64()) {
            write!(f, "({}, {})", x, y)
        } else {
            write!(f, "Coords<{}>", std::any::type_name::<C>())
        }
    }
}

impl<C: Real> PartialOrd for Coords<C> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.y().partial_cmp(&other.y()).and_then(|y_ord| 
            if y_ord == cmp::Ordering::Equal {
                self.x().partial_cmp(&other.x())
            } else {
                Some(y_ord)
            }
        )
    }
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

    pub fn coords(&self) -> Coords<V::Coordinate> {
        Coords([self.x(), self.y()])
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
}

impl<V: Vertex> From<V> for VertexExt<V> {
    #[inline(always)]
    fn from(v: V) -> Self {
        Self(v)
    }
}

impl<V: Vertex> PartialOrd for VertexExt<V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.y().partial_cmp(&other.y()).and_then(|y_ord| 
            if y_ord == cmp::Ordering::Equal {
                self.x().partial_cmp(&other.x())
            } else {
                Some(y_ord)
            }
        )
    }
}

impl<V: Vertex> PartialEq for VertexExt<V> {
    fn eq(&self, other: &Self) -> bool {
        self.x() == other.x() && self.y() == other.y()
    }
}

impl<C: Debug + Real> Vertex for [C; 2] {
    type Coordinate = C;

    #[inline(always)]
    fn x(&self) -> Self::Coordinate {
        self[0]
    }

    #[inline(always)]
    fn y(&self) -> Self::Coordinate {
        self[1]
    }
}

impl<C: Debug + Real> Vertex for (C, C) {
    type Coordinate = C;

    #[inline(always)]
    fn x(&self) -> Self::Coordinate {
        self.0
    }

    #[inline(always)]
    fn y(&self) -> Self::Coordinate {
        self.1
    }
}

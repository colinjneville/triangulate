use std::{error, fmt};

use backtrace::Backtrace;

/// Describes an error which occurred during trapezoidation
#[derive(Debug)]
#[non_exhaustive]
pub enum TrapezoidationError {
    /// A polygon was encountered with fewer than 3 vertices
    NotEnoughVertices(usize),
    /// A trapezoidation precondition was violated in the provided [PolygonList](crate::PolygonList), or a trapezoidation bug was encountered.
    InternalError(InternalError),
}

impl error::Error for TrapezoidationError { }

impl fmt::Display for TrapezoidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotEnoughVertices(vertices) => write!(f, "Polygon only contains {} vertices", vertices),
            Self::InternalError(error) => fmt::Display::fmt(error, f),
        }
    }
}

#[derive(Debug)]
pub struct InternalError {
    pub msg: String,
    pub backtrace: Backtrace,
}

impl InternalError {
    #[cold]
    #[inline(always)]
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            backtrace: Backtrace::new_unresolved(),
        }
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{:?}", self.msg, self.backtrace)
    }
}

impl error::Error for InternalError { }

/// Describes an error which occurred during triangulation
#[derive(Debug)]
#[non_exhaustive]
pub enum TriangulationError<FBError: error::Error> {
    /// An error occured during the trapezoidation step
    TrapezoidationError(TrapezoidationError),
    /// No vertices were included within the [PolygonList](crate::PolygonList)
    NoVertices,
    /// A triangulation precondition was violated in the provided [PolygonList](crate::PolygonList), 
    /// or a triangulation bug was encountered.
    InternalError(InternalError),
    /// The [FanBuilder](crate::FanBuilder) returned an error.
    FanBuilder(FBError),
    #[cfg(feature = "_debugging")]
    SvgOutput(std::io::Error),
}

impl<FBError: error::Error> TriangulationError<FBError> {
    #[inline(always)]
    pub(crate) fn internal(msg: impl Into<String>) -> Self {
        TriangulationError::InternalError(InternalError {
            msg: msg.into(),
            backtrace: Backtrace::new_unresolved(),
        })
    }
}

impl<FBError: error::Error> From<FBError> for TriangulationError<FBError> {
    fn from(e: FBError) -> Self {
        Self::FanBuilder(e)
    }
}

impl<FBError: error::Error> fmt::Display for TriangulationError<FBError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TrapezoidationError(error) => fmt::Display::fmt(error, f),
            Self::NoVertices => write!(f, "Polygon set contains no vertices"),
            Self::InternalError(error) => fmt::Display::fmt(error, f),
            Self::FanBuilder(error) => fmt::Display::fmt(error, f),
            #[cfg(feature = "_debugging")]
            Self::SvgOutput(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl<FBError: error::Error> std::error::Error for TriangulationError<FBError> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InternalError(error) => Some(error),
            Self::FanBuilder(error) => error.source(), // This should be Some(error), but that forces restricting FBError to 'static.
            #[cfg(feature = "_debugging")]
            Self::SvgOutput(error) => Some(error),
            _ => None,
        }
    }
}

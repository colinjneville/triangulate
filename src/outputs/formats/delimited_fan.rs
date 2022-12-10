use core::fmt;
use std::{error, marker::PhantomData};

use crate::{FanFormat, PolygonList, TriangulationError, outputs::fan_format::FanBuilder, Fans};

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum DelimitedFanError {
    IndexMatchesDelimiter,
}

impl fmt::Display for DelimitedFanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DelimitedFanError::IndexMatchesDelimiter => write!(f, "A vertex index equal to the delimiter value was encountered"),
        }
    }
}

impl error::Error for DelimitedFanError { }

/// A [FanFormat] which separates each fan with a specified delimiter value
#[derive(Debug)]
pub struct DelimitedFanFormat<'p, P: PolygonList<'p> + ?Sized, FS: Fans<Fan=P::Index>> {
    fans: FS,
    delimiter: P::Index,
    _phantom: PhantomData<&'p P>,
}

impl<'p, P: PolygonList<'p> + ?Sized, FS: Fans<Fan=P::Index>> DelimitedFanFormat<'p, P, FS> {
    /// Create a delimited format which stores its output in the given [Fans]
    pub fn new(fans: FS, delimiter: P::Index) -> Self {
        Self { fans, delimiter, _phantom: PhantomData, }
    }
}

impl <'p, P: PolygonList<'p> + ?Sized, FS: Fans<Fan=P::Index>> FanFormat<'p, P> for DelimitedFanFormat<'p, P, FS> {
    type Builder = DelimitedFanBuilder<'p, P, FS>;

    fn initialize(self, polygon_list: &'p P, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<Self::Builder, <Self::Builder as FanBuilder<'p, P>>::Error> {
        DelimitedFanBuilder::new(self.fans, self.delimiter, polygon_list, vi0, vi1, vi2)
    }
}

#[derive(Debug)]
pub struct DelimitedFanBuilder<'p, P: PolygonList<'p> + ?Sized, FS: Fans<Fan=P::Index>> {
    fans: FS,
    delimiter: P::Index,
    initial_vert_count: usize,
    _phantom: PhantomData<&'p P>,
}

impl<'p, P: PolygonList<'p> + ?Sized, FS: Fans<Fan=P::Index>> DelimitedFanBuilder<'p, P, FS> {
    fn new(mut fans: FS, delimiter: P::Index, _polygon_list: &'p P, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<Self, <Self as FanBuilder<'p, P>>::Error> {
        let initial_vert_count = fans.len();

        if fans.len() > 0 {
            fans.push(delimiter.clone());
        }
        
        let mut fb = Self {
            fans,
            delimiter,
            initial_vert_count,
            _phantom: PhantomData,
        };

        fb.push_index(vi0)?;
        fb.push_index(vi1)?;
        fb.push_index(vi2)?;

        Ok(fb)
    }

    fn push_index(&mut self, vi: P::Index) -> Result<(), <Self as FanBuilder<'p, P>>::Error> {
        if self.delimiter == vi {
            Err(DelimitedFanError::IndexMatchesDelimiter)
        } else {
            self.fans.push(vi);
            Ok(())
        }
    }
}

impl<'p, P: PolygonList<'p> + ?Sized, FS: Fans<Fan=P::Index>> FanBuilder<'p, P> for DelimitedFanBuilder<'p, P, FS> {
    type Output = FS;
    type Error = DelimitedFanError;

    fn new_fan(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error> {
        self.fans.push(self.delimiter.clone());
        
        self.push_index(vi0)?;
        self.push_index(vi1)?;
        self.push_index(vi2)?;

        Ok(())
    }

    fn extend_fan(&mut self, vi: P::Index) -> Result<(), Self::Error> {
        self.push_index(vi)?;
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        Ok(self.fans)
    }

    fn fail(mut self, _error: &TriangulationError<Self::Error>) {
        self.fans.truncate(self.initial_vert_count);
    }
}

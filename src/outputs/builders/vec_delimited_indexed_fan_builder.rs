use std::{error, fmt, marker::PhantomData};

use crate::{FanBuilder, PolygonList, TriangulationError};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VecDelimitedIndexedError {
    DelimiterIndexReached,
}

impl fmt::Display for VecDelimitedIndexedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Designated delimiter index is present in the polygon list")
    }
}

impl error::Error for VecDelimitedIndexedError {
    
}

pub struct VecDelimitedIndexedFanBuilder<'f, 'a: 'f, P: PolygonList<'a>> {
    list: &'f mut Vec<P::Index>,
    delimiter: P::Index,
    initial_vertex_count: usize,
    _a: PhantomData<&'a ()>
}

impl<'f, 'a: 'f, P: PolygonList<'a>> VecDelimitedIndexedFanBuilder<'f, 'a, P> {
    fn push(&mut self, index: P::Index) -> Result<(), <Self as FanBuilder<'a, P>>::Error> {
        if index == self.delimiter {
            Err(VecDelimitedIndexedError::DelimiterIndexReached)
        } else {
            self.list.push(index);
            Ok(())
        }
    }

    fn delimit(&mut self) {
        self.list.push(self.delimiter.clone());
    }
}

impl<'f, 'a: 'f, P: PolygonList<'a>> FanBuilder<'a, P> for VecDelimitedIndexedFanBuilder<'f, 'a, P> {
    type Initializer = (&'f mut Vec<P::Index>, P::Index);
    type Output = &'f mut [P::Index];
    type Error = VecDelimitedIndexedError;

    fn new(initializer: Self::Initializer, _polygon_list: P, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<Self, Self::Error>
    where Self: Sized {
        let (list, delimiter) = initializer;
        let initial_vertex_count = list.len();
        let mut builder = Self {
            list,
            delimiter,
            initial_vertex_count,
            _a: PhantomData,
        };
        // If we were given an existing Vec, delimit the existing indicies
        if let Some(last_index) = builder.list.last() {
            if last_index != &builder.delimiter {
                builder.delimit();
            }
        }
        builder.push(vi0)?;
        builder.push(vi1)?;
        builder.push(vi2)?;

        Ok(builder)
    }

    fn new_fan(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error> {
        self.delimit();
        self.push(vi0)?;
        self.push(vi1)?;
        self.push(vi2)?;
        Ok(())
    }

    fn extend_fan(&mut self, vi: P::Index) -> Result<(), Self::Error> {
        self.push(vi)
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        Ok(&mut self.list[self.initial_vertex_count..])
    }

    fn fail(self, _error: &TriangulationError<Self::Error>) {
        self.list.truncate(self.initial_vertex_count)
    }
}
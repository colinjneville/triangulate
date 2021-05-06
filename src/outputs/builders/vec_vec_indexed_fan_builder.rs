use std::convert;

use crate::{FanBuilder, PolygonList, TriangulationError, VertexIndex};

pub struct VecVecIndexedFanBuilder<'f, Index: VertexIndex> {
    current_fan: Vec<Index>,
    fans: &'f mut Vec<Vec<Index>>,
    initial_fan_count: usize,
}

impl<'f, 'a, P: PolygonList<'a>> FanBuilder<'a, P> for VecVecIndexedFanBuilder<'f, P::Index> {
    type Initializer = &'f mut Vec<Vec<P::Index>>;
    type Output = &'f mut [Vec<P::Index>];
    type Error = convert::Infallible;

    fn new(fans: Self::Initializer, _polygon_list: P, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<Self, Self::Error> {
        let initial_fan_count = fans.len();
        let mut fb = VecVecIndexedFanBuilder::<P::Index> {
            fans,
            current_fan: Vec::new(),
            initial_fan_count,
        };
        // Self implements FanBuilder for all P's, so we must be explicit here
        <Self as FanBuilder<'a, P>>::new_fan(&mut fb, vi0, vi1, vi2)?;
        Ok(fb)
    }

    fn new_fan(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error> {
        let prev_fan = std::mem::replace(&mut self.current_fan, vec![vi0, vi1, vi2]);
        self.fans.push(prev_fan);
        Ok(())
    }

    fn extend_fan(&mut self, vi: P::Index) -> Result<(), Self::Error> {
        self.current_fan.push(vi);
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        self.fans.push(self.current_fan);
        Ok(&mut self.fans[self.initial_fan_count..])
    }

    fn fail(self, _error: &TriangulationError<Self::Error>) {
        self.fans.truncate(self.initial_fan_count);
    }
}

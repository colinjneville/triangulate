use crate::{FanBuilder, PolygonList, TriangulationError};


pub struct VecVecFanBuilder<'f, 'a, P: PolygonList<'a>> 
where P::Vertex: Clone {
    polygon_list: P,
    fans: &'f mut Vec<Vec<P::Vertex>>,
    current_fan: Vec<P::Vertex>,
    initial_fan_count: usize,
}

impl<'f, 'a, P: PolygonList<'a>> FanBuilder<'a, P> for VecVecFanBuilder<'f, 'a, P> 
where P::Vertex: Clone {
    type Initializer = &'f mut Vec<Vec<P::Vertex>>;
    type Output = &'f mut [Vec<P::Vertex>];
    type Error = std::convert::Infallible;

    fn new(fans: Self::Initializer, polygon_list: P, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<Self, Self::Error> {
        let initial_fan_count = fans.len();
        let mut fb = Self {
            polygon_list,
            fans,
            current_fan: Vec::new(),
            initial_fan_count,
        };
        fb.new_fan(vi0, vi1, vi2)?;
        Ok(fb)
    }

    fn new_fan(&mut self, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<(), Self::Error> {
        let (v0, v1, v2) = (self.polygon_list.get_vertex(vi0).clone(), self.polygon_list.get_vertex(vi1).clone(), self.polygon_list.get_vertex(vi2).clone());
        let prev_fan = std::mem::replace(&mut self.current_fan, vec![v0, v1, v2]);
        if prev_fan.len() > 0 {
            self.fans.push(prev_fan);
        }
        Ok(())
    }

    fn extend_fan(&mut self, vi: P::Index) -> Result<(), Self::Error> {
        self.current_fan.push(self.polygon_list.get_vertex(vi).clone());
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        if self.current_fan.len() > 0 {
            self.fans.push(self.current_fan);
        }
        Ok(&mut self.fans[self.initial_fan_count..])
    }

    fn fail(self, _error: &TriangulationError<Self::Error>) {
        self.fans.truncate(self.initial_fan_count);
    }
}

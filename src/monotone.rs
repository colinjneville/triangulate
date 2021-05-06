use std::marker::PhantomData;

use smallvec::{SmallVec, smallvec};

use crate::{FanBuilder, FanBuilderState, PolygonList, PolygonListExt, TriangleWinding, VertexIndex, errors::{TriangulationError, TriangulationInternalError}, math::is_left_of_line};

pub(crate) struct MonotoneBuilder<Index: VertexIndex> {
    vec: SmallVec<[Index; 8]>,
    diff_x: bool,
    diff_y: bool,
}

impl<Index: VertexIndex> MonotoneBuilder<Index> {
    pub fn new(vi: Index) -> Self {
        Self {
            vec: smallvec![vi],
            diff_x: false,
            diff_y: false,
        }
    }

    pub fn add_vertex<'a, P: PolygonList<'a, Index=Index>>(&mut self, ps: PolygonListExt<'a, P>, vi: Index) {
        if !self.diff_x {
            if ps[vi.clone()].x() != ps[self.vec[0].clone()].x() {
                self.diff_x = true;
            }
        }
        if !self.diff_y {
            if ps[vi.clone()].y() != ps[self.vec[0].clone()].y() {
                self.diff_y = true;
            }
        }

        self.vec.push(vi);
    }

    pub fn build<'a, P: PolygonList<'a, Index=Index>>(self, ps: PolygonListExt<'a, P>) -> Result<Option<Monotone<Index>>, TriangulationInternalError> {
        if self.vec.len() < 3 {
            return Err(TriangulationInternalError::new(format!("Monotone needs at least 3 vertices, has {}", self.vec.len())));
        }

        if self.diff_x && self.diff_y {
            let is_left_chain = is_left_of_line(&ps[self.vec[self.vec.len() - 1].clone()], &ps[self.vec[0].clone()], &ps[self.vec[1].clone()]);
            Ok(Some(Monotone::new(self.vec, is_left_chain)))
        } else {
            Ok(None)
        }
    }
}

pub struct Monotone<Index> {
    // Skipped stack from [0, skipped_top), pending stack from [pending_top, len), expended values remain in [skipped_top, pending_top)
    pub(crate) skipped_and_pending: SmallVec<[Index; 8]>,
    skipped_top: usize,
    pending_top: usize,
    // Is the chain on the left of the polygon (and the single edge on the right)?
    is_left_chain: bool,
}

#[cfg(feature = "debugging")]
impl<Index: VertexIndex> std::fmt::Display for Monotone<Index> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ")?;
        for i in 0..self.skipped_top {
            write!(f, "{:03?} ", self.skipped_and_pending[i])?;
        }
        write!(f, "] ")?;
        for i in self.skipped_top..self.pending_top {
            write!(f, "{:03?} ", self.skipped_and_pending[i])?;
        }
        write!(f, "[ ")?;
        for i in self.pending_top..self.skipped_and_pending.len() {
            write!(f, "{:03?} ", self.skipped_and_pending[i])?;
        }
        write!(f, "]")
    }
}

impl<Index: VertexIndex> Monotone<Index> {
    fn new(vertices: SmallVec<[Index; 8]>, is_left_chain: bool) -> Self {
        Self {
            skipped_and_pending: vertices,
            skipped_top: 2,
            pending_top: 2,
            is_left_chain,
        }
    }

    pub(crate) fn build_fans<'z, 'a, P: PolygonList<'a, Index=Index>, FB: FanBuilder<'a, P>>(mut self, ps: PolygonListExt<'a, P>, fbs: &'z mut FanBuilderState<'a, P, FB>) -> Result<(), TriangulationError<FB::Error>> {
        enum BuilderOrDeferredTris<'z, 'a, P: PolygonList<'a>, FB: FanBuilder<'a, P>> {
            Builder(&'z mut FB),
            DeferredTris(&'z mut FanBuilderState<'a, P, FB>, usize, PhantomData<&'a ()>),
        }
        impl<'z, 'a, P: PolygonList<'a>, FB: FanBuilder<'a, P>> BuilderOrDeferredTris<'z, 'a, P, FB> {
            fn add_triangle(&mut self, vi: P::Index) -> Result<(), FB::Error> {
                match self {
                    BuilderOrDeferredTris::Builder(fb) => fb.extend_fan(vi)?,
                    BuilderOrDeferredTris::DeferredTris(_, dt, _) => *dt += 1,
                };
                Ok(())
            }
        }

        while self.pending_top < self.skipped_and_pending.len() {
            if self.can_triangulate(ps) {
                let vi1 = self.skipped_pop();
                let mut vi0 = self.skipped_peek();
                let mut vi2 = self.pending_peek();

                // Advancing fan/backtracking fan and left chain/right chain both invert the winding.
                // If we need to de-invert the winding, defer add_triangle calls until we have processed the fan 
                // and can make the calls in a reversed order
                let can_triangulate_next = self.can_triangulate(ps);
                let mut bodt: BuilderOrDeferredTris<'_, '_, P, FB> = if can_triangulate_next ^ self.is_left_chain ^ (FB::WINDING == TriangleWinding::Clockwise) {
                    if !self.is_left_chain {
                        std::mem::swap(&mut vi0, &mut vi2);
                    }
                    BuilderOrDeferredTris::Builder(fbs.new_fan(ps.0, vi0, vi1, vi2)?)
                } else {
                    BuilderOrDeferredTris::DeferredTris(fbs, 1, PhantomData)
                };

                // If the next triangle comes from backtracking the stack, the active vertex is the fan root, 
                // otherwise the penultimate stack vertex is the root
                if can_triangulate_next {
                    // We already confirmed we can extend at least one more triangle
                    self.skipped_pop();
                    bodt.add_triangle(self.skipped_peek()).map_err(TriangulationError::from)?;

                    // Then continue adding triangles as much as possible
                    while self.can_triangulate(ps) {
                        self.skipped_pop();
                        bodt.add_triangle(self.skipped_peek()).map_err(TriangulationError::from)?;
                    }
                } else {
                    self.transfer_pending();
                    while self.can_triangulate(ps) {
                        self.skipped_pop();
                        bodt.add_triangle(self.pending_peek()).map_err(TriangulationError::from)?;
                        self.transfer_pending();
                    }
                }

                
                if let BuilderOrDeferredTris::DeferredTris(fbs, dt, _) = bodt {
                    let (vi0, vi1, vi2) = if can_triangulate_next {
                        (self.pending_peek(), self.skipped_peek(), self.deferred_index(0))
                    } else {
                        let (vi0, vi1) = self.skipped_peek2();
                        (vi0, vi1, self.deferred_index(dt - 1))
                    };
                    let fb = fbs.new_fan(ps.0, vi0, vi1, vi2)?;
                    for i in 1..dt {
                        let index = if can_triangulate_next {
                            i
                        } else {
                            dt - i - 1
                        };
                        if let Err(e) = fb.extend_fan(self.deferred_index(index)) {
                            return Err(e.into());
                        }
                    }
                }
            }
            if self.has_pending() {
                self.transfer_pending();
            }
        }
        
        Ok(())
    }

    fn skipped_peek(&self) -> Index {
        self.skipped_and_pending[self.skipped_top - 1].clone()
    }

    fn skipped_peek2(&self) -> (Index, Index) {
        (self.skipped_and_pending[self.skipped_top - 2].clone(), self.skipped_and_pending[self.skipped_top - 1].clone())
    }

    fn skipped_pop(&mut self) -> Index {
        self.skipped_top -= 1;
        self.skipped_and_pending[self.skipped_top].clone()
    }

    fn pending_peek(&self) -> Index {
        self.skipped_and_pending[self.pending_top].clone()
    }

    fn transfer_pending(&mut self) {
        if self.skipped_top != self.pending_top {
            self.skipped_and_pending.swap(self.skipped_top, self.pending_top);
        }
        self.skipped_top += 1;
        self.pending_top += 1;
    }

    fn has_pending(&self) -> bool { self.pending_top < self.skipped_and_pending.len() }

    fn deferred_index(&self, i: usize) -> Index {
        self.skipped_and_pending[self.skipped_top + i].clone()
    }

    fn can_triangulate<'a, P: PolygonList<'a, Index=Index>>(&self, ps: PolygonListExt<'a, P>) -> bool {
        self.skipped_top >= 2 && self.has_pending() && {
            let v_min = &ps[self.pending_peek()];
            let (v_max, v) = { let (vi_max, vi) = self.skipped_peek2(); (&ps[vi_max], &ps[vi]) };
            
            self.is_left_chain == is_left_of_line(v_min, v_max, v)
        }
    }
}

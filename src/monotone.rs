use std::marker::PhantomData;

use num_traits::real::Real;
use smallvec::{SmallVec, smallvec};

use crate::{FanFormat, FanBuilderState, PolygonList, PolygonListExt, TriangleWinding, VertexIndex, errors::{TriangulationError, InternalError}, math::is_left_of_line, FanBuilder, Coords};

pub(crate) struct MonotoneBuilder<Index: VertexIndex, C: Real> {
    vec: SmallVec<[(Index, Coords<C>); 16]>,
    diff_x: bool,
    diff_y: bool,
}

impl<Index: VertexIndex, C: Real> MonotoneBuilder<Index, C> {
    pub fn new(vi: Index, c: Coords<C>) -> Self {
        Self {
            vec: smallvec![(vi, c)],
            diff_x: false,
            diff_y: false,
        }
    }

    pub fn add_vertex(&mut self, vi: Index, c: Coords<C>) {
        if !self.diff_x && c.x() != self.vec[0].1.x() {
            self.diff_x = true;
        }
        if !self.diff_y && c.y() != self.vec[0].1.y() {
            self.diff_y = true;
        }

        self.vec.push((vi, c));
    }

    pub fn build(self) -> Result<Option<Monotone<Index, C>>, InternalError> {
        if self.vec.len() < 3 {
            return Err(InternalError::new(format!("Monotone needs at least 3 vertices, has {}", self.vec.len())));
        }

        if self.diff_x && self.diff_y {
            let is_left_chain = is_left_of_line(self.vec[self.vec.len() - 1].1, self.vec[0].1, self.vec[1].1);
            Ok(Some(Monotone::new(self.vec, is_left_chain)))
        } else {
            Ok(None)
        }
    }
}

pub struct Monotone<Index: VertexIndex, C: Real> {
    // Skipped stack from [0, skipped_top), pending stack from [pending_top, len), expended/deferred values remain in [skipped_top, pending_top)
    pub(crate) skipped_and_pending: SmallVec<[(Index, Coords<C>); 16]>,
    skipped_top: usize,
    pending_top: usize,
    // Is the chain on the left of the polygon (and the single edge on the right)?
    is_left_chain: bool,
}

#[cfg(feature = "_debugging")]
impl<Index: VertexIndex, C: Real> std::fmt::Display for Monotone<Index, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "is_left_chain: {}", self.is_left_chain)?;
        write!(f, "[ ")?;
        for i in 0..self.skipped_top {
            write!(f, "{:03?} ", self.skipped_and_pending[i].0)?;
        }
        write!(f, "] ")?;
        for i in self.skipped_top..self.pending_top {
            write!(f, "{:03?} ", self.skipped_and_pending[i].0)?;
        }
        write!(f, "[ ")?;
        for i in self.pending_top..self.skipped_and_pending.len() {
            write!(f, "{:03?} ", self.skipped_and_pending[i].0)?;
        }
        write!(f, "]")
    }
}

impl<Index: VertexIndex, C: Real> Monotone<Index, C> {
    fn new(vertices: SmallVec<[(Index, Coords<C>); 16]>, is_left_chain: bool) -> Self {
        Self {
            skipped_and_pending: vertices,
            skipped_top: 2,
            pending_top: 2,
            is_left_chain,
        }
    }

    pub(crate) fn build_fans<'z, 'p, P: PolygonList<'p, Index=Index> + ?Sized, FB: FanFormat<'p, P>>(mut self, ps: PolygonListExt<'p, P>, fbs: &'z mut FanBuilderState<'p, P, FB>) -> Result<(), TriangulationError<<FB::Builder as FanBuilder<'p, P>>::Error>> {
        enum BuilderOrDeferredTris<'z, 'p, P: PolygonList<'p> + ?Sized, FB: FanFormat<'p, P>> {
            Builder(&'z mut FB::Builder),
            DeferredTris(&'z mut FanBuilderState<'p, P, FB>, usize, PhantomData<&'p ()>),
        }
        impl<'z, 'p, P: PolygonList<'p> + ?Sized, FB: FanFormat<'p, P>> BuilderOrDeferredTris<'z, 'p, P, FB> {
            fn add_triangle(&mut self, vi: P::Index) -> Result<(), <FB::Builder as FanBuilder<'p, P>>::Error> {
                match self {
                    BuilderOrDeferredTris::Builder(fb) => fb.extend_fan(vi)?,
                    BuilderOrDeferredTris::DeferredTris(_, dt, _) => *dt += 1,
                };
                Ok(())
            }
        }

        while self.remaining_vertices() >= 3 {
            if self.can_triangulate() {
                // The base triangle, with all 3 points specified
                let vi1 = self.skipped_pop().0;
                let mut vi0 = self.skipped_peek().0;
                let mut vi2 = self.pending_peek().0;

                // Advancing fan/backtracking fan and left chain/right chain both invert the winding.
                // If we need to de-invert the winding, defer add_triangle calls until we have processed the fan 
                // and can make the calls in a reversed order
                let is_backtracking = self.can_triangulate();
                let mut bodt: BuilderOrDeferredTris<'_, '_, P, FB> = if is_backtracking ^ self.is_left_chain ^ (FB::Builder::WINDING == TriangleWinding::Clockwise) {
                    if !self.is_left_chain {
                        std::mem::swap(&mut vi0, &mut vi2);
                    }
                    BuilderOrDeferredTris::Builder(fbs.new_fan(ps.polygon_list(), vi0, vi1, vi2)?)
                } else {
                    BuilderOrDeferredTris::DeferredTris(fbs, 1, PhantomData)
                };

                // If the next triangle comes from backtracking the stack, the active vertex is the fan root, 
                // otherwise the penultimate stack vertex is the root
                if is_backtracking {
                    // We already confirmed we can extend at least one more triangle
                    self.skipped_pop();
                    bodt.add_triangle(self.skipped_peek().0).map_err(TriangulationError::from)?;

                    // Then continue adding triangles as much as possible
                    while self.can_triangulate() {
                        self.skipped_pop();
                        bodt.add_triangle(self.skipped_peek().0).map_err(TriangulationError::from)?;
                    }
                } else {
                    self.transfer_pending();
                    while self.can_triangulate() {
                        self.skipped_pop();
                        bodt.add_triangle(self.pending_peek().0).map_err(TriangulationError::from)?;

                        self.transfer_pending();
                    }
                }

                // If we had to defer extend_fan calls, execute them now in reverse order
                if let BuilderOrDeferredTris::DeferredTris(fbs, dt, _) = bodt {
                    let (vi0, vi1, vi2) = if is_backtracking {
                        (self.pending_peek().0, self.skipped_peek().0, self.deferred_index(is_backtracking, 0))
                    } else {
                        let ((vi0,  _), (vi1, _)) = self.skipped_peek2();
                        (vi0, vi1, self.deferred_index(is_backtracking, 0))
                    };
                    let fb = fbs.new_fan(ps.polygon_list(), vi0, vi1, vi2)?;
                    for i in 1..dt {
                        if let Err(e) = fb.extend_fan(self.deferred_index(is_backtracking, i)) {
                            return Err(e.into());
                        }
                    }
                }
            }

            if self.has_pending() {
                self.transfer_pending();
            } else {
                // In rare cases, pushing vertices and popping whenever backtracking is possible will not be able to triangulate everything.
                // If we have pushed all vertices and we can't backtrack, reset our position to the initial setup (minus all removed vertices)
                self.reset_position();
            }
        }

        let remaining_vertices = self.skipped_and_pending.len() - self.pending_top + self.skipped_top;
        if remaining_vertices == 2 {
            Ok(())
        } else {
            Err(TriangulationError::internal(format!("Expected 2 remaining vertices, found {remaining_vertices}")))
        }
    }

    #[inline(always)]
    fn skipped_peek(&self) -> (Index, Coords<C>) {
        self.skipped_and_pending[self.skipped_top - 1].clone()
    }

    #[inline(always)]
    fn skipped_peek2(&self) -> ((Index, Coords<C>), (Index, Coords<C>)) {
        (self.skipped_and_pending[self.skipped_top - 2].clone(), self.skipped_and_pending[self.skipped_top - 1].clone())
    }

    #[inline(always)]
    fn skipped_pop(&mut self) -> (Index, Coords<C>) {
        self.skipped_top -= 1;
        self.skipped_and_pending[self.skipped_top].clone()
    }

    #[inline(always)]
    fn pending_peek(&self) -> (Index, Coords<C>) {
        self.skipped_and_pending[self.pending_top].clone()
    }

    fn remaining_vertices(&self) -> usize {
        self.skipped_and_pending.len() - self.pending_top + self.skipped_top
    }

    fn transfer_pending(&mut self) {
        if self.skipped_top != self.pending_top {
            self.skipped_and_pending.swap(self.skipped_top, self.pending_top);
        }
        self.skipped_top += 1;
        self.pending_top += 1;
    }

    fn reset_position(&mut self) {
        self.skipped_and_pending.truncate(self.skipped_top);
        self.skipped_top = 2;
        self.pending_top = 2;
    }

    fn has_pending(&self) -> bool { self.pending_top < self.skipped_and_pending.len() }

    fn deferred_index(&self, is_backtracking: bool, i: usize) -> Index {
        let index = if is_backtracking {
            self.skipped_top + i
        } else {
            self.pending_top - 1 - i
        };
        self.skipped_and_pending[index].0.clone()
    }

    fn can_triangulate(&self) -> bool {
        self.skipped_top >= 2 && self.has_pending() && {
            let c_min = self.pending_peek().1;
            let ((_, c_max), (_, c)) = self.skipped_peek2();
            
            self.is_left_chain == is_left_of_line(c_min, c_max, c)
        }
    }
}

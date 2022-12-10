use std::{convert::TryInto, marker::PhantomData, ops};

use crate::{FanFormat, TriangulationError, VertexExt, VertexIndex, trapezoidation::{Trapezoidation, TrapezoidationState}, errors::TrapezoidationError, FanBuilder};

use super::vertex::Vertex;

/// Used to destinguish multiple polygons while iterating with 
/// [PolygonList::iter_indices].
#[derive(Debug)]
pub enum PolygonElement<Index: VertexIndex> {
    /// Provides a [VertexIndex] value that the [PolygonList] can convert to a [Vertex] 
    /// reference.
    ContinuePolygon(Index),
    /// Indicates that the next [ContinuePolygon](PolygonElement::ContinuePolygon) index belongs to a new polygon.
    /// It is not necessary to begin or end iteration with [NewPolygon](PolygonElement::NewPolygon).
    NewPolygon,
}

impl<Index: VertexIndex> From<Index> for PolygonElement<Index> {
    fn from(index: Index) -> Self {
        PolygonElement::ContinuePolygon(index)
    }
}

impl<Index: VertexIndex> From<Option<Index>> for PolygonElement<Index> {
    fn from(index: Option<Index>) -> Self {
        match index {
            Some(index) => PolygonElement::ContinuePolygon(index),
            None => PolygonElement::NewPolygon,
        }
    }
}

/// Used to treat a single [Polygon] as a [PolygonList]. Created by
/// [Polygon::as_polygon_list].
#[repr(transparent)]
pub struct SinglePolygon<'p, P: Polygon<'p>>(P, PhantomData<&'p ()>);

impl<'p, P: Polygon<'p>> SinglePolygon<'p, P> {
    fn new(polygon: P) -> Self {
        Self(polygon, PhantomData)
    }
}

impl<'p, P: Polygon<'p>> PolygonList<'p> for SinglePolygon<'p, P> {
    type Vertex = P::Vertex;
    type Index = P::Index;
    type IntoItem = P::Index;
    type Iter<'i> = P::Iter<'i>
    where Self: 'i, Self::Vertex: 'i, 'p: 'i;

    fn vertex_count(&self) -> usize {
        self.0.vertex_count()
    }

    fn iter_indices<'i>(&'i self) -> Self::Iter<'i>
    where Self: 'i, Self::Vertex: 'i, 'p: 'i {
        self.0.iter_indices()
    }

    fn get_vertex<'a>(&'a self, index: Self::Index) -> &'a Self::Vertex
    where 'p: 'a {
        self.0.get_vertex(index)
    }
}

/// An indexable polygon's vertices
pub trait Polygon<'p>: 'p + Sized {
    /// The type of vertices of the polygon
    type Vertex: Vertex + 'p;
    /// A type used to uniquely identify a [Vertex] (e.g. [usize] for a [Vec<\[f32, f32\]>](Vec))
    type Index: VertexIndex + 'p;
    /// The [Iterator] type that [Polygon::iter_indices] returns
    type Iter<'i>: Iterator<Item=Self::Index>
    where Self: 'i, Self::Vertex: 'i, 'p: 'i;

    /// Provides the number of vertices of the polygon.
    fn vertex_count(&self) -> usize;

    /// Iterate through all [Polygon::Index]es of all polygons.
    /// Indices must be returned in either clockwise or counter-clockwise order, 
    /// without repeating the initial index. Between each polygon, implementers
    /// must return a [PolygonElement::NewPolygon] value.
    ///
    /// [PolygonElement::ContinuePolygon] values must be yielded in successive groups of at least
    /// 3, otherwise triangulation will fail.
    ///
    /// Successive [PolygonElement::NewPolygon]s are idempotent, and [PolygonElement::NewPolygon]`s as the initial 
    /// or final iteration values are not required and have no effect.
    fn iter_indices<'i>(&'i self) -> Self::Iter<'i>
    where Self: 'i, Self::Vertex: 'i, 'p: 'i;

    /// Get the [Polygon::Vertex] uniquely identified by the [Polygon::Index] value
    fn get_vertex(&self, index: Self::Index) -> &Self::Vertex;

    /// Treat this [Polygon] as a [PolygonList] containing a single polygon
    fn as_polygon_list(&self) -> &SinglePolygon<'p, Self> {
        unsafe {
            &*(self as *const Self as *const SinglePolygon<'p, Self>)
        }
    }

    /// Create a [PolygonList] with the [Polygon::Index] type substituted with another.
    /// 
    /// The old index type must be convertable via [TryInto] to the new index type, and vice versa, 
    /// but will panic if the conversion fails
    fn index_with<New: TryInto<Self::Index>>(self) -> IndexWith<'p, SinglePolygon<'p, Self>, Self::Index, Self::Index, New> 
    where Self::Index: VertexIndex + crate::Mappable<Self::Index> + TryInto<New>,
          <Self::Index as crate::Mappable<Self::Index>>::Output<New>: VertexIndex + crate::Mappable<New, Output<Self::Index> = Self::Index> {
        SinglePolygon::new(self).index_with()
    }

    /// Generate a [Trapezoidation], which can later be triangulated. 
    /// 
    /// Unless the [Trapezoidation] is needed for other reasons, this can be done in a single step with [Polygon::triangulate].
    fn trapezoidize(&'p self) -> Result<Trapezoidation<'p, SinglePolygon<'p, Self>>, TrapezoidationError> {
        self.as_polygon_list().trapezoidize()
    }

    /// Triangulate the polygon into the layout specified by `format`
    fn triangulate<FB: FanFormat<'p, SinglePolygon<'p, Self>>>(&'p self, format: FB) -> Result<<FB::Builder as FanBuilder<'p, SinglePolygon<'p, Self>>>::Output, TriangulationError<<FB::Builder as FanBuilder<'p, SinglePolygon<'p, Self>>>::Error>> {
        self.as_polygon_list().triangulate(format)
    }
}

/// An indexable list of polygons and their vertices
pub trait PolygonList<'p>: 'p {
    /// The type of vertices of the polygons
    type Vertex: Vertex + 'p;
    /// A type used to uniquely identify a [Vertex] (e.g. `[usize; 2]` for a [Vec<Vec<\[f32, f32\]>>](Vec))
    type Index: VertexIndex + 'p;
    /// The [PolygonList::Index] [Iterator] type
    type IntoItem: Into<PolygonElement<Self::Index>>;
    /// The [Iterator] type that [PolygonList::iter_indices] returns
    type Iter<'i>: Iterator<Item=Self::IntoItem>
    where Self: 'i, Self::Vertex: 'i, 'p: 'i;

    /// Provides the total number of vertices among all polygons.
    fn vertex_count(&self) -> usize;

    /// Iterate through all `Index`es of all polygons.
    /// Indices must be returned in either clockwise or counter-clockwise order, 
    /// without repeating the initial index. Between each polygon, implementers
    /// must return a `NewPolygon` value.
    ///
    /// `ContinuePolygon` values must be yielded in successive groups of at least
    /// 3, otherwise triangulation will fail.
    ///
    /// Successive `NewPolygon`s are idempotent, and `NewPolygon`s as the initial 
    /// or final iteration values have no effect.
    fn iter_indices<'i>(&'i self) -> Self::Iter<'i>
    where Self: 'i, Self::Vertex: 'i, 'p: 'i;

    /// Get the [PolygonList::Vertex] uniquely identified by `index`
    fn get_vertex<'a>(&'a self, index: Self::Index) -> &'a Self::Vertex
    where 'p: 'a;

    /// Substitute the [PolygonList::Index] type with another.
    ///
    /// The old index type must be convertable via [TryInto] to the new index type, and vice versa, 
    /// but will panic if the conversion fails
    fn index_with<Old: TryInto<New>, New: TryInto<Old>>(self) -> IndexWith<'p, Self, Self::Index, Old, New> 
    where Self: Sized,
          Self::Index: VertexIndex + crate::Mappable<Old>,
          <Self::Index as crate::Mappable<Old>>::Output<New>: VertexIndex + crate::Mappable<New, Output<Old> = Self::Index> {
        IndexWith::new(self)
    }
    
    /// Generate a [Trapezoidation], which can later be triangulated. 
    /// 
    /// Unless the [Trapezoidation] is needed for other reasons, this can be done in a single step with [PolygonList::triangulate].
    fn trapezoidize(&'p self) -> Result<Trapezoidation<'p, Self>, TrapezoidationError> {
        TrapezoidationState::new(self).build()
    }

    /// Triangulate the polygons into the layout specified by `format`
    fn triangulate<FB: FanFormat<'p, Self>>(&'p self, format: FB) -> Result<<FB::Builder as FanBuilder<'p, Self>>::Output, TriangulationError<<FB::Builder as FanBuilder<'p, Self>>::Error>> {
        self.trapezoidize().map_err(TriangulationError::TrapezoidationError)?.triangulate(format)
    }
}

// Allows indexing to directly return `VertexExt`s internally for convenience to add display and math functionality 
#[derive(Debug)]
pub(crate) struct PolygonListExt<'p, P: PolygonList<'p> + ?Sized>(&'p P);

impl<'p, P: PolygonList<'p> + ?Sized> Clone for PolygonListExt<'p, P> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<'p, P: PolygonList<'p> + ?Sized> Copy for PolygonListExt<'p, P> { }

impl<'p, P: PolygonList<'p> + ?Sized> PolygonListExt<'p, P> {
    pub fn new(p: &'p P) -> Self {
        Self(p)
    }

    pub fn iter_polygon_vertices(&self) -> P::Iter<'_> {
        self.0.iter_indices()
    }

    pub fn vertex_count(&self) -> usize {
        self.0.vertex_count()
    }

    pub fn polygon_list(&self) -> &'p P {
        self.0
    }
}

impl<'p, P: PolygonList<'p> + ?Sized> ops::Index<P::Index> for PolygonListExt<'p, P> {
    type Output = VertexExt<P::Vertex>;

    fn index(&self, index: P::Index) -> &Self::Output {
        VertexExt::to_newtype_ref(self.0.get_vertex(index))
    }
}

/// [Iterator] for a [PolygonList] represented as nested vectors
pub struct VecVecIter<'a, 'p: 'a, V: Vertex, P: Polygon<'p, Index=usize, Vertex=V>> {
    parent: &'a [P],
    outer_index: usize,
    inner_index: usize,
    _phantom: PhantomData<&'p ()>,
}

impl<'a, 'p: 'a, V: Vertex, P: Polygon<'p, Index=usize, Vertex=V>> VecVecIter<'a, 'p, V, P> {
    fn new(parent: &'a [P]) -> Self {
        Self {
            parent,
            outer_index: 0,
            inner_index: 0,
            _phantom: PhantomData,
        }
    }
}

impl<'a, 'p: 'a, V: Vertex, P: Polygon<'p, Index=usize, Vertex=V>> Iterator for VecVecIter<'a, 'p, V, P> {
    type Item = PolygonElement<[usize; 2]>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.outer_index < self.parent.len() {
            Some(if self.inner_index < (self.parent[self.outer_index]).vertex_count() {
                let result = [self.outer_index, self.inner_index];
                self.inner_index += 1;
                PolygonElement::ContinuePolygon(result)
            } else {
                self.inner_index = 0;
                self.outer_index += 1;
                PolygonElement::NewPolygon
            })
        } else {
            None
        }
    }
}

impl<'p, V: 'p + Vertex, T: 'p + ops::Deref<Target=[V]>> Polygon<'p> for T {
    type Vertex = V;
    type Index = usize;
    type Iter<'i> = ops::Range<usize>
    where Self: 'i, Self::Vertex: 'i, 'p: 'i;

    fn vertex_count(&self) -> usize {
        (*self).len()
    }

    fn iter_indices<'i>(&'i self) -> Self::Iter<'i> 
    where Self: 'i, Self::Vertex: 'i, 'p: 'i {
        0..self.vertex_count()
    }

    fn get_vertex(&self, index: Self::Index) -> &Self::Vertex {
        &(*self)[index]
    }
}

impl<'p, V: Vertex + 'p, P: 'p + Polygon<'p, Vertex=V, Index=usize>, D: 'p + ops::Deref<Target=[P]>> PolygonList<'p> for D {
    type Vertex = V;
    type Index = [usize; 2];
    type IntoItem = PolygonElement<Self::Index>;
    type Iter<'i> = VecVecIter<'i, 'p, V, P>
    where Self: 'i, Self::Vertex: 'i, 'p: 'i;

    fn vertex_count(&self) -> usize {
        (*self).iter().map(|p| (*p).vertex_count()).sum()
    }

    fn iter_indices<'i>(&'i self) -> Self::Iter<'i>
    where Self: 'i, Self::Vertex: 'i, 'p: 'i {
        VecVecIter::new(self)
    }

    fn get_vertex<'a>(&'a self, index: Self::Index) -> &'a Self::Vertex
    where 'p: 'a {
        let [i0, i1] = index;
        self[i0].get_vertex(i1)
    }
}

fn conversion_panic<T, U>(_: T) -> U {
    panic!("Conversion of index failed")
}

/// [Iterator] for the [IndexWith] wrapper
pub struct IndexWithIter<'i, Iter: Iterator + 'i, OldIndex: VertexIndex + crate::Mappable<Old>, New: TryInto<Old>, Old: TryInto<New>> 
where Iter::Item: Into<PolygonElement<OldIndex>>,
      OldIndex::Output<New>: VertexIndex + crate::Mappable<New, Output<Old> = OldIndex> {
    iter: Iter,
    _phantom: PhantomData<&'i (OldIndex, New, Old)>,
}

impl<'i, Iter: Iterator + 'i, OldIndex: VertexIndex + crate::Mappable<Old>, New: TryInto<Old>, Old: TryInto<New>> IndexWithIter<'i, Iter, OldIndex, New, Old>
where Iter::Item: Into<PolygonElement<OldIndex>>,
      OldIndex::Output<New>: VertexIndex + crate::Mappable<New, Output<Old> = OldIndex> {
    pub(crate) fn new(iter: Iter) -> Self {
        Self { iter, _phantom: PhantomData }
    }
}

impl<'i, Iter: Iterator + 'i, OldIndex: VertexIndex + crate::Mappable<New>, In: TryInto<New>, New: TryInto<In>> Iterator for IndexWithIter<'i, Iter, OldIndex, In, New> 
where Iter::Item: Into<PolygonElement<OldIndex>>,
      OldIndex::Output<In>: VertexIndex + crate::Mappable<In, Output<New> = OldIndex> {
    type Item = PolygonElement<OldIndex::Output<In>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(into_polygon_vertex) => {
                let polygon_vertex: PolygonElement<OldIndex> = into_polygon_vertex.into();
                Some(match polygon_vertex {
                    PolygonElement::ContinuePolygon(index) => PolygonElement::ContinuePolygon(index.map(|i| i.try_into().unwrap_or_else(conversion_panic))),
                    PolygonElement::NewPolygon => PolygonElement::NewPolygon,
                })
            }
            None => None,
        }
    }
}

/// Wrapper to change the [PolygonList::Index] type
#[derive(Debug, Clone, Copy)]
pub struct IndexWith<'p, P: PolygonList<'p, Index=OldIndex>, OldIndex: VertexIndex + crate::Mappable<Old>, Old: TryInto<New>, New: TryInto<Old>>(P, PhantomData<&'p (OldIndex, Old, New)>)
where OldIndex::Output<New>: VertexIndex + crate::Mappable<New, Output<Old> = OldIndex>;

impl<'p, P: PolygonList<'p, Index=OldIndex>, OldIndex: VertexIndex + crate::Mappable<Old>, Old: TryInto<New>, New: TryInto<Old>> IndexWith<'p, P, OldIndex, Old, New> 
where OldIndex::Output<New>: VertexIndex + crate::Mappable<New, Output<Old> = OldIndex> {
    fn new(polygon_list: P) -> Self {
        Self(polygon_list, PhantomData)
    }
}

impl<'p, P: PolygonList<'p, Index=OldIndex>, OldIndex: VertexIndex + crate::Mappable<Old>, New: TryInto<Old>, Old: TryInto<New>> PolygonList<'p> for IndexWith<'p, P, OldIndex, Old, New> 
where OldIndex::Output<New>: VertexIndex + crate::Mappable<New, Output<Old> = OldIndex> {
    type Vertex = P::Vertex;
    type Index = OldIndex::Output<New>;
    type IntoItem = PolygonElement<Self::Index>;
    type Iter<'i> = IndexWithIter<'i, P::Iter<'i>, OldIndex, New, Old>
    where Self: 'i, 'p: 'i;

    fn vertex_count(&self) -> usize {
        self.0.vertex_count()
    }

    fn iter_indices<'i>(&'i self) -> Self::Iter<'i>
    where Self: 'i, Self::Vertex: 'i, 'p: 'i {
        IndexWithIter::new(self.0.iter_indices())
    }

    fn get_vertex<'a>(&'a self, index: Self::Index) -> &'a Self::Vertex
    where 'p: 'a {
        self.0.get_vertex(crate::Mappable::map(index, |t| t.try_into().unwrap_or_else(conversion_panic)))
    }
}

mod private {
    pub trait Sealed { }

    impl<'p, P: super::PolygonList<'p>> Sealed for P { }
}
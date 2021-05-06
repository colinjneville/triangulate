use std::{convert::TryInto, marker::PhantomData, ops};

use crate::{FanBuilder, TriangulationError, VertexExt, VertexIndex, do_triangulate};

use super::vertex::Vertex;

/// `PolygonVertex` destinguishes multiple polygons while iterating with 
/// `PolygonList::iter_polygon_vertices()`.
#[derive(Debug)]
pub enum PolygonVertex<Index: VertexIndex> {
    /// Provides an `Index` value that the `PolygonList` can convert to a `Vertex` 
    /// reference.
    ContinuePolygon(Index),
    /// Indicates that the next `ContinuePolygon` index belongs to a new polygon.
    NewPolygon,
}

impl<Index: VertexIndex> From<Index> for PolygonVertex<Index> {
    fn from(index: Index) -> Self {
        PolygonVertex::ContinuePolygon(index)
    }
}

impl<Index: VertexIndex> From<Option<Index>> for PolygonVertex<Index> {
    fn from(index: Option<Index>) -> Self {
        match index {
            Some(index) => PolygonVertex::ContinuePolygon(index),
            None => PolygonVertex::NewPolygon,
        }
    }
}

/// An indexable list of polygons and their vertices
pub trait PolygonList<'a>: Copy {
    /// The type of vertices of the polygons.
    type Vertex: 'a + Vertex;
    /// A type used to uniquely identify a `Vertex` (such as `(usize, usize)` could be a `Vertex` for a `Vec<Vec<MyVertex>>`)
    type Index: 'a + VertexIndex;
    /// The contained type 
    type IntoItem: Into<PolygonVertex<Self::Index>>;
    /// The `Iterator` type that `iter_polygon_vertices` returns.
    type Iter: Iterator<Item=Self::IntoItem>;

    /// Provides the total number of vertices among all polygons.
    fn vertex_count(self) -> usize;

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
    fn iter_polygon_vertices(self) -> Self::Iter;

    /// Get the `Vertex` uniquely identified by the `Index` value
    fn get_vertex(self, index: Self::Index) -> &'a Self::Vertex;
}

/// `Triangulate` provides the `triangulate()` function to implementers of `PolygonList`.
/// 
/// This trait is sealed and is not intended to be manually implemented.
pub trait Triangulate<'a, P: PolygonList<'a>>: private::Sealed {
    /// Triangulates this `PolygonList` using the specified `FanBuilder`.
    /// 
    /// The polygons from the `PolygonList` are broken down into 
    /// [triangle fans](https://en.wikipedia.org/wiki/Triangle_fan) which are
    /// fed to the `FanBuilder` through its callback functions.
    fn triangulate<FB: FanBuilder<'a, P>>(self, initializer: FB::Initializer) -> Result<FB::Output, TriangulationError<FB::Error>>;
}

impl<'a, P: PolygonList<'a>> Triangulate<'a, P> for P {
    #[inline]
    fn triangulate<FB: FanBuilder<'a, P>>(self, initializer: FB::Initializer) -> Result<FB::Output, TriangulationError<FB::Error>> {
        do_triangulate::<_, FB>(PolygonListExt(self, PhantomData), initializer)
    }
}

pub trait TriangulateDefault<'a, P: PolygonList<'a>> {
    fn triangulate_default<FB: FanBuilder<'a, P>>(self) -> Result<FB::Output, TriangulationError<FB::Error>>
    where FB::Initializer: Default;
}

impl<'a, P: PolygonList<'a>> TriangulateDefault<'a, P> for P {
    #[inline]
    fn triangulate_default<FB: FanBuilder<'a, P>>(self) -> Result<FB::Output, TriangulationError<FB::Error>>
    where FB::Initializer: Default {
        do_triangulate::<_, FB>(PolygonListExt(self, PhantomData), Default::default())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PolygonListExt<'a, P: PolygonList<'a>>(pub P, PhantomData<&'a ()>);

impl<'a, P: PolygonList<'a>> PolygonListExt<'a, P> {
    pub fn iter_polygon_vertices(&self) -> P::Iter {
        self.0.iter_polygon_vertices()
    }

    pub fn vertex_count(&self) -> usize {
        self.0.vertex_count()
    }
}

impl<'a, P: PolygonList<'a>> ops::Index<P::Index> for PolygonListExt<'a, P> {
    type Output = VertexExt<P::Vertex>;

    fn index(&self, index: P::Index) -> &'a Self::Output {
        VertexExt::to_newtype_ref(self.0.get_vertex(index))
    }
}

pub struct VecVecIter<'a, V: Vertex> {
    parent: &'a Vec<Vec<V>>,
    outer_index: usize,
    inner_index: usize,
}

impl<'a, V: Vertex> VecVecIter<'a, V> {
    fn new(parent: &'a Vec<Vec<V>>) -> Self {
        Self {
            parent,
            outer_index: 0,
            inner_index: 0,
        }
    }
}

impl<'a, V: Vertex> Iterator for VecVecIter<'a, V> {
    type Item = PolygonVertex<(usize, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.outer_index < self.parent.len() {
            Some(if self.inner_index < self.parent[self.outer_index].len() {
                let result = (self.outer_index, self.inner_index);
                self.inner_index += 1;
                PolygonVertex::ContinuePolygon(result)
            } else {
                self.inner_index = 0;
                self.outer_index += 1;
                PolygonVertex::NewPolygon
            })
        } else {
            None
        }
    }
}

impl<'a, V: Vertex> PolygonList<'a> for &'a Vec<Vec<V>> {
    type Vertex = V;
    type Index = (usize, usize);
    type IntoItem = PolygonVertex<Self::Index>;
    type Iter = VecVecIter<'a, V>;

    fn vertex_count(self) -> usize {
        self.iter().map(|p| p.len()).sum()
    }

    fn iter_polygon_vertices(self) -> Self::Iter {
        VecVecIter::new(self)
    }

    fn get_vertex(self, index: Self::Index) -> &'a Self::Vertex {
        let (p, v) = index;
        &self[p][v]
    }
}

impl<'a, V: Vertex> PolygonList<'a> for &'a Vec<V> {
    type Vertex = V;
    type Index = usize;
    type IntoItem = Self::Index;
    type Iter = ops::Range<Self::Index>;

    fn vertex_count(self) -> usize {
        self.len()
    }

    fn iter_polygon_vertices(self) -> Self::Iter {
        0..self.len()
    }

    fn get_vertex(self, index: Self::Index) -> &'a Self::Vertex {
        &self[index]
    }
}

impl<'a, V: Vertex> PolygonList<'a> for &'a [V] {
    type Vertex = V;
    type Index = usize;
    type IntoItem = Self::Index;
    type Iter = ops::Range<Self::Index>;

    fn vertex_count(self) -> usize {
        self.len()
    }

    fn iter_polygon_vertices(self) -> Self::Iter {
        0..self.len()
    }

    fn get_vertex(self, index: Self::Index) -> &'a Self::Vertex {
        &self[index]
    }
}

pub struct IndexWithIter<'a, In, Out, Index: VertexIndex, P: PolygonList<'a, Index=Index>> {
    iter: P::Iter,
    _in: PhantomData<In>,
    _out: PhantomData<Out>,
}

impl<'a, In: VertexIndex + TryInto<Out, Error=IntoError>, Out: VertexIndex, IntoError, P: PolygonList<'a, Index=In>> Iterator for IndexWithIter<'a, In, Out, In, P> {
    type Item = PolygonVertex<Out>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(into_polygon_vertex) => {
                let polygon_vertex: PolygonVertex<In> = into_polygon_vertex.into();
                Some(match polygon_vertex {
                    PolygonVertex::ContinuePolygon(index) => PolygonVertex::ContinuePolygon(index.try_into().unwrap_or_else(|_| panic!("Conversion of index failed"))),
                    PolygonVertex::NewPolygon => PolygonVertex::NewPolygon,
                })
            }
            None => None,
        }
    }
}

impl<'a, In: VertexIndex + TryInto<Out, Error=IntoError>, Out: VertexIndex, IntoError, P: PolygonList<'a, Index=(In, In)>> Iterator for IndexWithIter<'a, In, Out, (In, In), P> {
    type Item = PolygonVertex<(Out, Out)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(into_polygon_vertex) => {
                let polygon_vertex: PolygonVertex<(In, In)> = into_polygon_vertex.into();
                Some(match polygon_vertex {
                    PolygonVertex::ContinuePolygon((index0, index1)) => {
                        let index0 = index0.try_into().unwrap_or_else(|_| panic!("Conversion of index failed"));
                        let index1 = index1.try_into().unwrap_or_else(|_| panic!("Conversion of index failed"));
                        PolygonVertex::ContinuePolygon((index0, index1))
                    }
                    PolygonVertex::NewPolygon => PolygonVertex::NewPolygon,
                })
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IndexWithU16<'a, P: PolygonList<'a, Index=usize>>(P, PhantomData<&'a ()>);

impl<'a, P: PolygonList<'a, Index=usize>> IndexWithU16<'a, P> {
    pub fn new(polygon_list: P) -> Self {
        Self(polygon_list, PhantomData)
    }
}

impl<'a, P: PolygonList<'a, Index=usize>> PolygonList<'a> for IndexWithU16<'a, P> {
    type Vertex = P::Vertex;
    type Index = u16;
    type IntoItem = PolygonVertex<Self::Index>;
    type Iter = IndexWithIter<'a, usize, u16, usize, P>;

    fn vertex_count(self) -> usize {
        self.0.vertex_count()
    }

    fn iter_polygon_vertices(self) -> Self::Iter {
        IndexWithIter { iter: self.0.iter_polygon_vertices() , _in: PhantomData, _out: PhantomData }
    }

    fn get_vertex(self, index: Self::Index) -> &'a Self::Vertex {
        self.0.get_vertex(index as usize)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IndexWithU16U16<'a, P: PolygonList<'a, Index=(usize, usize)>>(P, PhantomData<&'a ()>);

impl<'a, P: PolygonList<'a, Index=(usize, usize)>> IndexWithU16U16<'a, P> {
    pub fn new(polygon_list: P) -> Self {
        Self(polygon_list, PhantomData)
    }
}

impl<'a, P: PolygonList<'a, Index=(usize, usize)>> PolygonList<'a> for IndexWithU16U16<'a, P> {
    type Vertex = P::Vertex;
    type Index = (u16, u16);
    type IntoItem = PolygonVertex<Self::Index>;
    type Iter = IndexWithIter<'a, usize, u16, (usize, usize), P>;

    fn vertex_count(self) -> usize {
        self.0.vertex_count()
    }

    fn iter_polygon_vertices(self) -> Self::Iter {
        IndexWithIter { iter: self.0.iter_polygon_vertices(), _in: PhantomData, _out: PhantomData }
    }

    fn get_vertex(self, index: Self::Index) -> &'a Self::Vertex {
        self.0.get_vertex((index.0 as usize, index.1 as usize))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IndexWithU32<'a, P: PolygonList<'a, Index=usize>>(P, PhantomData<&'a ()>);

impl<'a, P: PolygonList<'a, Index=usize>> IndexWithU32<'a, P> {
    pub fn new(polygon_list: P) -> Self {
        Self(polygon_list, PhantomData)
    }
}

impl<'a, P: PolygonList<'a, Index=usize>> PolygonList<'a> for IndexWithU32<'a, P> {
    type Vertex = P::Vertex;
    type Index = u32;
    type IntoItem = PolygonVertex<Self::Index>;
    type Iter = IndexWithIter<'a, usize, u32, usize, P>;

    fn vertex_count(self) -> usize {
        self.0.vertex_count()
    }

    fn iter_polygon_vertices(self) -> Self::Iter {
        IndexWithIter { iter: self.0.iter_polygon_vertices() , _in: PhantomData, _out: PhantomData }
    }

    fn get_vertex(self, index: Self::Index) -> &'a Self::Vertex {
        self.0.get_vertex(index as usize)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IndexWithU32U32<'a, P: PolygonList<'a, Index=(usize, usize)>>(P, PhantomData<&'a ()>);

impl<'a, P: PolygonList<'a, Index=(usize, usize)>> IndexWithU32U32<'a, P> {
    pub fn new(polygon_list: P) -> Self {
        Self(polygon_list, PhantomData)
    }
}

impl<'a, P: PolygonList<'a, Index=(usize, usize)>> PolygonList<'a> for IndexWithU32U32<'a, P> {
    type Vertex = P::Vertex;
    type Index = (u32, u32);
    type IntoItem = PolygonVertex<Self::Index>;
    type Iter = IndexWithIter<'a, usize, u32, (usize, usize), P>;

    fn vertex_count(self) -> usize {
        self.0.vertex_count()
    }

    fn iter_polygon_vertices(self) -> Self::Iter {
        IndexWithIter { iter: self.0.iter_polygon_vertices(), _in: PhantomData, _out: PhantomData }
    }

    fn get_vertex(self, index: Self::Index) -> &'a Self::Vertex {
        self.0.get_vertex((index.0 as usize, index.1 as usize))
    }
}

mod private {
    pub trait Sealed { }

    impl<'a, P: super::PolygonList<'a>> Sealed for P { }
}
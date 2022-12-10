use core::fmt;
use std::clone;

use num_traits::real::Real;

use crate::{Vertex, VertexIndex, idx::{Idx, IdxDisplay}, trapezoid::Trapezoid, Coords};

#[derive(Debug)]
pub(crate) enum QueryNode<V: Vertex, Index: VertexIndex> {
    Branch(Idx<QueryNode<V, Index>>, Idx<QueryNode<V, Index>>, QueryNodeBranch<V::Coordinate>),
    Sink(Idx<Trapezoid<V, Index>>),
}

impl<V: Vertex, Index: VertexIndex> clone::Clone for QueryNode<V, Index> {
    fn clone(&self) -> Self {
        match self {
            Self::Branch(a, b, c) => Self::Branch(*a, *b, c.clone()),
            Self::Sink(a) => Self::Sink(*a),
        }
    }
}

impl<V: Vertex, Index: VertexIndex> fmt::Display for QueryNode<V, Index> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Branch(_, _, branch) => write!(f, "{}", branch),
            Self::Sink(ti) => write!(f, "S({})", ti),
        }
    }
}

pub(crate) enum QueryNodeBranch<C: Real> {
    X(Coords<C>, Coords<C>),
    Y(Coords<C>),
}

impl<C: Real> fmt::Debug for QueryNodeBranch<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X(c_min_x, c_max_x) => f.debug_tuple("X").field(c_min_x).field(c_max_x).finish(),
            Self::Y(c_y) => f.debug_tuple("Y").field(c_y).finish(),
        }
    }
}

impl<C: Real> clone::Clone for QueryNodeBranch<C> {
    fn clone(&self) -> Self {
        match self {
            Self::X(c_min, c_max) => Self::X(*c_min, *c_max),
            Self::Y(c) => Self::Y(*c),
        }
    }
}

impl<C: Real> fmt::Display for QueryNodeBranch<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X(c_min_x, c_max_x) => write!(f, "X({}, {})", c_min_x, c_max_x),
            Self::Y(c_y) => write!(f, "Y({})", c_y),
        }
    }
}

pub struct IndexedQueryNode<'a, V: Vertex, Index: VertexIndex>(Idx<QueryNode<V, Index>>, &'a QueryNode<V, Index>);

impl<V: Vertex, Index: VertexIndex> QueryNode<V, Index> {
    #[cfg(feature = "_debugging")]
    pub fn as_text_tree<'a>(&'a self, qi: Idx<Self>, qs: &'a [Self]) -> text_trees::TreeNode<IndexedQueryNode<'a, V, Index>> {
        let node = IndexedQueryNode(qi, self.into());
        match self {
            QueryNode::Branch(left, right, _) => text_trees::TreeNode::with_child_nodes(node, vec![qs[*left].as_text_tree(*left, qs), qs[*right].as_text_tree(*right, qs)].into_iter()),
            QueryNode::Sink(_) => node.into(),
        }
    }
}

impl<'a, V: Vertex, Index: VertexIndex> std::fmt::Display for IndexedQueryNode<'a, V, Index> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.0, self.1)
    }
}

impl<V: Vertex, Index: VertexIndex> IdxDisplay for QueryNode<V, Index> {
    fn fmt(f: &mut std::fmt::Formatter<'_>, idx: usize) -> std::fmt::Result {
        write!(f, "q{}", idx)
    }
}

impl<V: Vertex, Index: VertexIndex> QueryNode<V, Index> {
    pub fn root(ti: Idx<Trapezoid<V, Index>>) -> Self {
        Self::Sink(ti)
    }

    #[must_use]
    pub fn branch_x(&mut self, qi_left: Idx<Self>, qi_right: Idx<Self>, c_min_x: Coords<V::Coordinate>, c_max_x: Coords<V::Coordinate>, ti_right: Idx<Trapezoid<V, Index>>) -> (Self, Self) {
        (self.branch(qi_left, qi_right, QueryNodeBranch::X(c_min_x, c_max_x)), QueryNode::Sink(ti_right))
    }

    #[must_use]
    pub fn merge_x(&mut self, qi_left: Idx<Self>, qi_right: Idx<Self>, c_min_x: Coords<V::Coordinate>, c_max_x: Coords<V::Coordinate>) -> Self {
        self.branch(qi_left, qi_right, QueryNodeBranch::X(c_min_x, c_max_x))
    }

    #[must_use]
    pub fn branch_y(&mut self, qi_left: Idx<Self>, qi_right: Idx<Self>, c_y: Coords<V::Coordinate>, ti_up: Idx<Trapezoid<V, Index>>) -> (Self, Self) {
        (self.branch(qi_left, qi_right, QueryNodeBranch::Y(c_y)), QueryNode::Sink(ti_up))
    }

    #[must_use]
    fn branch(&mut self, qi_left: Idx<Self>, qi_right: Idx<Self>, branch: QueryNodeBranch<V::Coordinate>) -> Self {
        let mut new = QueryNode::Branch(qi_left, qi_right, branch);
        std::mem::swap(self, &mut new);
        new
    }
}

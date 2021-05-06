use crate::{Vertex, VertexIndex, idx::{Idx, IdxDisplay}, nexus::Nexus, segment::Segment, trapezoid::Trapezoid};

#[derive(Debug)]
pub(crate) enum QueryNode<V: Vertex, Index: VertexIndex> {
    Branch(Idx<QueryNode<V, Index>>, Idx<QueryNode<V, Index>>, QueryNodeBranch<V, Index>),
    Sink(Idx<Trapezoid<V, Index>>),
}

// https://github.com/rust-lang/rust/issues/26925
impl<V: Vertex, Index: VertexIndex> std::clone::Clone for QueryNode<V, Index> {
    fn clone(&self) -> Self {
        match self {
            Self::Branch(a, b, c) => Self::Branch(*a, *b, c.clone()),
            Self::Sink(a) => Self::Sink(*a),
        }
    }
}

impl<V: Vertex, Index: VertexIndex> std::fmt::Display for QueryNode<V, Index> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Branch(_, _, branch) => write!(f, "{}", branch),
            Self::Sink(ti) => write!(f, "S({})", ti),
        }
    }
}

#[derive(Debug)]
pub(crate) enum QueryNodeBranch<V: Vertex, Index: VertexIndex> {
    X(Idx<Segment<V, Index>>),
    Y(Idx<Nexus<V, Index>>),
}

// https://github.com/rust-lang/rust/issues/26925
impl<V: Vertex, Index: VertexIndex> std::clone::Clone for QueryNodeBranch<V, Index> {
    fn clone(&self) -> Self {
        match self {
            Self::X(a) => Self::X(*a),
            Self::Y(a) => Self::Y(*a),
        }
    }
}

impl<V: Vertex, Index: VertexIndex> std::fmt::Display for QueryNodeBranch<V, Index> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X(si_x) => write!(f, "X({})", si_x),
            Self::Y(ni_y) => write!(f, "Y({})", ni_y),
        }
    }
}

pub struct IndexedQueryNode<'a, V: Vertex, Index: VertexIndex>(Idx<QueryNode<V, Index>>, &'a QueryNode<V, Index>);

impl<V: Vertex, Index: VertexIndex> QueryNode<V, Index> {
    #[cfg(feature = "debugging")]
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
    pub fn into_x(&mut self, qi_left: Idx<Self>, qi_right: Idx<Self>, si_x: Idx<Segment<V, Index>>, ti_right: Idx<Trapezoid<V, Index>>) -> (Self, Self) {
        (self.into_branch(qi_left, qi_right, QueryNodeBranch::X(si_x)), QueryNode::Sink(ti_right))
    }

    #[must_use]
    pub fn into_x_merge(&mut self, qi_left: Idx<Self>, qi_right: Idx<Self>, si_x: Idx<Segment<V, Index>>) -> Self {
        self.into_branch(qi_left, qi_right, QueryNodeBranch::X(si_x))
    }

    #[must_use]
    pub fn into_y(&mut self, qi_left: Idx<Self>, qi_right: Idx<Self>, ni_y: Idx<Nexus<V, Index>>, ti_up: Idx<Trapezoid<V, Index>>) -> (Self, Self) {
        (self.into_branch(qi_left, qi_right, QueryNodeBranch::Y(ni_y)), QueryNode::Sink(ti_up))
    }

    #[must_use]
    fn into_branch(&mut self, qi_left: Idx<Self>, qi_right: Idx<Self>, branch: QueryNodeBranch<V, Index>) -> Self {
        let mut new = QueryNode::Branch(qi_left, qi_right, branch);
        std::mem::swap(self, &mut new);
        new
    }
}

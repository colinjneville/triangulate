/// A list of triangles represented as triplets of vertices of type `V`
pub trait List<V> {
    /// Add a new triangle to the list
    fn push(&mut self, v0: V, v1: V, v2: V);

    /// The number of triangles in the list
    fn len(&self) -> usize;

    /// Remove newly added triangles until there are only `len` remaining
    fn truncate(&mut self, len: usize);

    /// Returns `true` if the collection contains no triangles
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<V> List<V> for Vec<V> {
    fn push(&mut self, v0: V, v1: V, v2: V) {
        self.push(v0);
        self.push(v1);
        self.push(v2);
    }

    fn len(&self) -> usize {
        self.len() / 3
    }

    fn truncate(&mut self, len: usize) {
        self.truncate(len * 3)
    }
}

impl<V> List<V> for Vec<[V; 3]> {
    fn push(&mut self, v0: V, v1: V, v2: V) {
        self.push([v0, v1, v2]);
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}

impl<V> List<V> for Vec<(V, V, V)> {
    fn push(&mut self, v0: V, v1: V, v2: V) {
        self.push((v0, v1, v2));
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}

impl<V, L: List<V>> List<V> for &mut L {
    fn push(&mut self, v0: V, v1: V, v2: V) {
        (**self).push(v0, v1, v2)
    }

    fn len(&self) -> usize {
        (**self).len()
    }

    fn truncate(&mut self, len: usize) {
        (**self).truncate(len)
    }
}

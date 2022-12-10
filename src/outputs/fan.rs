/// A triangle fan with vertices of type `V`
pub trait Fan<V> {
    /// Initialize the fan with a single triangle.
    fn new(v0: V, v1: V, v2: V) -> Self;

    /// Add another triangle to the fan, specifying only the newly added vertex `v`
    fn push(&mut self, v: V);
}

impl<V> Fan<V> for Vec<V> {
    fn new(v0: V, v1: V, v2: V) -> Self {
        vec![v0, v1, v2]
    }

    fn push(&mut self, v: V) {
        self.push(v)
    }
}

/// A collection of multiple [Fan]s.
pub trait Fans {
    /// The type of the individual [Fan]s
    type Fan;

    /// The number of [Fan]s
    fn len(&self) -> usize;

    /// Remove newly added [Fan]s until there are only `len` remaining
    fn truncate(&mut self, len: usize);

    /// Add a new [Fan]
    fn push(&mut self, fan: Self::Fan);

    /// Returns `true` if the collection contains no [Fan]s
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<F> Fans for Vec<F> {
    type Fan = F;

    fn len(&self) -> usize {
        self.len()
    }

    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }

    fn push(&mut self, fan: Self::Fan) {
        self.push(fan)
    }
}

impl<FS: Fans> Fans for &mut FS {
    type Fan = FS::Fan;

    fn len(&self) -> usize {
        (**self).len()
    }

    fn truncate(&mut self, len: usize) {
        (**self).truncate(len)
    }

    fn push(&mut self, fan: Self::Fan) {
        (**self).push(fan)
    }
}

use std::marker::PhantomData;

use crate::List;

pub(crate) struct GenericList<L: List<V>, V> {
    list: L,
    initial_vert_count: usize,
    _phantom: PhantomData<V>,
}

impl<L: List<V>, V> GenericList<L, V> {
    pub fn new(list: L) -> Self {
        let initial_vert_count = list.len();
        Self {
            list,
            initial_vert_count,
            _phantom: PhantomData,
        }
    }

    pub fn new_triangle(&mut self, v0: V, v1: V, v2: V) {
        self.list.push(v0, v1, v2);
    }

    pub fn build(self) -> L {
        self.list
    }

    pub fn fail(mut self) {
        self.list.truncate(self.initial_vert_count);
    }
}

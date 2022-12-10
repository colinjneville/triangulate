use std::marker::PhantomData;

use crate::{Fans, Fan};

pub(crate) struct GenericFans<FS: Fans, V>
where FS::Fan: Fan<V> {
    fans: FS,
    current_fan: FS::Fan,
    initial_fan_count: usize,
    _phantom: PhantomData<V>,
}

impl<FS: Fans, V> GenericFans<FS, V> 
where FS::Fan: Fan<V> {
    pub fn new(fans: FS, v0: V, v1: V, v2: V) -> Self {
        let initial_fan_count = fans.len();
        let current_fan = FS::Fan::new(v0, v1, v2);
        Self {
            fans,
            current_fan,
            initial_fan_count,
            _phantom: PhantomData,
        }
    }

    pub fn new_fan(&mut self, v0: V, v1: V, v2: V) {
        let prev_fan = std::mem::replace(&mut self.current_fan, FS::Fan::new(v0, v1, v2));
        self.fans.push(prev_fan);
    }

    pub fn extend_fan(&mut self, v: V) {
        self.current_fan.push(v);
    }

    pub fn build(mut self) -> FS {
        self.fans.push(self.current_fan);
        self.fans
    }

    pub fn fail(mut self) {
        self.fans.truncate(self.initial_fan_count);
    }
}

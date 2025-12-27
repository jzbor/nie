use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};


#[derive(Default)]
pub struct Registry<K: Eq + std::hash::Hash, V: Clone>(LazyLock<RwLock<HashMap<K, V>>>);

impl<K: Eq + std::hash::Hash, V: Clone> Registry<K, V> {
    pub const fn new() -> Self {
        Registry(LazyLock::new(RwLock::default))
    }

    pub fn lookup(&self, key: &K) -> Option<V> {
        self.0.read().unwrap().get(key).cloned()
    }

    pub fn store(&self, key: K, value: V) {
        self.0.write().unwrap().insert(key, value);
    }
}


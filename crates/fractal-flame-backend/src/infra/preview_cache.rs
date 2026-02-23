use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Default)]
pub struct PreviewCache {
    cache: RwLock<HashMap<String, Vec<u8>>>,
}

impl PreviewCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.cache.read().ok()?.get(key).cloned()
    }

    pub fn insert(&self, key: String, value: Vec<u8>) {
        if let Ok(mut guard) = self.cache.write() {
            guard.insert(key, value);
        }
    }

    pub fn key(variation_id: &str, symmetry: usize, gamma: f64) -> String {
        format!("{}:{}:{}", variation_id, symmetry, gamma)
    }
}

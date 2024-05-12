use crate::resp::RespFrame;
use dashmap::{DashMap, DashSet};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Backend(Arc<BackendInner>);

#[derive(Clone, Debug)]
pub struct BackendInner {
    map: DashMap<String, RespFrame>,
    hmap: DashMap<String, DashMap<String, RespFrame>>,
    set: DashMap<String, DashSet<String>>,
}

impl Deref for Backend {
    type Target = BackendInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self(Arc::new(BackendInner::default()))
    }
}

impl Default for BackendInner {
    fn default() -> Self {
        Self {
            map: DashMap::new(),
            hmap: DashMap::new(),
            set: DashMap::new(),
        }
    }
}

impl Backend {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&self, key: &str) -> Option<RespFrame> {
        self.map.get(key).map(|v| v.value().clone())
    }

    pub fn set(&self, key: String, value: RespFrame) {
        self.map.insert(key, value);
    }

    pub fn hget(&self, key: &str, field: &str) -> Option<RespFrame> {
        self.hmap
            .get(key)
            .and_then(|v| v.get(field).map(|v| v.value().clone()))
    }

    pub fn hset(&self, key: String, field: String, value: RespFrame) {
        let hmap = self.hmap.entry(key).or_default();
        hmap.insert(field, value);
    }

    pub fn hgetall(&self, key: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap.get(key).map(|v| v.clone())
    }

    pub fn sismember(&self, key: &str, value: &str) -> bool {
        self.set
            .get(key)
            .and_then(|v| v.get(value).map(|_| true))
            .unwrap_or(false)
    }
    pub fn insert_set(&self, key: String, values: Vec<String>) {
        let set = self.set.get_mut(&key);
        match set {
            Some(set) => {
                for value in values {
                    (*set).insert(value);
                }
            }
            None => {
                let new_set = DashSet::new();
                for value in values {
                    new_set.insert(value);
                }
                self.set.insert(key.to_string(), new_set);
            }
        }
    }
}

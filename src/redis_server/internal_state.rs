use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct RedisInternalState {
    key_value_store: HashMap<String, String>,
}

impl RedisInternalState {
    pub fn new() -> Self {
        Self {
            key_value_store: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.key_value_store.get(key)
    }

    pub fn set(&mut self, key: String, value: String) {
        self.key_value_store.insert(key, value);
    }
}

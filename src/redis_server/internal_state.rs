use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::bail;

#[derive(Debug, Default)]
pub struct RedisInternalState {
    key_value_store: HashMap<String, (Option<SystemTime>, String)>,
}

impl RedisInternalState {
    pub fn new() -> Self {
        Self {
            key_value_store: HashMap::new(),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&String> {
        if let Some((expiry, _value)) = self.key_value_store.get(key) {
            // if the value is old, chuck it
            if expiry.is_some_and(|e| e < SystemTime::now()) {
                self.key_value_store.remove(key);
            }
        }
        // map new expiry onto the value
        self.key_value_store.get(key).map(|(_expiry, value)| value)
    }

    pub fn set(&mut self, key: String, value: String, expiry: Option<i64>) -> anyhow::Result<()> {
        if expiry.is_some_and(|e| e < 0) {
            bail!("ERR invalid expire time in set, {:?}", expiry);
        }

        let expiry = expiry.map(|e| SystemTime::now() + std::time::Duration::from_millis(e as u64));
        self.key_value_store.insert(key, (expiry, value));
        Ok(())
    }
}

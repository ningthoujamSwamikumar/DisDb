use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use clap::Subcommand;

pub mod connection;

pub struct KVStore {
    store: HashMap<String, String>,
}

impl KVStore {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    /// Returns `Some(value)` if present else `None`
    pub fn get(&self, key: impl AsRef<str>) -> Option<&String> {
        self.store.get(key.as_ref())
    }

    /// Inserts a key value pair
    /// Returns `None` if key isn't already present
    /// Returns the Old value if already present, after updating with the new value
    pub fn set(&mut self, key: String, value: String) -> Option<String> {
        self.store.insert(key, value)
    }

    /// Clears all the key-value pairs, and keep the allocated memories for reuse
    pub fn clear(&mut self) -> bool {
        self.store.clear();
        true
    }
}

#[derive(Debug, Subcommand, Serialize, Deserialize)]
pub enum KvsCommand {
    Set { key: String, value: String },
    Get { key: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum KvsResult {
    Get(Option<String>),
    Set(Option<String>)
}

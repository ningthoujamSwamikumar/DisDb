use std::collections::HashMap;

use clap::Subcommand;
use serde::{Deserialize, Serialize};

pub mod client;
pub mod connection;
pub mod error;
pub mod server;

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
    pub fn clear(&mut self) {
        self.store.clear();
    }

    pub fn count(&self) -> usize {
        self.store.len()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Frame {
    Command(KvsCommand),
    Result(KvsResult),
}

#[derive(Debug, Subcommand, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum KvsCommand {
    Set { key: String, value: String },
    Get { key: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum KvsResult {
    Get(Option<String>),
    Set(Option<String>),
    Error(String),
}

impl From<KvsCommand> for Frame {
    fn from(value: KvsCommand) -> Self {
        Frame::Command(value)
    }
}

impl From<KvsResult> for Frame {
    fn from(value: KvsResult) -> Self {
        Frame::Result(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::KVStore;

    #[test]
    fn test_db_funcs() {
        let mut db = KVStore::new();

        assert_eq!(db.count(), 0_usize, "Initially db should be empty!");
        assert!(
            db.get("something").is_none(),
            "Should be None if the key doesn't exist!"
        );
        assert!(
            db.set("key1".into(), "val1".into()).is_none(),
            "First time set should return None!"
        );
        assert_eq!(
            db.get("key1"),
            Some(&"val1".to_string()),
            "Unexpected value!"
        );
        assert_eq!(
            db.set("key1".into(), "val1_1".into()),
            Some("val1".into()),
            "Update existing entry should return old value!"
        );

        db.set("key2".into(), "val2".into());
        db.set("key3".into(), "val3".into());
        assert_eq!(db.count(), 3_usize, "Unexpected db entry count!");

        db.clear();
        assert_eq!(db.count(), 0_usize, "Expected db to be empty after clear!");
    }
}

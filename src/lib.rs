use std::collections::HashMap;

use clap::Subcommand;
use serde::{Deserialize, Serialize};

pub mod connection;
pub mod error;

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

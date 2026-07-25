mod cli;

use std::collections::HashMap;

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
    pub fn get(&self, key: impl AsRef<String>) -> Option<&String> {
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

pub enum KvsCommands<'a> {
    GET(&'a str),
    SET(String, String),
    CLEAR,
    NoOp
}

/*
impl <'a> From<&'a str> for Commands<'a> {
    fn from(value: &str) -> Self {
        let cmd = value.trim();
        let tokens = cmd.split_whitespace();
        match tokens.next() {
            Some(first) => {
                match first.to_lowercase() {
                    "get" => {

                    }
                    "set" => {

                    },
                    "clear" => Commands::CLEAR
                }
            },
            None => {
                Commands::NoOp
            },
        }
    }
} */

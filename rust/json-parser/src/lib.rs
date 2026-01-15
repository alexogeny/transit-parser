//! Generic JSON parser with streaming support.

mod reader;
mod writer;

pub use reader::{JsonReader, ReadOptions};
pub use writer::{JsonWriter, WriteOptions};

use serde_json::Value;
use std::path::Path;
use transit_core::ParseError;

/// A parsed JSON document.
#[derive(Debug, Clone, Default)]
pub struct JsonDocument {
    /// The root JSON value.
    pub root: Value,
}

impl JsonDocument {
    /// Create a new document from a JSON value.
    pub fn new(root: Value) -> Self {
        Self { root }
    }

    /// Create an empty object document.
    pub fn empty_object() -> Self {
        Self {
            root: Value::Object(Default::default()),
        }
    }

    /// Create an empty array document.
    pub fn empty_array() -> Self {
        Self {
            root: Value::Array(Default::default()),
        }
    }

    /// Parse a JSON document from path.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        JsonReader::read_path(path.as_ref(), ReadOptions::default())
    }

    /// Parse JSON from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        JsonReader::read_bytes(bytes, ReadOptions::default())
    }

    /// Parse JSON from string.
    pub fn from_str(json: &str) -> Result<Self, ParseError> {
        JsonReader::read_str(json, ReadOptions::default())
    }

    /// Write JSON to path.
    pub fn to_path(&self, path: impl AsRef<Path>) -> Result<(), ParseError> {
        JsonWriter::write_path(&self.root, path.as_ref(), WriteOptions::default())
    }

    /// Write JSON to string.
    pub fn to_string(&self) -> Result<String, ParseError> {
        JsonWriter::write_string(&self.root, WriteOptions::default())
    }

    /// Write JSON to pretty-printed string.
    pub fn to_string_pretty(&self) -> Result<String, ParseError> {
        JsonWriter::write_string(&self.root, WriteOptions { pretty: true })
    }

    /// Check if the root is an object.
    pub fn is_object(&self) -> bool {
        self.root.is_object()
    }

    /// Check if the root is an array.
    pub fn is_array(&self) -> bool {
        self.root.is_array()
    }

    /// Get the root as an array (if it is one).
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        self.root.as_array()
    }

    /// Get a value by JSON pointer (e.g., "/foo/bar/0").
    pub fn pointer(&self, pointer: &str) -> Option<&Value> {
        self.root.pointer(pointer)
    }
}

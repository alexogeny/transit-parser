//! Generic CSV parser with schema inference.

mod reader;
mod schema;
mod writer;

pub use reader::{CsvReader, ReadOptions};
pub use schema::{ColumnType, CsvSchema};
pub use writer::{CsvWriter, WriteOptions};

use serde_json::Value;
use std::path::Path;
use transit_core::ParseError;

/// A parsed CSV document.
#[derive(Debug, Clone, Default)]
pub struct CsvDocument {
    /// Inferred or provided schema.
    pub schema: CsvSchema,
    /// Rows as JSON objects.
    pub rows: Vec<Value>,
}

impl CsvDocument {
    /// Create a new empty document.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a CSV file from path.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        CsvReader::read_path(path.as_ref(), ReadOptions::default())
    }

    /// Parse CSV from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        CsvReader::read_bytes(bytes, ReadOptions::default())
    }

    /// Parse CSV from string.
    pub fn from_str(csv: &str) -> Result<Self, ParseError> {
        CsvReader::read_str(csv, ReadOptions::default())
    }

    /// Write CSV to path.
    pub fn to_path(&self, path: impl AsRef<Path>) -> Result<(), ParseError> {
        CsvWriter::write_path(&self.rows, &self.schema, path.as_ref(), WriteOptions::default())
    }

    /// Write CSV to string.
    pub fn to_string(&self) -> Result<String, ParseError> {
        CsvWriter::write_string(&self.rows, &self.schema, WriteOptions::default())
    }

    /// Get the number of rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if document is empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

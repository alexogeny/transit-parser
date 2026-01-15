//! JSON writer.

use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use transit_core::ParseError;

/// Options for writing JSON files.
#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    /// Whether to pretty-print the output.
    pub pretty: bool,
}

/// JSON writer.
pub struct JsonWriter;

impl JsonWriter {
    /// Write JSON to path.
    pub fn write_path(value: &Value, path: &Path, options: WriteOptions) -> Result<(), ParseError> {
        let mut file = File::create(path)?;
        let json = Self::value_to_string(value, &options)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Write JSON to string.
    pub fn write_string(value: &Value, options: WriteOptions) -> Result<String, ParseError> {
        Self::value_to_string(value, &options)
    }

    fn value_to_string(value: &Value, options: &WriteOptions) -> Result<String, ParseError> {
        if options.pretty {
            serde_json::to_string_pretty(value).map_err(|e| ParseError::Json(e.to_string()))
        } else {
            serde_json::to_string(value).map_err(|e| ParseError::Json(e.to_string()))
        }
    }
}

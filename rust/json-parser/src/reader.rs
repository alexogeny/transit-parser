//! JSON reader.

use crate::JsonDocument;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use transit_core::ParseError;

/// Options for reading JSON files.
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    /// Whether to allow comments in JSON.
    pub allow_comments: bool,
    /// Whether to allow trailing commas.
    pub allow_trailing_comma: bool,
}

/// JSON reader.
pub struct JsonReader;

impl JsonReader {
    /// Read a JSON file from path.
    pub fn read_path(path: &Path, options: ReadOptions) -> Result<JsonDocument, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::read_impl(reader, options)
    }

    /// Read JSON from bytes.
    pub fn read_bytes(bytes: &[u8], options: ReadOptions) -> Result<JsonDocument, ParseError> {
        Self::read_impl(bytes, options)
    }

    /// Read JSON from string.
    pub fn read_str(json: &str, options: ReadOptions) -> Result<JsonDocument, ParseError> {
        Self::read_bytes(json.as_bytes(), options)
    }

    fn read_impl<R: Read>(reader: R, _options: ReadOptions) -> Result<JsonDocument, ParseError> {
        let value: Value =
            serde_json::from_reader(reader).map_err(|e| ParseError::Json(e.to_string()))?;
        Ok(JsonDocument::new(value))
    }
}

/// Iterator for streaming JSON arrays.
#[allow(dead_code)]
pub struct JsonArrayIterator<R: Read> {
    reader: std::io::BufReader<R>,
    buffer: String,
    depth: i32,
    in_string: bool,
    escape_next: bool,
}

#[allow(dead_code)]
impl<R: Read> JsonArrayIterator<R> {
    /// Create a new streaming iterator for a JSON array.
    pub fn new(reader: R) -> Result<Self, ParseError> {
        let mut buf_reader = std::io::BufReader::new(reader);
        let buffer = String::new();

        // Find the opening bracket
        let mut found_start = false;
        let mut byte = [0u8; 1];
        while buf_reader.read(&mut byte)? == 1 {
            let c = byte[0] as char;
            if c.is_whitespace() {
                continue;
            }
            if c == '[' {
                found_start = true;
                break;
            } else {
                return Err(ParseError::Json("Expected array at root".to_string()));
            }
        }

        if !found_start {
            return Err(ParseError::Json("Empty input".to_string()));
        }

        Ok(Self {
            reader: buf_reader,
            buffer,
            depth: 0,
            in_string: false,
            escape_next: false,
        })
    }
}

impl<R: Read> Iterator for JsonArrayIterator<R> {
    type Item = Result<Value, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();
        self.depth = 0;
        self.in_string = false;
        self.escape_next = false;

        let mut byte = [0u8; 1];
        let mut found_start = false;

        while self.reader.read(&mut byte).ok()? == 1 {
            let c = byte[0] as char;

            // Skip whitespace before element
            if !found_start && c.is_whitespace() {
                continue;
            }

            // Skip commas between elements
            if !found_start && c == ',' {
                continue;
            }

            // End of array
            if !found_start && c == ']' {
                return None;
            }

            found_start = true;
            self.buffer.push(c);

            // Track string state
            if self.escape_next {
                self.escape_next = false;
                continue;
            }

            if c == '\\' && self.in_string {
                self.escape_next = true;
                continue;
            }

            if c == '"' {
                self.in_string = !self.in_string;
                continue;
            }

            if self.in_string {
                continue;
            }

            // Track depth for nested structures
            match c {
                '{' | '[' => self.depth += 1,
                '}' | ']' => self.depth -= 1,
                ',' if self.depth == 0 => {
                    // End of element
                    self.buffer.pop(); // Remove trailing comma
                    break;
                }
                _ => {}
            }

            // End of simple value at depth 0
            if self.depth < 0 {
                self.buffer.pop(); // Remove the closing bracket
                break;
            }
        }

        if self.buffer.is_empty() {
            return None;
        }

        // Parse the collected element
        match serde_json::from_str(&self.buffer) {
            Ok(value) => Some(Ok(value)),
            Err(e) => Some(Err(ParseError::Json(e.to_string()))),
        }
    }
}

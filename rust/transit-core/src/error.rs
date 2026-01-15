//! Error types for transit data parsing and conversion.

use thiserror::Error;

/// Errors that can occur during parsing.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV parsing error: {0}")]
    Csv(String),

    #[error("XML parsing error: {0}")]
    Xml(String),

    #[error("JSON parsing error: {0}")]
    Json(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    #[error("Invalid time format: {0}")]
    InvalidTime(String),

    #[error("Unsupported schema version: {0}")]
    UnsupportedVersion(String),

    #[error("Zip archive error: {0}")]
    Zip(String),
}

/// Errors that can occur during format conversion.
#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Mapping error: {field} - {message}")]
    Mapping { field: String, message: String },

    #[error("Missing reference: {ref_type} '{ref_id}' not found")]
    MissingReference { ref_type: String, ref_id: String },

    #[error("Conversion error: {0}")]
    Conversion(String),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
}

/// Errors that can occur during validation.
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Required file missing: {0}")]
    MissingFile(String),

    #[error("Invalid field value in {file}: {field} = '{value}' - {reason}")]
    InvalidField {
        file: String,
        field: String,
        value: String,
        reason: String,
    },

    #[error("Referential integrity error: {source_file}.{source_field} references non-existent {target_file}.{target_field} = '{value}'")]
    ReferentialIntegrity {
        source_file: String,
        source_field: String,
        target_file: String,
        target_field: String,
        value: String,
    },

    #[error("Duplicate ID: {file}.{field} = '{value}'")]
    DuplicateId {
        file: String,
        field: String,
        value: String,
    },
}

/// A warning generated during parsing or conversion (non-fatal).
#[derive(Debug, Clone)]
pub struct Warning {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}

impl Warning {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            location: None,
        }
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

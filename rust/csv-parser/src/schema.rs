//! CSV schema definition and inference.

use serde::{Deserialize, Serialize};

/// Schema for a CSV document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CsvSchema {
    /// Column definitions in order.
    pub columns: Vec<ColumnDefinition>,
}

impl CsvSchema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_columns(columns: Vec<ColumnDefinition>) -> Self {
        Self { columns }
    }

    pub fn add_column(&mut self, name: impl Into<String>, col_type: ColumnType) {
        self.columns.push(ColumnDefinition {
            name: name.into(),
            col_type,
            nullable: true,
        });
    }

    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }
}

/// Definition of a single column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    /// Column name (from header).
    pub name: String,
    /// Inferred or specified type.
    pub col_type: ColumnType,
    /// Whether the column can contain null/empty values.
    pub nullable: bool,
}

/// Column data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColumnType {
    /// String type.
    #[default]
    String,
    /// Integer type.
    Integer,
    /// Floating point type.
    Float,
    /// Boolean type.
    Boolean,
    /// Date type (YYYY-MM-DD).
    Date,
    /// DateTime type.
    DateTime,
}

impl ColumnType {
    /// Infer type from a string value.
    pub fn infer(value: &str) -> Self {
        if value.is_empty() {
            return Self::String; // Can't infer from empty
        }

        // Try boolean
        if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
            return Self::Boolean;
        }

        // Try integer
        if value.parse::<i64>().is_ok() {
            return Self::Integer;
        }

        // Try float
        if value.parse::<f64>().is_ok() {
            return Self::Float;
        }

        // Try date (YYYY-MM-DD or YYYYMMDD)
        if value.len() == 10
            && value.chars().nth(4) == Some('-')
            && chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
        {
            return Self::Date;
        }
        if value.len() == 8
            && value.chars().all(|c| c.is_ascii_digit())
            && chrono::NaiveDate::parse_from_str(value, "%Y%m%d").is_ok()
        {
            return Self::Date;
        }

        Self::String
    }

    /// Merge two types to find common type.
    pub fn merge(self, other: Self) -> Self {
        if self == other {
            return self;
        }

        // String is the most general type
        if self == Self::String || other == Self::String {
            return Self::String;
        }

        // Integer can be promoted to Float
        if (self == Self::Integer && other == Self::Float)
            || (self == Self::Float && other == Self::Integer)
        {
            return Self::Float;
        }

        // Default to String for incompatible types
        Self::String
    }
}

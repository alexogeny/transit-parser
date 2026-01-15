//! CSV reader.

use crate::schema::{ColumnDefinition, ColumnType, CsvSchema};
use crate::CsvDocument;
use csv::ReaderBuilder;
use serde_json::{Map, Value};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use transit_core::ParseError;

/// Options for reading CSV files.
#[derive(Debug, Clone)]
pub struct ReadOptions {
    /// Whether to infer column types.
    pub infer_types: bool,
    /// Delimiter character.
    pub delimiter: u8,
    /// Whether the first row is a header.
    pub has_header: bool,
    /// Whether to be lenient with malformed data.
    pub lenient: bool,
    /// Number of rows to sample for type inference.
    pub sample_size: usize,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            infer_types: true,
            delimiter: b',',
            has_header: true,
            lenient: false,
            sample_size: 100,
        }
    }
}

/// CSV reader.
pub struct CsvReader;

impl CsvReader {
    /// Read a CSV file from path.
    pub fn read_path(path: &Path, options: ReadOptions) -> Result<CsvDocument, ParseError> {
        let file = File::open(path)?;
        Self::read_impl(file, options)
    }

    /// Read CSV from bytes.
    pub fn read_bytes(bytes: &[u8], options: ReadOptions) -> Result<CsvDocument, ParseError> {
        Self::read_impl(bytes, options)
    }

    /// Read CSV from string.
    pub fn read_str(csv: &str, options: ReadOptions) -> Result<CsvDocument, ParseError> {
        Self::read_bytes(csv.as_bytes(), options)
    }

    fn read_impl<R: Read>(reader: R, options: ReadOptions) -> Result<CsvDocument, ParseError> {
        let mut csv_reader = ReaderBuilder::new()
            .delimiter(options.delimiter)
            .has_headers(options.has_header)
            .flexible(options.lenient)
            .trim(csv::Trim::All)
            .from_reader(reader);

        // Get headers
        let headers: Vec<String> = if options.has_header {
            csv_reader
                .headers()
                .map_err(|e| ParseError::Csv(e.to_string()))?
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        // Read all records
        let mut records: Vec<csv::StringRecord> = Vec::new();
        for result in csv_reader.records() {
            match result {
                Ok(record) => records.push(record),
                Err(e) if options.lenient => {
                    eprintln!("Warning: skipping malformed record: {}", e);
                }
                Err(e) => return Err(ParseError::Csv(e.to_string())),
            }
        }

        // Infer schema
        let schema = if options.infer_types {
            Self::infer_schema(&headers, &records, options.sample_size)
        } else {
            CsvSchema::with_columns(
                headers
                    .iter()
                    .map(|name| ColumnDefinition {
                        name: name.clone(),
                        col_type: ColumnType::String,
                        nullable: true,
                    })
                    .collect(),
            )
        };

        // Convert records to JSON
        let rows: Vec<Value> = records
            .iter()
            .map(|record| Self::record_to_json(record, &schema))
            .collect();

        Ok(CsvDocument { schema, rows })
    }

    fn infer_schema(headers: &[String], records: &[csv::StringRecord], sample_size: usize) -> CsvSchema {
        let sample = &records[..records.len().min(sample_size)];

        let columns: Vec<ColumnDefinition> = headers
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let mut col_type = ColumnType::String;
                let mut nullable = false;
                let mut first = true;

                for record in sample {
                    if let Some(value) = record.get(i) {
                        if value.is_empty() {
                            nullable = true;
                            continue;
                        }

                        let inferred = ColumnType::infer(value);
                        if first {
                            col_type = inferred;
                            first = false;
                        } else {
                            col_type = col_type.merge(inferred);
                        }
                    }
                }

                ColumnDefinition {
                    name: name.clone(),
                    col_type,
                    nullable,
                }
            })
            .collect();

        CsvSchema::with_columns(columns)
    }

    fn record_to_json(record: &csv::StringRecord, schema: &CsvSchema) -> Value {
        let mut map = Map::new();

        for (i, col) in schema.columns.iter().enumerate() {
            let value = record.get(i).unwrap_or("");

            let json_value = if value.is_empty() {
                Value::Null
            } else {
                match col.col_type {
                    ColumnType::String => Value::String(value.to_string()),
                    ColumnType::Integer => value
                        .parse::<i64>()
                        .map(Value::from)
                        .unwrap_or_else(|_| Value::String(value.to_string())),
                    ColumnType::Float => value
                        .parse::<f64>()
                        .map(Value::from)
                        .unwrap_or_else(|_| Value::String(value.to_string())),
                    ColumnType::Boolean => value
                        .parse::<bool>()
                        .map(Value::from)
                        .unwrap_or_else(|_| Value::String(value.to_string())),
                    ColumnType::Date | ColumnType::DateTime => Value::String(value.to_string()),
                }
            };

            map.insert(col.name.clone(), json_value);
        }

        Value::Object(map)
    }
}

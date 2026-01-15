//! CSV writer.

use crate::schema::CsvSchema;
use csv::WriterBuilder;
use serde_json::Value;
use std::fs::File;
use std::path::Path;
use transit_core::ParseError;

/// Options for writing CSV files.
#[derive(Debug, Clone)]
pub struct WriteOptions {
    /// Delimiter character.
    pub delimiter: u8,
    /// Whether to include header row.
    pub include_header: bool,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            delimiter: b',',
            include_header: true,
        }
    }
}

/// CSV writer.
pub struct CsvWriter;

impl CsvWriter {
    /// Write CSV to path.
    pub fn write_path(
        rows: &[Value],
        schema: &CsvSchema,
        path: &Path,
        options: WriteOptions,
    ) -> Result<(), ParseError> {
        let file = File::create(path)?;
        Self::write_impl(rows, schema, file, options)
    }

    /// Write CSV to string.
    pub fn write_string(
        rows: &[Value],
        schema: &CsvSchema,
        options: WriteOptions,
    ) -> Result<String, ParseError> {
        let mut buffer = Vec::new();
        Self::write_impl(rows, schema, &mut buffer, options)?;
        String::from_utf8(buffer).map_err(|e| ParseError::Csv(e.to_string()))
    }

    fn write_impl<W: std::io::Write>(
        rows: &[Value],
        schema: &CsvSchema,
        writer: W,
        options: WriteOptions,
    ) -> Result<(), ParseError> {
        let mut csv_writer = WriterBuilder::new()
            .delimiter(options.delimiter)
            .from_writer(writer);

        // Write header
        if options.include_header {
            let headers: Vec<&str> = schema.columns.iter().map(|c| c.name.as_str()).collect();
            csv_writer
                .write_record(&headers)
                .map_err(|e| ParseError::Csv(e.to_string()))?;
        }

        // Write rows
        for row in rows {
            let record: Vec<String> = schema
                .columns
                .iter()
                .map(|col| Self::value_to_string(row.get(&col.name)))
                .collect();
            csv_writer
                .write_record(&record)
                .map_err(|e| ParseError::Csv(e.to_string()))?;
        }

        csv_writer.flush().map_err(|e| ParseError::Csv(e.to_string()))?;
        Ok(())
    }

    fn value_to_string(value: Option<&Value>) -> String {
        match value {
            None | Some(Value::Null) => String::new(),
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(v) => v.to_string(),
        }
    }
}

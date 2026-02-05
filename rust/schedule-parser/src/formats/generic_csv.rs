//! Generic CSV exporter with configurable columns.

use crate::models::{seconds_to_time_string, Schedule, ScheduleRow};
use csv::Writer;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use transit_core::ParseError;

/// Time format for export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TimeFormat {
    /// HH:MM:SS format.
    #[default]
    HhMmSs,
    /// HH:MM format.
    HhMm,
    /// Seconds since midnight.
    Seconds,
}

/// Column configuration for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnConfig {
    /// Internal field name.
    pub field: String,
    /// Header name in output.
    pub header: String,
    /// Whether to include this column.
    pub include: bool,
}

impl ColumnConfig {
    /// Create a new column config.
    pub fn new(field: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            header: header.into(),
            include: true,
        }
    }

    /// Create excluded column.
    pub fn excluded(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            header: String::new(),
            include: false,
        }
    }
}

/// Export configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Columns to export (in order).
    pub columns: Vec<ColumnConfig>,
    /// Time format.
    pub time_format: TimeFormat,
    /// Delimiter character.
    pub delimiter: u8,
    /// Whether to include header row.
    pub include_header: bool,
    /// Value for null/empty fields.
    pub null_value: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            columns: Self::default_columns(),
            time_format: TimeFormat::HhMmSs,
            delimiter: b',',
            include_header: true,
            null_value: String::new(),
        }
    }
}

impl ExportConfig {
    /// Create a new export config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create config with specific columns.
    pub fn with_columns(columns: Vec<&str>) -> Self {
        let cols = columns
            .into_iter()
            .map(|c| ColumnConfig::new(c, c))
            .collect();

        Self {
            columns: cols,
            ..Default::default()
        }
    }

    /// Create config with column mapping.
    pub fn with_column_mapping(mapping: Vec<(&str, &str)>) -> Self {
        let cols = mapping
            .into_iter()
            .map(|(field, header)| ColumnConfig::new(field, header))
            .collect();

        Self {
            columns: cols,
            ..Default::default()
        }
    }

    /// Set time format.
    pub fn time_format(mut self, format: TimeFormat) -> Self {
        self.time_format = format;
        self
    }

    /// Set delimiter.
    pub fn delimiter(mut self, delim: u8) -> Self {
        self.delimiter = delim;
        self
    }

    /// Set null value.
    pub fn null_value(mut self, value: impl Into<String>) -> Self {
        self.null_value = value.into();
        self
    }

    /// Default column configuration.
    fn default_columns() -> Vec<ColumnConfig> {
        vec![
            ColumnConfig::new("run_number", "run_number"),
            ColumnConfig::new("block", "block"),
            ColumnConfig::new("start_place", "start_place"),
            ColumnConfig::new("end_place", "end_place"),
            ColumnConfig::new("start_time", "start_time"),
            ColumnConfig::new("end_time", "end_time"),
            ColumnConfig::new("trip_id", "trip_id"),
            ColumnConfig::new("depot", "depot"),
            ColumnConfig::new("vehicle_class", "vehicle_class"),
            ColumnConfig::new("vehicle_type", "vehicle_type"),
            ColumnConfig::new("start_lat", "start_lat"),
            ColumnConfig::new("start_lon", "start_lon"),
            ColumnConfig::new("end_lat", "end_lat"),
            ColumnConfig::new("end_lon", "end_lon"),
            ColumnConfig::new("route_shape_id", "route_shape_id"),
        ]
    }
}

/// CSV exporter for schedules.
pub struct CsvExporter {
    config: ExportConfig,
}

impl CsvExporter {
    /// Create a new exporter with the given config.
    pub fn new(config: ExportConfig) -> Self {
        Self { config }
    }

    /// Create an exporter with default config.
    pub fn default_config() -> Self {
        Self::new(ExportConfig::default())
    }

    /// Export schedule to a file.
    pub fn export_to_path(
        &self,
        schedule: &Schedule,
        path: impl AsRef<Path>,
    ) -> Result<(), ParseError> {
        let file = File::create(path).map_err(ParseError::Io)?;
        self.export_to_writer(schedule, file)
    }

    /// Export schedule to a writer.
    pub fn export_to_writer<W: Write>(
        &self,
        schedule: &Schedule,
        writer: W,
    ) -> Result<(), ParseError> {
        let mut csv_writer = Writer::from_writer(writer);

        // Write header
        if self.config.include_header {
            let headers: Vec<&str> = self
                .config
                .columns
                .iter()
                .filter(|c| c.include)
                .map(|c| c.header.as_str())
                .collect();
            csv_writer
                .write_record(&headers)
                .map_err(|e| ParseError::Csv(e.to_string()))?;
        }

        // Write rows
        for row in &schedule.rows {
            let record = self.row_to_record(row);
            csv_writer
                .write_record(&record)
                .map_err(|e| ParseError::Csv(e.to_string()))?;
        }

        csv_writer
            .flush()
            .map_err(|e| ParseError::Csv(e.to_string()))?;
        Ok(())
    }

    /// Export schedule to string.
    pub fn export_to_string(&self, schedule: &Schedule) -> Result<String, ParseError> {
        let mut buffer = Vec::new();
        self.export_to_writer(schedule, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| ParseError::Csv(e.to_string()))
    }

    /// Export schedule to bytes.
    pub fn export_to_bytes(&self, schedule: &Schedule) -> Result<Vec<u8>, ParseError> {
        let mut buffer = Vec::new();
        self.export_to_writer(schedule, &mut buffer)?;
        Ok(buffer)
    }

    /// Convert a schedule row to a CSV record.
    fn row_to_record(&self, row: &ScheduleRow) -> Vec<String> {
        self.config
            .columns
            .iter()
            .filter(|c| c.include)
            .map(|col| self.get_field_value(row, &col.field))
            .collect()
    }

    /// Get a field value from a row.
    fn get_field_value(&self, row: &ScheduleRow, field: &str) -> String {
        let value = match field {
            "run_number" => row.run_number.clone(),
            "block" => row.block.clone(),
            "start_place" => row.start_place.clone(),
            "end_place" => row.end_place.clone(),
            "start_time" => row.start_time.as_ref().map(|t| self.format_time(t)),
            "end_time" => row.end_time.as_ref().map(|t| self.format_time(t)),
            "trip_id" => row.trip_id.clone(),
            "depot" => row.depot.clone(),
            "vehicle_class" => row.vehicle_class.clone(),
            "vehicle_type" => row.vehicle_type.clone(),
            "start_lat" => row.start_lat.map(|v| v.to_string()),
            "start_lon" => row.start_lon.map(|v| v.to_string()),
            "end_lat" => row.end_lat.map(|v| v.to_string()),
            "end_lon" => row.end_lon.map(|v| v.to_string()),
            "route_shape_id" => row.route_shape_id.clone(),
            "row_type" => Some(format!("{:?}", row.row_type).to_lowercase()),
            "duty_id" => row.duty_id.clone(),
            "shift_id" => row.shift_id.clone(),
            "route_short_name" => row.route_short_name.clone(),
            "headsign" => row.headsign.clone(),
            _ => None,
        };

        value.unwrap_or_else(|| self.config.null_value.clone())
    }

    /// Format a time string according to config.
    fn format_time(&self, time: &str) -> String {
        match self.config.time_format {
            TimeFormat::HhMmSs => {
                // Already in HH:MM:SS or convert from seconds
                if time.contains(':') {
                    time.to_string()
                } else if let Ok(secs) = time.parse::<u32>() {
                    seconds_to_time_string(secs)
                } else {
                    time.to_string()
                }
            }
            TimeFormat::HhMm => {
                // Convert to HH:MM
                if time.contains(':') {
                    let parts: Vec<&str> = time.split(':').collect();
                    if parts.len() >= 2 {
                        format!("{}:{}", parts[0], parts[1])
                    } else {
                        time.to_string()
                    }
                } else if let Ok(secs) = time.parse::<u32>() {
                    let hours = secs / 3600;
                    let minutes = (secs % 3600) / 60;
                    format!("{:02}:{:02}", hours, minutes)
                } else {
                    time.to_string()
                }
            }
            TimeFormat::Seconds => {
                // Convert to seconds
                if time.contains(':') {
                    let parts: Vec<&str> = time.split(':').collect();
                    match parts.len() {
                        3 => {
                            let hours: u32 = parts[0].parse().unwrap_or(0);
                            let minutes: u32 = parts[1].parse().unwrap_or(0);
                            let seconds: u32 = parts[2].parse().unwrap_or(0);
                            (hours * 3600 + minutes * 60 + seconds).to_string()
                        }
                        2 => {
                            let hours: u32 = parts[0].parse().unwrap_or(0);
                            let minutes: u32 = parts[1].parse().unwrap_or(0);
                            (hours * 3600 + minutes * 60).to_string()
                        }
                        _ => time.to_string(),
                    }
                } else {
                    time.to_string()
                }
            }
        }
    }
}

impl Default for CsvExporter {
    fn default() -> Self {
        Self::default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RowType;

    fn make_row() -> ScheduleRow {
        ScheduleRow {
            run_number: Some("R1".to_string()),
            block: Some("B1".to_string()),
            start_place: Some("STOP_A".to_string()),
            end_place: Some("STOP_B".to_string()),
            start_time: Some("08:00:00".to_string()),
            end_time: Some("09:00:00".to_string()),
            trip_id: Some("TRIP1".to_string()),
            row_type: RowType::Revenue,
            ..Default::default()
        }
    }

    #[test]
    fn test_export_default() {
        let schedule = Schedule::from_rows(vec![make_row()]);
        let exporter = CsvExporter::default_config();

        let result = exporter.export_to_string(&schedule).unwrap();

        assert!(result.contains("run_number"));
        assert!(result.contains("R1"));
        assert!(result.contains("TRIP1"));
    }

    #[test]
    fn test_export_custom_columns() {
        let schedule = Schedule::from_rows(vec![make_row()]);
        let config = ExportConfig::with_columns(vec!["run_number", "block", "trip_id"]);
        let exporter = CsvExporter::new(config);

        let result = exporter.export_to_string(&schedule).unwrap();

        assert!(result.contains("run_number,block,trip_id"));
        assert!(result.contains("R1,B1,TRIP1"));
        assert!(!result.contains("start_place"));
    }

    #[test]
    fn test_export_column_mapping() {
        let schedule = Schedule::from_rows(vec![make_row()]);
        let config = ExportConfig::with_column_mapping(vec![
            ("run_number", "driver"),
            ("block", "vehicle_block"),
        ]);
        let exporter = CsvExporter::new(config);

        let result = exporter.export_to_string(&schedule).unwrap();

        assert!(result.contains("driver,vehicle_block"));
        assert!(result.contains("R1,B1"));
    }

    #[test]
    fn test_time_format_seconds() {
        let schedule = Schedule::from_rows(vec![make_row()]);
        let config = ExportConfig::with_columns(vec!["start_time", "end_time"])
            .time_format(TimeFormat::Seconds);
        let exporter = CsvExporter::new(config);

        let result = exporter.export_to_string(&schedule).unwrap();

        assert!(result.contains("28800")); // 08:00:00
        assert!(result.contains("32400")); // 09:00:00
    }

    #[test]
    fn test_time_format_hhmm() {
        let schedule = Schedule::from_rows(vec![make_row()]);
        let config = ExportConfig::with_columns(vec!["start_time"]).time_format(TimeFormat::HhMm);
        let exporter = CsvExporter::new(config);

        let result = exporter.export_to_string(&schedule).unwrap();

        assert!(result.contains("08:00"));
        assert!(!result.contains("08:00:00"));
    }

    #[test]
    fn test_null_value() {
        let mut row = make_row();
        row.depot = None;

        let schedule = Schedule::from_rows(vec![row]);
        let config = ExportConfig::with_columns(vec!["run_number", "depot"]).null_value("N/A");
        let exporter = CsvExporter::new(config);

        let result = exporter.export_to_string(&schedule).unwrap();

        assert!(result.contains("R1,N/A"));
    }
}

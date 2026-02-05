//! CSV reader for schedule files with flexible column mapping.

use crate::models::{RowType, Schedule, ScheduleRow};
use csv::StringRecord;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use transit_core::ParseError;

/// Column mapping for schedule CSV files.
///
/// Maps standard field names to actual column names in the CSV.
#[derive(Debug, Clone, Default)]
pub struct ColumnMapping {
    /// Map of standard field name -> CSV column name.
    mappings: HashMap<String, String>,
}

impl ColumnMapping {
    /// Create a new empty mapping.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create mapping from a HashMap.
    pub fn from_map(mappings: HashMap<String, String>) -> Self {
        Self { mappings }
    }

    /// Add a mapping.
    pub fn add(&mut self, field: impl Into<String>, column: impl Into<String>) {
        self.mappings.insert(field.into(), column.into());
    }

    /// Get the CSV column name for a standard field.
    pub fn get_column(&self, field: &str) -> Option<&str> {
        self.mappings.get(field).map(|s| s.as_str())
    }

    /// Create a default mapping with standard column names.
    pub fn default_mapping() -> Self {
        let mut m = Self::new();
        m.add("run_number", "run_number");
        m.add("block", "block");
        m.add("start_place", "start_place");
        m.add("end_place", "end_place");
        m.add("start_time", "start_time");
        m.add("end_time", "end_time");
        m.add("trip_id", "trip_id");
        m.add("depot", "depot");
        m.add("vehicle_class", "vehicle_class");
        m.add("vehicle_type", "vehicle_type");
        m.add("start_lat", "start_lat");
        m.add("start_lon", "start_lon");
        m.add("end_lat", "end_lat");
        m.add("end_lon", "end_lon");
        m.add("route_shape_id", "route_shape_id");
        m.add("row_type", "row_type");
        m.add("duty_id", "duty_id");
        m.add("shift_id", "shift_id");
        m.add("route_short_name", "route_short_name");
        m.add("headsign", "headsign");
        m
    }

    /// Try to auto-detect column mapping from headers.
    ///
    /// Uses fuzzy matching to find likely column mappings.
    pub fn auto_detect(headers: &[String]) -> Self {
        let mut mapping = Self::new();

        let patterns: &[(&str, &[&str])] = &[
            (
                "run_number",
                &["run_number", "run", "run_no", "driver_id", "operator_id"],
            ),
            ("block", &["block", "block_id", "vehicle_block", "blk"]),
            (
                "start_place",
                &[
                    "start_place",
                    "start_stop",
                    "from_stop",
                    "origin",
                    "start_location",
                ],
            ),
            (
                "end_place",
                &[
                    "end_place",
                    "end_stop",
                    "to_stop",
                    "destination",
                    "end_location",
                ],
            ),
            (
                "start_time",
                &["start_time", "departure_time", "depart_time", "start"],
            ),
            (
                "end_time",
                &["end_time", "arrival_time", "arrive_time", "end"],
            ),
            ("trip_id", &["trip_id", "trip", "gtfs_trip_id"]),
            ("depot", &["depot", "garage", "depot_id", "base"]),
            ("vehicle_class", &["vehicle_class", "veh_class", "bus_type"]),
            ("vehicle_type", &["vehicle_type", "veh_type", "vehicle"]),
            ("start_lat", &["start_lat", "from_lat", "origin_lat"]),
            ("start_lon", &["start_lon", "from_lon", "origin_lon"]),
            ("end_lat", &["end_lat", "to_lat", "dest_lat"]),
            ("end_lon", &["end_lon", "to_lon", "dest_lon"]),
            ("route_shape_id", &["route_shape_id", "shape_id"]),
            (
                "row_type",
                &["row_type", "type", "activity_type", "movement_type"],
            ),
            ("duty_id", &["duty_id", "duty", "crew_id"]),
            ("shift_id", &["shift_id", "shift"]),
            ("route_short_name", &["route_short_name", "route", "line"]),
            ("headsign", &["headsign", "destination", "direction"]),
        ];

        for (field, possible_names) in patterns {
            for header in headers {
                let header_lower = header.to_lowercase();
                for &name in *possible_names {
                    if header_lower == name || header_lower.contains(name) {
                        mapping.add(*field, header.clone());
                        break;
                    }
                }
                if mapping.get_column(field).is_some() {
                    break;
                }
            }
        }

        mapping
    }
}

/// Options for reading schedule CSV files.
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    /// Column mapping to use.
    pub column_mapping: Option<ColumnMapping>,

    /// Whether to auto-detect column mapping if not provided.
    pub auto_detect_columns: bool,

    /// CSV delimiter character.
    pub delimiter: Option<u8>,

    /// Whether the CSV has headers.
    pub has_headers: bool,

    /// Skip rows where all fields are empty.
    pub skip_empty_rows: bool,
}

impl ReadOptions {
    /// Create default options.
    pub fn new() -> Self {
        Self {
            column_mapping: None,
            auto_detect_columns: true,
            delimiter: None,
            has_headers: true,
            skip_empty_rows: true,
        }
    }

    /// Set column mapping.
    pub fn with_mapping(mut self, mapping: ColumnMapping) -> Self {
        self.column_mapping = Some(mapping);
        self
    }

    /// Set delimiter.
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = Some(delimiter);
        self
    }
}

/// Schedule CSV reader.
pub struct ScheduleReader;

impl ScheduleReader {
    /// Read a schedule from a file path.
    pub fn read_path(path: impl AsRef<Path>, options: ReadOptions) -> Result<Schedule, ParseError> {
        let path = path.as_ref();
        let file = File::open(path).map_err(ParseError::Io)?;
        let reader = BufReader::new(file);

        let mut schedule = Self::read_reader(reader, options)?;
        schedule.metadata.source_file = path.to_string_lossy().into_owned().into();
        Ok(schedule)
    }

    /// Read a schedule from bytes.
    pub fn read_bytes(bytes: &[u8], options: ReadOptions) -> Result<Schedule, ParseError> {
        Self::read_reader(bytes, options)
    }

    /// Read a schedule from a string.
    pub fn read_str(csv_str: &str, options: ReadOptions) -> Result<Schedule, ParseError> {
        Self::read_bytes(csv_str.as_bytes(), options)
    }

    /// Read from any reader.
    fn read_reader<R: Read>(reader: R, options: ReadOptions) -> Result<Schedule, ParseError> {
        let mut csv_builder = csv::ReaderBuilder::new();
        csv_builder.has_headers(options.has_headers);

        if let Some(delim) = options.delimiter {
            csv_builder.delimiter(delim);
        }

        let mut csv_reader = csv_builder.from_reader(reader);

        // Get headers
        let headers: Vec<String> = if options.has_headers {
            csv_reader
                .headers()
                .map_err(|e| ParseError::Csv(e.to_string()))?
                .iter()
                .map(String::from)
                .collect()
        } else {
            Vec::new()
        };

        // Determine column mapping
        let mapping = match options.column_mapping {
            Some(m) => m,
            None if options.auto_detect_columns && !headers.is_empty() => {
                ColumnMapping::auto_detect(&headers)
            }
            None => ColumnMapping::default_mapping(),
        };

        // Create header index map
        let header_index: HashMap<String, usize> = headers
            .iter()
            .enumerate()
            .map(|(i, h)| (h.clone(), i))
            .collect();

        // Parse rows
        let mut rows = Vec::new();
        for result in csv_reader.records() {
            let record = result.map_err(|e| ParseError::Csv(e.to_string()))?;

            if options.skip_empty_rows && record.iter().all(|f| f.trim().is_empty()) {
                continue;
            }

            let row = Self::parse_row(&record, &mapping, &header_index)?;
            rows.push(row);
        }

        let mut schedule = Schedule::from_rows(rows);
        schedule.metadata.column_mapping = Some(
            mapping
                .mappings
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        );

        Ok(schedule)
    }

    /// Parse a single record into a ScheduleRow.
    fn parse_row(
        record: &StringRecord,
        mapping: &ColumnMapping,
        header_index: &HashMap<String, usize>,
    ) -> Result<ScheduleRow, ParseError> {
        let get_field = |field: &str| -> Option<String> {
            let column = mapping.get_column(field)?;
            let idx = *header_index.get(column)?;
            let value = record.get(idx)?.trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            }
        };

        let get_f64 =
            |field: &str| -> Option<f64> { get_field(field).and_then(|s| s.parse().ok()) };

        let row_type = get_field("row_type")
            .map(|s| parse_row_type(&s))
            .unwrap_or(RowType::Revenue);

        Ok(ScheduleRow {
            run_number: get_field("run_number"),
            block: get_field("block"),
            start_place: get_field("start_place"),
            end_place: get_field("end_place"),
            start_time: get_field("start_time"),
            end_time: get_field("end_time"),
            trip_id: get_field("trip_id"),
            depot: get_field("depot"),
            vehicle_class: get_field("vehicle_class"),
            vehicle_type: get_field("vehicle_type"),
            start_lat: get_f64("start_lat"),
            start_lon: get_f64("start_lon"),
            end_lat: get_f64("end_lat"),
            end_lon: get_f64("end_lon"),
            route_shape_id: get_field("route_shape_id"),
            row_type,
            duty_id: get_field("duty_id"),
            shift_id: get_field("shift_id"),
            route_short_name: get_field("route_short_name"),
            headsign: get_field("headsign"),
        })
    }
}

/// Parse a row type string to RowType enum.
fn parse_row_type(s: &str) -> RowType {
    match s.to_lowercase().as_str() {
        "revenue" | "trip" | "service" => RowType::Revenue,
        "pull_out" | "pullout" | "pull-out" | "po" => RowType::PullOut,
        "pull_in" | "pullin" | "pull-in" | "pi" => RowType::PullIn,
        "deadhead" | "dh" | "dead" | "non_revenue" => RowType::Deadhead,
        "break" | "brk" | "meal" => RowType::Break,
        "relief" | "changeover" | "swap" => RowType::Relief,
        "layover" | "wait" | "dwell" => RowType::Layover,
        _ => RowType::Revenue, // Default to revenue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_simple_csv() {
        let csv = r#"run_number,block,start_place,end_place,start_time,end_time,trip_id
R1,B1,STOP_A,STOP_B,08:00:00,09:00:00,TRIP1
R1,B1,STOP_B,STOP_C,09:15:00,10:00:00,TRIP2
"#;

        let schedule = ScheduleReader::read_str(csv, ReadOptions::new()).unwrap();
        assert_eq!(schedule.len(), 2);
        assert_eq!(schedule.rows[0].trip_id, Some("TRIP1".to_string()));
        assert_eq!(schedule.rows[1].start_place, Some("STOP_B".to_string()));
    }

    #[test]
    fn test_auto_detect_columns() {
        let csv = r#"run,blk,from_stop,to_stop,departure_time,arrival_time,gtfs_trip_id
R1,B1,STOP_A,STOP_B,08:00:00,09:00:00,TRIP1
"#;

        let schedule = ScheduleReader::read_str(csv, ReadOptions::new()).unwrap();
        assert_eq!(schedule.len(), 1);
        assert_eq!(schedule.rows[0].block, Some("B1".to_string()));
        assert_eq!(schedule.rows[0].trip_id, Some("TRIP1".to_string()));
    }

    #[test]
    fn test_row_type_parsing() {
        let csv = r#"run_number,block,start_time,end_time,trip_id,row_type
R1,B1,06:00:00,06:30:00,,pull_out
R1,B1,06:30:00,08:00:00,TRIP1,revenue
R1,B1,08:00:00,08:15:00,,break
R1,B1,08:15:00,09:30:00,TRIP2,revenue
R1,B1,09:30:00,10:00:00,,pull_in
"#;

        let schedule = ScheduleReader::read_str(csv, ReadOptions::new()).unwrap();
        assert_eq!(schedule.len(), 5);
        assert_eq!(schedule.rows[0].row_type, RowType::PullOut);
        assert_eq!(schedule.rows[1].row_type, RowType::Revenue);
        assert_eq!(schedule.rows[2].row_type, RowType::Break);
        assert_eq!(schedule.rows[4].row_type, RowType::PullIn);
    }

    #[test]
    fn test_custom_mapping() {
        let csv = r#"driver,bus_block,origin,destination,depart,arrive,trip
D1,V1,A,B,08:00:00,09:00:00,T1
"#;

        let mut mapping = ColumnMapping::new();
        mapping.add("run_number", "driver");
        mapping.add("block", "bus_block");
        mapping.add("start_place", "origin");
        mapping.add("end_place", "destination");
        mapping.add("start_time", "depart");
        mapping.add("end_time", "arrive");
        mapping.add("trip_id", "trip");

        let options = ReadOptions::new().with_mapping(mapping);
        let schedule = ScheduleReader::read_str(csv, options).unwrap();

        assert_eq!(schedule.rows[0].run_number, Some("D1".to_string()));
        assert_eq!(schedule.rows[0].block, Some("V1".to_string()));
        assert_eq!(schedule.rows[0].trip_id, Some("T1".to_string()));
    }

    #[test]
    fn test_skip_empty_rows() {
        let csv = r#"run_number,block,start_time,trip_id
R1,B1,08:00:00,T1

R1,B1,09:00:00,T2
"#;

        let schedule = ScheduleReader::read_str(csv, ReadOptions::new()).unwrap();
        assert_eq!(schedule.len(), 2); // Empty row skipped
    }
}

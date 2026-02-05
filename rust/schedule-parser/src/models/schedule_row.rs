//! Schedule row model - the primary artifact of a schedule.

use serde::{Deserialize, Serialize};

/// Type of schedule row indicating what kind of movement it represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RowType {
    /// Revenue service trip (has a trip_id).
    #[default]
    Revenue,
    /// Pull-out from depot to first stop.
    PullOut,
    /// Pull-in from last stop to depot.
    PullIn,
    /// Deadhead between trips (interlining).
    Deadhead,
    /// Driver break.
    Break,
    /// Driver relief/changeover.
    Relief,
    /// Layover at a stop.
    Layover,
}

/// A single row in a schedule file.
///
/// This represents one movement or activity in the schedule, which could be:
/// - A revenue trip (references GTFS trip_id)
/// - A deadhead movement (pull-out, pull-in, or interlining)
/// - A break or relief
///
/// Format:
/// ```text
/// run_number, block, start_place, end_place, start_time, end_time,
/// trip_id, depot, vehicle_class, vehicle_type,
/// start_lat, start_lon, end_lat, end_lon, route_shape_id
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleRow {
    /// Run number (driver assignment identifier).
    pub run_number: Option<String>,

    /// Block identifier (vehicle assignment).
    pub block: Option<String>,

    /// Starting location (stop_id, depot code, or descriptive name).
    pub start_place: Option<String>,

    /// Ending location (stop_id, depot code, or descriptive name).
    pub end_place: Option<String>,

    /// Start time in HH:MM:SS or seconds format.
    pub start_time: Option<String>,

    /// End time in HH:MM:SS or seconds format.
    pub end_time: Option<String>,

    /// GTFS trip_id (None for deadheads/breaks).
    pub trip_id: Option<String>,

    /// Depot code for this block.
    pub depot: Option<String>,

    /// Vehicle class/category.
    pub vehicle_class: Option<String>,

    /// Specific vehicle type.
    pub vehicle_type: Option<String>,

    /// Starting latitude (for mapping).
    pub start_lat: Option<f64>,

    /// Starting longitude (for mapping).
    pub start_lon: Option<f64>,

    /// Ending latitude (for mapping).
    pub end_lat: Option<f64>,

    /// Ending longitude (for mapping).
    pub end_lon: Option<f64>,

    /// Route shape ID from GTFS shapes.txt.
    pub route_shape_id: Option<String>,

    /// Type of this row (revenue, deadhead, break, etc.).
    #[serde(default)]
    pub row_type: RowType,

    /// Duty identifier (for rostering).
    pub duty_id: Option<String>,

    /// Shift identifier (for rostering).
    pub shift_id: Option<String>,

    /// Route short name (for reference).
    pub route_short_name: Option<String>,

    /// Headsign/destination.
    pub headsign: Option<String>,
}

impl ScheduleRow {
    /// Create a new empty schedule row.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if this is a revenue (passenger-carrying) trip.
    pub fn is_revenue(&self) -> bool {
        self.row_type == RowType::Revenue && self.trip_id.is_some()
    }

    /// Check if this is any type of deadhead movement.
    pub fn is_deadhead(&self) -> bool {
        matches!(
            self.row_type,
            RowType::PullOut | RowType::PullIn | RowType::Deadhead
        )
    }

    /// Check if this is a break or relief.
    pub fn is_break_or_relief(&self) -> bool {
        matches!(self.row_type, RowType::Break | RowType::Relief)
    }

    /// Parse start_time as seconds since midnight.
    pub fn start_time_seconds(&self) -> Option<u32> {
        self.start_time
            .as_ref()
            .and_then(|t| parse_time_to_seconds(t))
    }

    /// Parse end_time as seconds since midnight.
    pub fn end_time_seconds(&self) -> Option<u32> {
        self.end_time
            .as_ref()
            .and_then(|t| parse_time_to_seconds(t))
    }

    /// Calculate duration in seconds.
    pub fn duration_seconds(&self) -> Option<u32> {
        match (self.start_time_seconds(), self.end_time_seconds()) {
            (Some(start), Some(end)) if end >= start => Some(end - start),
            _ => None,
        }
    }
}

/// Parse a time string to seconds since midnight.
///
/// Supports formats:
/// - HH:MM:SS (e.g., "14:30:00")
/// - HH:MM (e.g., "14:30")
/// - Seconds as integer (e.g., "52200")
fn parse_time_to_seconds(time: &str) -> Option<u32> {
    // Try parsing as plain seconds first
    if let Ok(secs) = time.parse::<u32>() {
        return Some(secs);
    }

    // Try HH:MM:SS or HH:MM format
    let parts: Vec<&str> = time.split(':').collect();
    match parts.len() {
        3 => {
            let hours: u32 = parts[0].parse().ok()?;
            let minutes: u32 = parts[1].parse().ok()?;
            let seconds: u32 = parts[2].parse().ok()?;
            Some(hours * 3600 + minutes * 60 + seconds)
        }
        2 => {
            let hours: u32 = parts[0].parse().ok()?;
            let minutes: u32 = parts[1].parse().ok()?;
            Some(hours * 3600 + minutes * 60)
        }
        _ => None,
    }
}

/// Convert seconds since midnight to HH:MM:SS format.
pub fn seconds_to_time_string(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_hhmmss() {
        assert_eq!(parse_time_to_seconds("14:30:00"), Some(52200));
        assert_eq!(parse_time_to_seconds("00:00:00"), Some(0));
        assert_eq!(parse_time_to_seconds("25:00:00"), Some(90000)); // Next day
    }

    #[test]
    fn test_parse_time_hhmm() {
        assert_eq!(parse_time_to_seconds("14:30"), Some(52200));
        assert_eq!(parse_time_to_seconds("00:00"), Some(0));
    }

    #[test]
    fn test_parse_time_seconds() {
        assert_eq!(parse_time_to_seconds("52200"), Some(52200));
        assert_eq!(parse_time_to_seconds("0"), Some(0));
    }

    #[test]
    fn test_seconds_to_time_string() {
        assert_eq!(seconds_to_time_string(52200), "14:30:00");
        assert_eq!(seconds_to_time_string(0), "00:00:00");
        assert_eq!(seconds_to_time_string(90000), "25:00:00");
    }

    #[test]
    fn test_schedule_row_duration() {
        let row = ScheduleRow {
            start_time: Some("08:00:00".to_string()),
            end_time: Some("08:30:00".to_string()),
            ..Default::default()
        };
        assert_eq!(row.duration_seconds(), Some(1800));
    }
}

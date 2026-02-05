//! Deadhead model - non-revenue vehicle movements.

use serde::{Deserialize, Serialize};

/// Type of deadhead movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeadheadType {
    /// Pull-out from depot to first revenue stop.
    PullOut,
    /// Pull-in from last revenue stop to depot.
    PullIn,
    /// Interlining - deadhead between revenue trips (not at depot).
    Interlining,
}

/// A deadhead movement - non-revenue vehicle repositioning.
///
/// Deadheads represent vehicle movements that don't carry passengers:
/// - **Pull-out**: Vehicle travels from depot to first passenger stop
/// - **Pull-in**: Vehicle travels from last passenger stop back to depot
/// - **Interlining**: Vehicle travels between trips (e.g., end of line A to start of line B)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deadhead {
    /// Type of deadhead movement.
    pub deadhead_type: DeadheadType,

    /// Starting location (stop_id, depot code, or coordinates).
    pub from_location: String,

    /// Ending location (stop_id, depot code, or coordinates).
    pub to_location: String,

    /// Start time in seconds since midnight.
    pub start_time_seconds: Option<u32>,

    /// End time in seconds since midnight.
    pub end_time_seconds: Option<u32>,

    /// Block this deadhead belongs to.
    pub block_id: Option<String>,

    /// Trip ID before this deadhead (for interlining).
    pub from_trip_id: Option<String>,

    /// Trip ID after this deadhead (for interlining).
    pub to_trip_id: Option<String>,

    /// Estimated distance in meters.
    pub distance_meters: Option<f64>,

    /// Starting latitude.
    pub from_lat: Option<f64>,

    /// Starting longitude.
    pub from_lon: Option<f64>,

    /// Ending latitude.
    pub to_lat: Option<f64>,

    /// Ending longitude.
    pub to_lon: Option<f64>,

    /// Whether this deadhead was inferred (vs explicit in schedule).
    pub is_inferred: bool,
}

impl Deadhead {
    /// Create a new pull-out deadhead.
    pub fn pull_out(depot: impl Into<String>, first_stop: impl Into<String>) -> Self {
        Self {
            deadhead_type: DeadheadType::PullOut,
            from_location: depot.into(),
            to_location: first_stop.into(),
            start_time_seconds: None,
            end_time_seconds: None,
            block_id: None,
            from_trip_id: None,
            to_trip_id: None,
            distance_meters: None,
            from_lat: None,
            from_lon: None,
            to_lat: None,
            to_lon: None,
            is_inferred: false,
        }
    }

    /// Create a new pull-in deadhead.
    pub fn pull_in(last_stop: impl Into<String>, depot: impl Into<String>) -> Self {
        Self {
            deadhead_type: DeadheadType::PullIn,
            from_location: last_stop.into(),
            to_location: depot.into(),
            start_time_seconds: None,
            end_time_seconds: None,
            block_id: None,
            from_trip_id: None,
            to_trip_id: None,
            distance_meters: None,
            from_lat: None,
            from_lon: None,
            to_lat: None,
            to_lon: None,
            is_inferred: false,
        }
    }

    /// Create an interlining deadhead.
    pub fn interlining(
        from_stop: impl Into<String>,
        to_stop: impl Into<String>,
    ) -> Self {
        Self {
            deadhead_type: DeadheadType::Interlining,
            from_location: from_stop.into(),
            to_location: to_stop.into(),
            start_time_seconds: None,
            end_time_seconds: None,
            block_id: None,
            from_trip_id: None,
            to_trip_id: None,
            distance_meters: None,
            from_lat: None,
            from_lon: None,
            to_lat: None,
            to_lon: None,
            is_inferred: false,
        }
    }

    /// Set the block ID.
    pub fn with_block(mut self, block_id: impl Into<String>) -> Self {
        self.block_id = Some(block_id.into());
        self
    }

    /// Set the times.
    pub fn with_times(mut self, start: u32, end: u32) -> Self {
        self.start_time_seconds = Some(start);
        self.end_time_seconds = Some(end);
        self
    }

    /// Set connecting trip IDs.
    pub fn with_trips(
        mut self,
        from_trip: Option<impl Into<String>>,
        to_trip: Option<impl Into<String>>,
    ) -> Self {
        self.from_trip_id = from_trip.map(|s| s.into());
        self.to_trip_id = to_trip.map(|s| s.into());
        self
    }

    /// Set coordinates.
    pub fn with_coordinates(
        mut self,
        from_lat: f64,
        from_lon: f64,
        to_lat: f64,
        to_lon: f64,
    ) -> Self {
        self.from_lat = Some(from_lat);
        self.from_lon = Some(from_lon);
        self.to_lat = Some(to_lat);
        self.to_lon = Some(to_lon);
        self
    }

    /// Mark as inferred (not explicitly in schedule).
    pub fn inferred(mut self) -> Self {
        self.is_inferred = true;
        self
    }

    /// Duration in seconds.
    pub fn duration_seconds(&self) -> Option<u32> {
        match (self.start_time_seconds, self.end_time_seconds) {
            (Some(start), Some(end)) if end >= start => Some(end - start),
            _ => None,
        }
    }

    /// Calculate distance using Haversine formula if coordinates are available.
    pub fn calculate_distance(&self) -> Option<f64> {
        match (self.from_lat, self.from_lon, self.to_lat, self.to_lon) {
            (Some(lat1), Some(lon1), Some(lat2), Some(lon2)) => {
                Some(haversine_distance(lat1, lon1, lat2, lon2))
            }
            _ => None,
        }
    }

    /// Check if this is a depot movement (pull-out or pull-in).
    pub fn is_depot_movement(&self) -> bool {
        matches!(
            self.deadhead_type,
            DeadheadType::PullOut | DeadheadType::PullIn
        )
    }
}

/// Calculate Haversine distance between two coordinates in meters.
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_M: f64 = 6_371_000.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS_M * c
}

/// Result of deadhead inference for a schedule.
#[derive(Debug, Clone, Default)]
pub struct DeadheadInferenceResult {
    /// Inferred pull-out deadheads.
    pub pull_outs: Vec<Deadhead>,
    /// Inferred pull-in deadheads.
    pub pull_ins: Vec<Deadhead>,
    /// Inferred interlining deadheads.
    pub interlinings: Vec<Deadhead>,
    /// Blocks that couldn't have deadheads inferred (missing info).
    pub incomplete_blocks: Vec<String>,
}

impl DeadheadInferenceResult {
    /// Total number of inferred deadheads.
    pub fn total_count(&self) -> usize {
        self.pull_outs.len() + self.pull_ins.len() + self.interlinings.len()
    }

    /// Get all deadheads as a single iterator.
    pub fn all_deadheads(&self) -> impl Iterator<Item = &Deadhead> {
        self.pull_outs
            .iter()
            .chain(self.pull_ins.iter())
            .chain(self.interlinings.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pull_out_creation() {
        let dh = Deadhead::pull_out("DEPOT1", "STOP_A")
            .with_block("B1")
            .with_times(21600, 22200);

        assert_eq!(dh.deadhead_type, DeadheadType::PullOut);
        assert_eq!(dh.from_location, "DEPOT1");
        assert_eq!(dh.to_location, "STOP_A");
        assert_eq!(dh.duration_seconds(), Some(600)); // 10 minutes
        assert!(dh.is_depot_movement());
    }

    #[test]
    fn test_interlining() {
        let dh = Deadhead::interlining("STOP_B", "STOP_C")
            .with_trips(Some("TRIP1"), Some("TRIP2"));

        assert_eq!(dh.deadhead_type, DeadheadType::Interlining);
        assert!(!dh.is_depot_movement());
        assert_eq!(dh.from_trip_id, Some("TRIP1".to_string()));
        assert_eq!(dh.to_trip_id, Some("TRIP2".to_string()));
    }

    #[test]
    fn test_haversine_distance() {
        // NYC to LA is approximately 3,940 km
        let distance = haversine_distance(40.7128, -74.0060, 34.0522, -118.2437);
        assert!((distance - 3_940_000.0).abs() < 100_000.0); // Within 100km
    }

    #[test]
    fn test_calculate_distance_with_coords() {
        let dh = Deadhead::pull_out("DEPOT", "STOP")
            .with_coordinates(40.7128, -74.0060, 40.7580, -73.9855); // ~5km in NYC

        let dist = dh.calculate_distance();
        assert!(dist.is_some());
        assert!(dist.unwrap() > 4000.0 && dist.unwrap() < 6000.0);
    }
}

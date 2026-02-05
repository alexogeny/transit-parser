//! Deadhead inference from schedule and GTFS data.

use crate::models::{Deadhead, DeadheadInferenceResult, Schedule};
use gtfs_parser::GtfsFeed;
use std::collections::HashMap;

/// Configuration for deadhead inference.
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// Depot locations (stop_id -> depot_code).
    pub depot_locations: HashMap<String, String>,

    /// Default depot to use if none specified.
    pub default_depot: Option<String>,

    /// Average deadhead speed in meters per second (for time estimation).
    pub average_speed_mps: f64,

    /// Minimum time gap to infer a deadhead (seconds).
    pub min_gap_seconds: u32,

    /// Whether to infer interlining deadheads.
    pub infer_interlining: bool,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            depot_locations: HashMap::new(),
            default_depot: None,
            average_speed_mps: 8.33, // ~30 km/h
            min_gap_seconds: 60,
            infer_interlining: true,
        }
    }
}

impl InferenceConfig {
    /// Create new config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a depot location.
    pub fn add_depot(mut self, stop_id: impl Into<String>, depot_code: impl Into<String>) -> Self {
        self.depot_locations
            .insert(stop_id.into(), depot_code.into());
        self
    }

    /// Set default depot.
    pub fn with_default_depot(mut self, depot: impl Into<String>) -> Self {
        self.default_depot = Some(depot.into());
        self
    }
}

/// Infers missing deadheads from schedule data.
pub struct DeadheadInferrer<'a> {
    config: InferenceConfig,
    #[allow(dead_code)]
    gtfs: Option<&'a GtfsFeed>,
    stop_coords: HashMap<String, (f64, f64)>,
}

impl<'a> DeadheadInferrer<'a> {
    /// Create a new inferrer without GTFS.
    pub fn new(config: InferenceConfig) -> Self {
        Self {
            config,
            gtfs: None,
            stop_coords: HashMap::new(),
        }
    }

    /// Create an inferrer with GTFS data for coordinate lookup.
    pub fn with_gtfs(config: InferenceConfig, gtfs: &'a GtfsFeed) -> Self {
        // Pre-compute stop coordinates
        let stop_coords: HashMap<String, (f64, f64)> = gtfs
            .feed
            .stops
            .iter()
            .map(|s| (s.id.clone(), (s.latitude, s.longitude)))
            .collect();

        Self {
            config,
            gtfs: Some(gtfs),
            stop_coords,
        }
    }

    /// Infer all missing deadheads for a schedule.
    pub fn infer(&self, schedule: &mut Schedule) -> DeadheadInferenceResult {
        let mut result = DeadheadInferenceResult::default();

        // Process each block
        for block_id in schedule.block_ids() {
            match self.infer_block_deadheads(schedule, &block_id) {
                Ok(block_result) => {
                    result.pull_outs.extend(block_result.pull_outs);
                    result.pull_ins.extend(block_result.pull_ins);
                    result.interlinings.extend(block_result.interlinings);
                }
                Err(_) => {
                    result.incomplete_blocks.push(block_id);
                }
            }
        }

        result
    }

    /// Infer deadheads for a single block.
    fn infer_block_deadheads(
        &self,
        schedule: &mut Schedule,
        block_id: &str,
    ) -> Result<DeadheadInferenceResult, &'static str> {
        let mut result = DeadheadInferenceResult::default();

        let block = schedule.get_block(block_id).ok_or("Block not found")?;
        let block = block.clone(); // Clone to avoid borrow issues

        // Get depot for this block
        let depot = block
            .depot
            .clone()
            .or_else(|| self.config.default_depot.clone())
            .ok_or("No depot available")?;

        // Find first revenue trip
        let first_trip = block.rows.iter().find(|r| r.is_revenue());
        if let Some(first) = first_trip {
            if let Some(start_place) = &first.start_place {
                // Check if pull-out already exists
                if block.pull_out().is_none() {
                    let mut pull_out = Deadhead::pull_out(depot.clone(), start_place)
                        .with_block(block_id)
                        .inferred();

                    // Add coordinates if available
                    if let Some(&(lat, lon)) = self.stop_coords.get(start_place) {
                        pull_out.to_lat = Some(lat);
                        pull_out.to_lon = Some(lon);
                    }

                    // Estimate time if first trip has a start time
                    if let Some(trip_start) = first.start_time_seconds() {
                        // Assume 15 minutes for pull-out by default
                        let pull_out_duration = self.estimate_duration(&depot, start_place);
                        pull_out.start_time_seconds =
                            Some(trip_start.saturating_sub(pull_out_duration));
                        pull_out.end_time_seconds = Some(trip_start);
                    }

                    result.pull_outs.push(pull_out);
                }
            }
        }

        // Find last revenue trip
        let last_trip = block.rows.iter().rev().find(|r| r.is_revenue());
        if let Some(last) = last_trip {
            if let Some(end_place) = &last.end_place {
                // Check if pull-in already exists
                if block.pull_in().is_none() {
                    let mut pull_in = Deadhead::pull_in(end_place, depot.clone())
                        .with_block(block_id)
                        .inferred();

                    // Add coordinates if available
                    if let Some(&(lat, lon)) = self.stop_coords.get(end_place) {
                        pull_in.from_lat = Some(lat);
                        pull_in.from_lon = Some(lon);
                    }

                    // Estimate time if last trip has an end time
                    if let Some(trip_end) = last.end_time_seconds() {
                        let pull_in_duration = self.estimate_duration(end_place, &depot);
                        pull_in.start_time_seconds = Some(trip_end);
                        pull_in.end_time_seconds = Some(trip_end + pull_in_duration);
                    }

                    result.pull_ins.push(pull_in);
                }
            }
        }

        // Infer interlining deadheads
        if self.config.infer_interlining {
            let revenue_trips: Vec<_> = block.rows.iter().filter(|r| r.is_revenue()).collect();

            for i in 0..revenue_trips.len().saturating_sub(1) {
                let prev = revenue_trips[i];
                let next = revenue_trips[i + 1];

                // Check for location discontinuity
                if let (Some(end_place), Some(start_place)) = (&prev.end_place, &next.start_place) {
                    if end_place != start_place {
                        // Check if there's a time gap
                        let needs_deadhead =
                            match (prev.end_time_seconds(), next.start_time_seconds()) {
                                (Some(end), Some(start)) => {
                                    start > end + self.config.min_gap_seconds
                                }
                                _ => true, // If no times, assume we need it
                            };

                        if needs_deadhead {
                            let mut interlining = Deadhead::interlining(end_place, start_place)
                                .with_block(block_id)
                                .with_trips(prev.trip_id.clone(), next.trip_id.clone())
                                .inferred();

                            // Add coordinates
                            if let Some(&(lat, lon)) = self.stop_coords.get(end_place) {
                                interlining.from_lat = Some(lat);
                                interlining.from_lon = Some(lon);
                            }
                            if let Some(&(lat, lon)) = self.stop_coords.get(start_place) {
                                interlining.to_lat = Some(lat);
                                interlining.to_lon = Some(lon);
                            }

                            // Set times
                            if let (Some(end), Some(start)) =
                                (prev.end_time_seconds(), next.start_time_seconds())
                            {
                                interlining.start_time_seconds = Some(end);
                                interlining.end_time_seconds = Some(start);
                            }

                            result.interlinings.push(interlining);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Estimate deadhead duration based on distance and average speed.
    fn estimate_duration(&self, from: &str, to: &str) -> u32 {
        // Try to calculate from coordinates
        if let (Some(&(lat1, lon1)), Some(&(lat2, lon2))) =
            (self.stop_coords.get(from), self.stop_coords.get(to))
        {
            let distance = haversine_distance(lat1, lon1, lat2, lon2);
            let time = distance / self.config.average_speed_mps;
            return time as u32;
        }

        // Default: 15 minutes
        900
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RowType, ScheduleRow};

    fn make_row(
        trip_id: &str,
        block: &str,
        start_place: &str,
        end_place: &str,
        start: &str,
        end: &str,
    ) -> ScheduleRow {
        ScheduleRow {
            trip_id: Some(trip_id.to_string()),
            block: Some(block.to_string()),
            start_place: Some(start_place.to_string()),
            end_place: Some(end_place.to_string()),
            start_time: Some(start.to_string()),
            end_time: Some(end.to_string()),
            row_type: RowType::Revenue,
            ..Default::default()
        }
    }

    #[test]
    fn test_infer_pull_out_and_in() {
        let config = InferenceConfig::new().with_default_depot("DEPOT");
        let inferrer = DeadheadInferrer::new(config);

        let mut schedule = Schedule::from_rows(vec![
            make_row("T1", "B1", "STOP_A", "STOP_B", "08:00:00", "09:00:00"),
            make_row("T2", "B1", "STOP_B", "STOP_C", "09:15:00", "10:00:00"),
        ]);

        let result = inferrer.infer(&mut schedule);

        assert_eq!(result.pull_outs.len(), 1);
        assert_eq!(result.pull_ins.len(), 1);

        let pull_out = &result.pull_outs[0];
        assert_eq!(pull_out.from_location, "DEPOT");
        assert_eq!(pull_out.to_location, "STOP_A");
        assert!(pull_out.is_inferred);

        let pull_in = &result.pull_ins[0];
        assert_eq!(pull_in.from_location, "STOP_C");
        assert_eq!(pull_in.to_location, "DEPOT");
    }

    #[test]
    fn test_infer_interlining() {
        let config = InferenceConfig::new().with_default_depot("DEPOT");
        let inferrer = DeadheadInferrer::new(config);

        let mut schedule = Schedule::from_rows(vec![
            make_row("T1", "B1", "A", "B", "08:00:00", "09:00:00"),
            // Location discontinuity: B -> C
            make_row("T2", "B1", "C", "D", "09:15:00", "10:00:00"),
        ]);

        let result = inferrer.infer(&mut schedule);

        assert_eq!(result.interlinings.len(), 1);

        let interlining = &result.interlinings[0];
        assert_eq!(interlining.from_location, "B");
        assert_eq!(interlining.to_location, "C");
        assert_eq!(interlining.from_trip_id, Some("T1".to_string()));
        assert_eq!(interlining.to_trip_id, Some("T2".to_string()));
    }

    #[test]
    fn test_no_interlining_when_continuous() {
        let config = InferenceConfig::new().with_default_depot("DEPOT");
        let inferrer = DeadheadInferrer::new(config);

        let mut schedule = Schedule::from_rows(vec![
            make_row("T1", "B1", "A", "B", "08:00:00", "09:00:00"),
            make_row("T2", "B1", "B", "C", "09:15:00", "10:00:00"), // Continuous
        ]);

        let result = inferrer.infer(&mut schedule);

        assert_eq!(result.interlinings.len(), 0);
    }

    #[test]
    fn test_incomplete_block_no_depot() {
        let config = InferenceConfig::new(); // No default depot
        let inferrer = DeadheadInferrer::new(config);

        let mut schedule =
            Schedule::from_rows(vec![make_row("T1", "B1", "A", "B", "08:00:00", "09:00:00")]);

        let result = inferrer.infer(&mut schedule);

        assert!(result.incomplete_blocks.contains(&"B1".to_string()));
    }
}

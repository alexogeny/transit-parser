//! GTFS referential integrity validation.

use crate::models::{Schedule, ScheduleRow};
use crate::validation::config::{GtfsComplianceLevel, ValidationConfig};
use gtfs_parser::GtfsFeed;
use std::collections::HashSet;

/// Error from GTFS integrity validation.
#[derive(Debug, Clone)]
pub struct GtfsIntegrityError {
    pub error_type: GtfsIntegrityErrorType,
    pub row_index: usize,
    pub field: String,
    pub value: String,
    pub message: String,
}

/// Types of GTFS integrity errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GtfsIntegrityErrorType {
    /// Referenced trip_id doesn't exist in GTFS.
    MissingTripId,
    /// Referenced stop doesn't exist in GTFS.
    MissingStopId,
    /// Referenced shape doesn't exist in GTFS.
    MissingShapeId,
    /// Times don't match GTFS stop_times.
    TimeInconsistency,
}

/// Warning from GTFS integrity check.
#[derive(Debug, Clone)]
pub struct GtfsIntegrityWarning {
    pub code: String,
    pub row_index: usize,
    pub message: String,
}

/// Result of GTFS integrity checking.
#[derive(Debug, Clone, Default)]
pub struct GtfsIntegrityResult {
    pub errors: Vec<GtfsIntegrityError>,
    pub warnings: Vec<GtfsIntegrityWarning>,
}

impl GtfsIntegrityResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Checks GTFS referential integrity.
pub struct GtfsIntegrityChecker<'a> {
    #[allow(dead_code)]
    gtfs: &'a GtfsFeed,
    config: &'a ValidationConfig,
    trip_ids: HashSet<String>,
    stop_ids: HashSet<String>,
    shape_ids: HashSet<String>,
}

impl<'a> GtfsIntegrityChecker<'a> {
    /// Create a new integrity checker.
    pub fn new(gtfs: &'a GtfsFeed, config: &'a ValidationConfig) -> Self {
        // Pre-compute ID sets for fast lookup
        let trip_ids: HashSet<String> = gtfs
            .feed
            .trips
            .iter()
            .map(|t| t.id.clone())
            .collect();

        let stop_ids: HashSet<String> = gtfs
            .feed
            .stops
            .iter()
            .map(|s| s.id.clone())
            .collect();

        let shape_ids: HashSet<String> = gtfs
            .feed
            .shapes
            .iter()
            .map(|s| s.id.clone())
            .collect();

        Self {
            gtfs,
            config,
            trip_ids,
            stop_ids,
            shape_ids,
        }
    }

    /// Check a single schedule row.
    pub fn check_row(&self, row: &ScheduleRow, row_index: usize) -> GtfsIntegrityResult {
        let mut result = GtfsIntegrityResult::default();

        // Check trip_id
        if let Some(ref trip_id) = row.trip_id {
            if !self.trip_ids.contains(trip_id) {
                match self.config.gtfs_compliance {
                    GtfsComplianceLevel::Strict => {
                        result.errors.push(GtfsIntegrityError {
                            error_type: GtfsIntegrityErrorType::MissingTripId,
                            row_index,
                            field: "trip_id".to_string(),
                            value: trip_id.clone(),
                            message: format!("Trip ID '{}' not found in GTFS trips.txt", trip_id),
                        });
                    }
                    GtfsComplianceLevel::Standard => {
                        result.warnings.push(GtfsIntegrityWarning {
                            code: "W001".to_string(),
                            row_index,
                            message: format!("Trip ID '{}' not found in GTFS", trip_id),
                        });
                    }
                    GtfsComplianceLevel::Lenient => {
                        // Ignore
                    }
                }
            }
        }

        // Check start_place as stop_id
        if let Some(ref start_place) = row.start_place {
            if !self.stop_ids.contains(start_place) && row.is_revenue() {
                self.add_stop_warning_or_error(&mut result, row_index, "start_place", start_place);
            }
        }

        // Check end_place as stop_id
        if let Some(ref end_place) = row.end_place {
            if !self.stop_ids.contains(end_place) && row.is_revenue() {
                self.add_stop_warning_or_error(&mut result, row_index, "end_place", end_place);
            }
        }

        // Check route_shape_id
        if let Some(ref shape_id) = row.route_shape_id {
            if !self.shape_ids.contains(shape_id) {
                match self.config.gtfs_compliance {
                    GtfsComplianceLevel::Strict => {
                        result.errors.push(GtfsIntegrityError {
                            error_type: GtfsIntegrityErrorType::MissingShapeId,
                            row_index,
                            field: "route_shape_id".to_string(),
                            value: shape_id.clone(),
                            message: format!("Shape ID '{}' not found in GTFS shapes.txt", shape_id),
                        });
                    }
                    _ => {
                        result.warnings.push(GtfsIntegrityWarning {
                            code: "W003".to_string(),
                            row_index,
                            message: format!("Shape ID '{}' not found in GTFS", shape_id),
                        });
                    }
                }
            }
        }

        result
    }

    fn add_stop_warning_or_error(
        &self,
        result: &mut GtfsIntegrityResult,
        row_index: usize,
        field: &str,
        value: &str,
    ) {
        match self.config.gtfs_compliance {
            GtfsComplianceLevel::Strict => {
                result.errors.push(GtfsIntegrityError {
                    error_type: GtfsIntegrityErrorType::MissingStopId,
                    row_index,
                    field: field.to_string(),
                    value: value.to_string(),
                    message: format!("Stop ID '{}' not found in GTFS stops.txt", value),
                });
            }
            GtfsComplianceLevel::Standard => {
                result.warnings.push(GtfsIntegrityWarning {
                    code: "W002".to_string(),
                    row_index,
                    message: format!("Stop ID '{}' ({}) not found in GTFS", value, field),
                });
            }
            GtfsComplianceLevel::Lenient => {}
        }
    }

    /// Check the entire schedule.
    pub fn check_schedule(&self, schedule: &Schedule) -> GtfsIntegrityResult {
        let mut combined = GtfsIntegrityResult::default();

        for (idx, row) in schedule.rows.iter().enumerate() {
            let row_result = self.check_row(row, idx);
            combined.errors.extend(row_result.errors);
            combined.warnings.extend(row_result.warnings);

            // Check max errors limit
            if let Some(max) = self.config.max_errors {
                if combined.errors.len() >= max {
                    break;
                }
            }
        }

        combined
    }

    /// Get summary of missing references.
    pub fn get_missing_references(&self, schedule: &Schedule) -> MissingReferences {
        let mut missing = MissingReferences::default();

        for row in &schedule.rows {
            if let Some(ref trip_id) = row.trip_id {
                if !self.trip_ids.contains(trip_id) {
                    missing.trip_ids.insert(trip_id.clone());
                }
            }
            if let Some(ref start) = row.start_place {
                if !self.stop_ids.contains(start) && row.is_revenue() {
                    missing.stop_ids.insert(start.clone());
                }
            }
            if let Some(ref end) = row.end_place {
                if !self.stop_ids.contains(end) && row.is_revenue() {
                    missing.stop_ids.insert(end.clone());
                }
            }
            if let Some(ref shape_id) = row.route_shape_id {
                if !self.shape_ids.contains(shape_id) {
                    missing.shape_ids.insert(shape_id.clone());
                }
            }
        }

        missing
    }
}

/// Summary of missing references.
#[derive(Debug, Clone, Default)]
pub struct MissingReferences {
    pub trip_ids: HashSet<String>,
    pub stop_ids: HashSet<String>,
    pub shape_ids: HashSet<String>,
}

impl MissingReferences {
    pub fn is_empty(&self) -> bool {
        self.trip_ids.is_empty() && self.stop_ids.is_empty() && self.shape_ids.is_empty()
    }

    pub fn total_count(&self) -> usize {
        self.trip_ids.len() + self.stop_ids.len() + self.shape_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RowType;

    fn make_gtfs_with_trip(trip_id: &str, stop_id: &str) -> GtfsFeed {
        use transit_core::{Stop, Trip};

        let mut feed = GtfsFeed::new();
        feed.feed.trips.push(Trip::new(trip_id, "R1", "S1"));
        feed.feed.stops.push(Stop::new(stop_id, "Test Stop", 0.0, 0.0));
        feed
    }

    #[test]
    fn test_valid_trip_reference() {
        let gtfs = make_gtfs_with_trip("TRIP1", "STOP1");
        let config = ValidationConfig::strict();
        let checker = GtfsIntegrityChecker::new(&gtfs, &config);

        let row = ScheduleRow {
            trip_id: Some("TRIP1".to_string()),
            start_place: Some("STOP1".to_string()),
            row_type: RowType::Revenue,
            ..Default::default()
        };

        let result = checker.check_row(&row, 0);
        assert!(result.is_valid());
    }

    #[test]
    fn test_missing_trip_strict() {
        let gtfs = make_gtfs_with_trip("TRIP1", "STOP1");
        let config = ValidationConfig::strict();
        let checker = GtfsIntegrityChecker::new(&gtfs, &config);

        let row = ScheduleRow {
            trip_id: Some("MISSING_TRIP".to_string()),
            row_type: RowType::Revenue,
            ..Default::default()
        };

        let result = checker.check_row(&row, 0);
        assert!(!result.is_valid());
        assert_eq!(result.errors[0].error_type, GtfsIntegrityErrorType::MissingTripId);
    }

    #[test]
    fn test_missing_trip_standard() {
        let gtfs = make_gtfs_with_trip("TRIP1", "STOP1");
        let config = ValidationConfig::new(); // Standard level
        let checker = GtfsIntegrityChecker::new(&gtfs, &config);

        let row = ScheduleRow {
            trip_id: Some("MISSING_TRIP".to_string()),
            row_type: RowType::Revenue,
            ..Default::default()
        };

        let result = checker.check_row(&row, 0);
        assert!(result.is_valid()); // No error, just warning
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_missing_trip_lenient() {
        let gtfs = make_gtfs_with_trip("TRIP1", "STOP1");
        let config = ValidationConfig::lenient();
        let checker = GtfsIntegrityChecker::new(&gtfs, &config);

        let row = ScheduleRow {
            trip_id: Some("MISSING_TRIP".to_string()),
            row_type: RowType::Revenue,
            ..Default::default()
        };

        let result = checker.check_row(&row, 0);
        assert!(result.is_valid());
        assert!(result.warnings.is_empty()); // No warnings either
    }
}

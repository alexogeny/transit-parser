//! Validation configuration and business rules.

use serde::{Deserialize, Serialize};

/// GTFS compliance level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GtfsComplianceLevel {
    /// Strict: All trip_ids, stop_ids must exist in GTFS.
    Strict,
    /// Standard: Most references should exist, allows some missing.
    #[default]
    Standard,
    /// Lenient: Only check what's provided, ignore missing references.
    Lenient,
}

/// Business rules for schedule validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRules {
    /// Minimum layover time between trips in seconds (default: 300 = 5 min).
    pub min_layover_seconds: u32,

    /// Maximum single trip duration in seconds (default: 14400 = 4 hours).
    pub max_trip_duration_seconds: u32,

    /// Maximum duty length in seconds (default: 32400 = 9 hours).
    pub max_duty_length_seconds: u32,

    /// Maximum continuous driving before required break (default: 16200 = 4.5 hours).
    pub max_continuous_driving_seconds: u32,

    /// Minimum break duration in seconds (default: 1800 = 30 min).
    pub min_break_duration_seconds: u32,

    /// Allowed deviation from GTFS times in seconds (default: 60 = 1 min).
    pub time_tolerance_seconds: u32,

    /// Minimum block duration in seconds (default: 0 = no minimum).
    pub min_block_duration_seconds: u32,

    /// Maximum block duration in seconds (default: 57600 = 16 hours).
    pub max_block_duration_seconds: u32,

    /// Flag orphan trips (trips not assigned to any block).
    pub flag_orphan_trips: bool,

    /// Flag missing coordinates.
    pub flag_missing_coordinates: bool,

    /// Flag unusual headways on same route (deviation from mean).
    pub headway_deviation_threshold: Option<f64>,
}

impl Default for BusinessRules {
    fn default() -> Self {
        Self {
            min_layover_seconds: 300,              // 5 minutes
            max_trip_duration_seconds: 14400,      // 4 hours
            max_duty_length_seconds: 32400,        // 9 hours
            max_continuous_driving_seconds: 16200, // 4.5 hours
            min_break_duration_seconds: 1800,      // 30 minutes
            time_tolerance_seconds: 60,            // 1 minute
            min_block_duration_seconds: 0,
            max_block_duration_seconds: 57600, // 16 hours
            flag_orphan_trips: true,
            flag_missing_coordinates: false,
            headway_deviation_threshold: Some(2.0), // 2x standard deviation
        }
    }
}

impl BusinessRules {
    /// Create strict business rules (tighter constraints).
    pub fn strict() -> Self {
        Self {
            min_layover_seconds: 600,              // 10 minutes
            max_trip_duration_seconds: 10800,      // 3 hours
            max_duty_length_seconds: 28800,        // 8 hours
            max_continuous_driving_seconds: 14400, // 4 hours
            min_break_duration_seconds: 2700,      // 45 minutes
            time_tolerance_seconds: 30,            // 30 seconds
            min_block_duration_seconds: 3600,      // 1 hour
            max_block_duration_seconds: 43200,     // 12 hours
            flag_orphan_trips: true,
            flag_missing_coordinates: true,
            headway_deviation_threshold: Some(1.5),
        }
    }

    /// Create lenient business rules (relaxed constraints).
    pub fn lenient() -> Self {
        Self {
            min_layover_seconds: 60,               // 1 minute
            max_trip_duration_seconds: 21600,      // 6 hours
            max_duty_length_seconds: 43200,        // 12 hours
            max_continuous_driving_seconds: 21600, // 6 hours
            min_break_duration_seconds: 900,       // 15 minutes
            time_tolerance_seconds: 300,           // 5 minutes
            min_block_duration_seconds: 0,
            max_block_duration_seconds: 86400, // 24 hours
            flag_orphan_trips: false,
            flag_missing_coordinates: false,
            headway_deviation_threshold: None,
        }
    }
}

/// Complete validation configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// GTFS compliance level.
    pub gtfs_compliance: GtfsComplianceLevel,

    /// Business rules.
    pub business_rules: BusinessRules,

    /// Whether to validate block continuity.
    pub validate_block_continuity: bool,

    /// Whether to validate duty constraints.
    pub validate_duty_constraints: bool,

    /// Whether to generate warnings (in addition to errors).
    pub generate_warnings: bool,

    /// Maximum number of errors to collect before stopping.
    pub max_errors: Option<usize>,
}

impl ValidationConfig {
    /// Create a new config with default settings.
    pub fn new() -> Self {
        Self {
            gtfs_compliance: GtfsComplianceLevel::Standard,
            business_rules: BusinessRules::default(),
            validate_block_continuity: true,
            validate_duty_constraints: true,
            generate_warnings: true,
            max_errors: None,
        }
    }

    /// Create a strict configuration.
    pub fn strict() -> Self {
        Self {
            gtfs_compliance: GtfsComplianceLevel::Strict,
            business_rules: BusinessRules::strict(),
            validate_block_continuity: true,
            validate_duty_constraints: true,
            generate_warnings: true,
            max_errors: None,
        }
    }

    /// Create a lenient configuration.
    pub fn lenient() -> Self {
        Self {
            gtfs_compliance: GtfsComplianceLevel::Lenient,
            business_rules: BusinessRules::lenient(),
            validate_block_continuity: false,
            validate_duty_constraints: false,
            generate_warnings: false,
            max_errors: None,
        }
    }

    /// Set GTFS compliance level.
    pub fn with_gtfs_compliance(mut self, level: GtfsComplianceLevel) -> Self {
        self.gtfs_compliance = level;
        self
    }

    /// Set business rules.
    pub fn with_business_rules(mut self, rules: BusinessRules) -> Self {
        self.business_rules = rules;
        self
    }

    /// Set max errors.
    pub fn with_max_errors(mut self, max: usize) -> Self {
        self.max_errors = Some(max);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ValidationConfig::new();
        assert_eq!(config.gtfs_compliance, GtfsComplianceLevel::Standard);
        assert_eq!(config.business_rules.min_layover_seconds, 300);
        assert!(config.validate_block_continuity);
    }

    #[test]
    fn test_strict_rules() {
        let rules = BusinessRules::strict();
        assert!(rules.min_layover_seconds > BusinessRules::default().min_layover_seconds);
        assert!(rules.max_duty_length_seconds < BusinessRules::default().max_duty_length_seconds);
    }

    #[test]
    fn test_lenient_rules() {
        let rules = BusinessRules::lenient();
        assert!(rules.min_layover_seconds < BusinessRules::default().min_layover_seconds);
        assert!(rules.max_duty_length_seconds > BusinessRules::default().max_duty_length_seconds);
    }
}

//! Main schedule validator.

use crate::models::Schedule;
use crate::validation::config::ValidationConfig;
use crate::validation::rules::{
    block_continuity::{BlockContinuityChecker, BlockContinuityError, BlockContinuityWarning},
    business_rules::{BusinessRuleChecker, BusinessRuleError, BusinessRuleWarning},
    gtfs_integrity::GtfsIntegrityChecker,
};
use gtfs_parser::GtfsFeed;
use serde::{Deserialize, Serialize};

/// A validation error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code (e.g., "E001", "E101").
    pub code: String,
    /// Error category.
    pub category: ErrorCategory,
    /// Human-readable message.
    pub message: String,
    /// Context (row index, block_id, duty_id, etc.).
    pub context: Option<String>,
}

/// Error category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// GTFS referential integrity error.
    GtfsIntegrity,
    /// Block continuity error.
    BlockContinuity,
    /// Business rule violation.
    BusinessRule,
}

/// A validation warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning code (e.g., "W001", "W101").
    pub code: String,
    /// Warning category.
    pub category: WarningCategory,
    /// Human-readable message.
    pub message: String,
    /// Context (row index, block_id, duty_id, etc.).
    pub context: Option<String>,
}

/// Warning category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningCategory {
    /// GTFS reference not found (lenient mode).
    GtfsReference,
    /// Block structure issue.
    BlockStructure,
    /// Business rule best practice.
    BestPractice,
}

/// Result of schedule validation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Validation errors (validation fails if any present).
    pub errors: Vec<ValidationError>,
    /// Validation warnings (informational).
    pub warnings: Vec<ValidationWarning>,
    /// Number of rows validated.
    pub rows_validated: usize,
    /// Number of blocks validated.
    pub blocks_validated: usize,
    /// Number of duties validated.
    pub duties_validated: usize,
    /// Whether validation was truncated due to max_errors.
    pub truncated: bool,
}

impl ValidationResult {
    /// Check if validation passed (no errors).
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get error count.
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get warning count.
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Get errors by category.
    pub fn errors_by_category(&self, category: ErrorCategory) -> Vec<&ValidationError> {
        self.errors.iter().filter(|e| e.category == category).collect()
    }

    /// Get warnings by category.
    pub fn warnings_by_category(&self, category: WarningCategory) -> Vec<&ValidationWarning> {
        self.warnings.iter().filter(|w| w.category == category).collect()
    }
}

/// Schedule validator.
pub struct Validator {
    config: ValidationConfig,
}

impl Validator {
    /// Create a new validator with the given configuration.
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Create a validator with default configuration.
    pub fn default_config() -> Self {
        Self::new(ValidationConfig::new())
    }

    /// Validate a schedule against a GTFS feed.
    pub fn validate(&self, schedule: &mut Schedule, gtfs: &GtfsFeed) -> ValidationResult {
        let mut result = ValidationResult {
            rows_validated: schedule.len(),
            ..Default::default()
        };

        // GTFS integrity checks
        let gtfs_checker = GtfsIntegrityChecker::new(gtfs, &self.config);
        let gtfs_result = gtfs_checker.check_schedule(schedule);

        for err in gtfs_result.errors {
            result.errors.push(ValidationError {
                code: format!("E{:03}", result.errors.len() + 1),
                category: ErrorCategory::GtfsIntegrity,
                message: err.message,
                context: Some(format!("row {}, field: {}", err.row_index, err.field)),
            });

            if self.check_truncation(&result) {
                return result;
            }
        }

        if self.config.generate_warnings {
            for warn in gtfs_result.warnings {
                result.warnings.push(ValidationWarning {
                    code: warn.code,
                    category: WarningCategory::GtfsReference,
                    message: warn.message,
                    context: Some(format!("row {}", warn.row_index)),
                });
            }
        }

        // Block continuity checks
        if self.config.validate_block_continuity {
            let block_checker = BlockContinuityChecker::new(&self.config);
            let block_result = block_checker.check_schedule(schedule);
            result.blocks_validated = schedule.block_ids().len();

            for err in block_result.errors {
                result.errors.push(self.convert_block_error(err));

                if self.check_truncation(&result) {
                    return result;
                }
            }

            if self.config.generate_warnings {
                for warn in block_result.warnings {
                    result.warnings.push(self.convert_block_warning(warn));
                }
            }
        }

        // Business rules checks
        let business_checker = BusinessRuleChecker::new(&self.config);
        let business_result = business_checker.check_schedule(schedule);

        for err in business_result.errors {
            result.errors.push(self.convert_business_error(err));

            if self.check_truncation(&result) {
                return result;
            }
        }

        if self.config.generate_warnings {
            for warn in business_result.warnings {
                result.warnings.push(self.convert_business_warning(warn));
            }
        }

        // Duty validation
        if self.config.validate_duty_constraints {
            let duty_result = business_checker.check_duties(schedule);
            result.duties_validated = schedule.run_numbers().len();

            for err in duty_result.errors {
                result.errors.push(self.convert_business_error(err));

                if self.check_truncation(&result) {
                    return result;
                }
            }

            if self.config.generate_warnings {
                for warn in duty_result.warnings {
                    result.warnings.push(self.convert_business_warning(warn));
                }
            }
        }

        result
    }

    /// Validate a schedule without GTFS (only structural/business rules).
    pub fn validate_structure(&self, schedule: &mut Schedule) -> ValidationResult {
        let mut result = ValidationResult {
            rows_validated: schedule.len(),
            ..Default::default()
        };

        // Block continuity checks
        if self.config.validate_block_continuity {
            let block_checker = BlockContinuityChecker::new(&self.config);
            let block_result = block_checker.check_schedule(schedule);
            result.blocks_validated = schedule.block_ids().len();

            for err in block_result.errors {
                result.errors.push(self.convert_block_error(err));

                if self.check_truncation(&result) {
                    return result;
                }
            }

            if self.config.generate_warnings {
                for warn in block_result.warnings {
                    result.warnings.push(self.convert_block_warning(warn));
                }
            }
        }

        // Business rules checks
        let business_checker = BusinessRuleChecker::new(&self.config);
        let business_result = business_checker.check_schedule(schedule);

        for err in business_result.errors {
            result.errors.push(self.convert_business_error(err));

            if self.check_truncation(&result) {
                return result;
            }
        }

        if self.config.generate_warnings {
            for warn in business_result.warnings {
                result.warnings.push(self.convert_business_warning(warn));
            }
        }

        // Duty validation
        if self.config.validate_duty_constraints {
            let duty_result = business_checker.check_duties(schedule);
            result.duties_validated = schedule.run_numbers().len();

            for err in duty_result.errors {
                result.errors.push(self.convert_business_error(err));
            }

            if self.config.generate_warnings {
                for warn in duty_result.warnings {
                    result.warnings.push(self.convert_business_warning(warn));
                }
            }
        }

        result
    }

    fn check_truncation(&self, result: &ValidationResult) -> bool {
        if let Some(max) = self.config.max_errors {
            if result.errors.len() >= max {
                return true;
            }
        }
        false
    }

    fn convert_block_error(&self, err: BlockContinuityError) -> ValidationError {
        ValidationError {
            code: format!("E1{:02}", 0),
            category: ErrorCategory::BlockContinuity,
            message: err.message,
            context: Some(format!(
                "block: {}{}",
                err.block_id,
                err.row_index.map(|i| format!(", row: {}", i)).unwrap_or_default()
            )),
        }
    }

    fn convert_block_warning(&self, warn: BlockContinuityWarning) -> ValidationWarning {
        ValidationWarning {
            code: warn.code,
            category: WarningCategory::BlockStructure,
            message: warn.message,
            context: Some(format!(
                "block: {}{}",
                warn.block_id,
                warn.row_index.map(|i| format!(", row: {}", i)).unwrap_or_default()
            )),
        }
    }

    fn convert_business_error(&self, err: BusinessRuleError) -> ValidationError {
        ValidationError {
            code: format!("E2{:02}", 0),
            category: ErrorCategory::BusinessRule,
            message: err.message,
            context: Some(err.context),
        }
    }

    fn convert_business_warning(&self, warn: BusinessRuleWarning) -> ValidationWarning {
        ValidationWarning {
            code: warn.code,
            category: WarningCategory::BestPractice,
            message: warn.message,
            context: Some(warn.context),
        }
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RowType, ScheduleRow};

    fn make_schedule(rows: Vec<ScheduleRow>) -> Schedule {
        Schedule::from_rows(rows)
    }

    fn make_row(trip_id: &str, block: &str, start: &str, end: &str) -> ScheduleRow {
        ScheduleRow {
            trip_id: Some(trip_id.to_string()),
            block: Some(block.to_string()),
            start_time: Some(start.to_string()),
            end_time: Some(end.to_string()),
            row_type: RowType::Revenue,
            ..Default::default()
        }
    }

    fn make_gtfs() -> GtfsFeed {
        use transit_core::Trip;

        let mut feed = GtfsFeed::new();
        feed.feed.trips.push(Trip::new("TRIP1", "R1", "S1"));
        feed
    }

    #[test]
    fn test_valid_schedule() {
        let gtfs = make_gtfs();
        let mut schedule = make_schedule(vec![
            make_row("TRIP1", "B1", "08:00:00", "09:00:00"),
        ]);

        let validator = Validator::default_config();
        let result = validator.validate(&mut schedule, &gtfs);

        assert!(result.is_valid());
    }

    #[test]
    fn test_missing_trip_strict() {
        let gtfs = make_gtfs();
        let mut schedule = make_schedule(vec![
            make_row("MISSING_TRIP", "B1", "08:00:00", "09:00:00"),
        ]);

        let config = ValidationConfig::strict();
        let validator = Validator::new(config);
        let result = validator.validate(&mut schedule, &gtfs);

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.category == ErrorCategory::GtfsIntegrity));
    }

    #[test]
    fn test_structure_only_validation() {
        let mut schedule = make_schedule(vec![
            make_row("T1", "B1", "08:00:00", "09:00:00"),
            make_row("T2", "B1", "09:10:00", "10:00:00"),
        ]);

        let validator = Validator::default_config();
        let result = validator.validate_structure(&mut schedule);

        assert!(result.is_valid());
        assert_eq!(result.blocks_validated, 1);
    }

    #[test]
    fn test_max_errors_truncation() {
        let gtfs = make_gtfs();
        let mut schedule = make_schedule(vec![
            make_row("MISSING1", "B1", "08:00:00", "09:00:00"),
            make_row("MISSING2", "B1", "09:00:00", "10:00:00"),
            make_row("MISSING3", "B1", "10:00:00", "11:00:00"),
        ]);

        let config = ValidationConfig::strict().with_max_errors(2);
        let validator = Validator::new(config);
        let result = validator.validate(&mut schedule, &gtfs);

        assert_eq!(result.errors.len(), 2);
    }
}

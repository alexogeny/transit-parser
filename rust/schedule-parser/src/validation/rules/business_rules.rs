//! Business rules validation.

use crate::models::{Duty, Schedule, ScheduleRow};
use crate::validation::config::ValidationConfig;

/// Error from business rules validation.
#[derive(Debug, Clone)]
pub struct BusinessRuleError {
    pub error_type: BusinessRuleErrorType,
    pub context: String, // block_id, duty_id, or row info
    pub message: String,
}

/// Types of business rule errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusinessRuleErrorType {
    /// Trip duration exceeds maximum.
    TripTooLong,
    /// Layover between trips too short.
    LayoverTooShort,
    /// Duty length exceeds maximum.
    DutyTooLong,
    /// Continuous driving exceeds maximum.
    ContinuousDrivingTooLong,
    /// Break duration too short.
    BreakTooShort,
}

/// Warning from business rules check.
#[derive(Debug, Clone)]
pub struct BusinessRuleWarning {
    pub code: String,
    pub context: String,
    pub message: String,
}

/// Result of business rules checking.
#[derive(Debug, Clone, Default)]
pub struct BusinessRuleResult {
    pub errors: Vec<BusinessRuleError>,
    pub warnings: Vec<BusinessRuleWarning>,
}

impl BusinessRuleResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Checks business rules.
pub struct BusinessRuleChecker<'a> {
    config: &'a ValidationConfig,
}

impl<'a> BusinessRuleChecker<'a> {
    /// Create a new business rule checker.
    pub fn new(config: &'a ValidationConfig) -> Self {
        Self { config }
    }

    /// Check a single schedule row.
    pub fn check_row(&self, row: &ScheduleRow, row_index: usize) -> BusinessRuleResult {
        let mut result = BusinessRuleResult::default();
        let rules = &self.config.business_rules;

        // Check trip duration
        if row.is_revenue() {
            if let Some(duration) = row.duration_seconds() {
                if duration > rules.max_trip_duration_seconds {
                    result.errors.push(BusinessRuleError {
                        error_type: BusinessRuleErrorType::TripTooLong,
                        context: format!("row {}", row_index),
                        message: format!(
                            "Trip duration {} seconds ({:.1} hours) exceeds maximum {} seconds",
                            duration,
                            duration as f64 / 3600.0,
                            rules.max_trip_duration_seconds
                        ),
                    });
                }
            }
        }

        // Check break duration
        if row.is_break_or_relief() {
            if let Some(duration) = row.duration_seconds() {
                if duration < rules.min_break_duration_seconds {
                    result.errors.push(BusinessRuleError {
                        error_type: BusinessRuleErrorType::BreakTooShort,
                        context: format!("row {}", row_index),
                        message: format!(
                            "Break duration {} seconds ({:.1} min) is less than minimum {} seconds",
                            duration,
                            duration as f64 / 60.0,
                            rules.min_break_duration_seconds
                        ),
                    });
                }
            }
        }

        // Check coordinates if flagging enabled
        if rules.flag_missing_coordinates && row.is_revenue() {
            if row.start_lat.is_none() || row.start_lon.is_none() {
                result.warnings.push(BusinessRuleWarning {
                    code: "W201".to_string(),
                    context: format!("row {}", row_index),
                    message: "Revenue trip missing start coordinates".to_string(),
                });
            }
            if row.end_lat.is_none() || row.end_lon.is_none() {
                result.warnings.push(BusinessRuleWarning {
                    code: "W202".to_string(),
                    context: format!("row {}", row_index),
                    message: "Revenue trip missing end coordinates".to_string(),
                });
            }
        }

        result
    }

    /// Check layover between consecutive rows.
    pub fn check_layover(
        &self,
        prev_row: &ScheduleRow,
        curr_row: &ScheduleRow,
        row_index: usize,
    ) -> BusinessRuleResult {
        let mut result = BusinessRuleResult::default();
        let rules = &self.config.business_rules;

        // Only check layover between revenue trips
        if !prev_row.is_revenue() || !curr_row.is_revenue() {
            return result;
        }

        if let (Some(prev_end), Some(curr_start)) = (
            prev_row.end_time_seconds(),
            curr_row.start_time_seconds(),
        ) {
            if curr_start > prev_end {
                let layover = curr_start - prev_end;
                if layover < rules.min_layover_seconds {
                    result.errors.push(BusinessRuleError {
                        error_type: BusinessRuleErrorType::LayoverTooShort,
                        context: format!("rows {}-{}", row_index - 1, row_index),
                        message: format!(
                            "Layover {} seconds ({:.1} min) is less than minimum {} seconds",
                            layover,
                            layover as f64 / 60.0,
                            rules.min_layover_seconds
                        ),
                    });
                }
            }
        }

        result
    }

    /// Check a duty against business rules.
    pub fn check_duty(&self, duty: &Duty) -> BusinessRuleResult {
        let mut result = BusinessRuleResult::default();
        let rules = &self.config.business_rules;

        // Check duty length
        if let Some(duration) = duty.duration_seconds() {
            if duration > rules.max_duty_length_seconds {
                result.errors.push(BusinessRuleError {
                    error_type: BusinessRuleErrorType::DutyTooLong,
                    context: format!("duty {}", duty.duty_id),
                    message: format!(
                        "Duty length {} seconds ({:.1} hours) exceeds maximum {} seconds",
                        duration,
                        duration as f64 / 3600.0,
                        rules.max_duty_length_seconds
                    ),
                });
            }
        }

        // Check continuous driving
        for (idx, piece) in duty.pieces_of_work().iter().enumerate() {
            if let Some(duration) = piece.duration_seconds() {
                if duration > rules.max_continuous_driving_seconds {
                    result.errors.push(BusinessRuleError {
                        error_type: BusinessRuleErrorType::ContinuousDrivingTooLong,
                        context: format!("duty {} piece {}", duty.duty_id, idx),
                        message: format!(
                            "Continuous driving {} seconds ({:.1} hours) exceeds maximum {} seconds",
                            duration,
                            duration as f64 / 3600.0,
                            rules.max_continuous_driving_seconds
                        ),
                    });
                }
            }
        }

        result
    }

    /// Check all rows in a schedule.
    pub fn check_schedule(&self, schedule: &Schedule) -> BusinessRuleResult {
        let mut combined = BusinessRuleResult::default();
        let rules = &self.config.business_rules;

        // Check individual rows
        for (idx, row) in schedule.rows.iter().enumerate() {
            let result = self.check_row(row, idx);
            combined.errors.extend(result.errors);
            combined.warnings.extend(result.warnings);

            // Check layover with previous row
            if idx > 0 {
                let layover_result = self.check_layover(&schedule.rows[idx - 1], row, idx);
                combined.errors.extend(layover_result.errors);
                combined.warnings.extend(layover_result.warnings);
            }

            // Check max errors limit
            if let Some(max) = self.config.max_errors {
                if combined.errors.len() >= max {
                    return combined;
                }
            }
        }

        // Check orphan trips (trips without block assignment)
        if rules.flag_orphan_trips {
            let orphans: Vec<_> = schedule
                .rows
                .iter()
                .enumerate()
                .filter(|(_, r)| r.is_revenue() && r.block.is_none())
                .collect();

            for (idx, _) in orphans {
                combined.warnings.push(BusinessRuleWarning {
                    code: "W203".to_string(),
                    context: format!("row {}", idx),
                    message: "Revenue trip not assigned to any block".to_string(),
                });
            }
        }

        combined
    }

    /// Check duties if duty validation is enabled.
    pub fn check_duties(&self, schedule: &mut Schedule) -> BusinessRuleResult {
        if !self.config.validate_duty_constraints {
            return BusinessRuleResult::default();
        }

        let mut combined = BusinessRuleResult::default();

        // Get duty IDs to avoid borrow issues
        let duty_ids: Vec<_> = {
            let duties = schedule.duties();
            duties.keys().cloned().collect()
        };

        for duty_id in duty_ids {
            if let Some(duty) = schedule.get_duty(&duty_id) {
                let duty_owned = duty.clone();
                let result = self.check_duty(&duty_owned);
                combined.errors.extend(result.errors);
                combined.warnings.extend(result.warnings);

                if let Some(max) = self.config.max_errors {
                    if combined.errors.len() >= max {
                        break;
                    }
                }
            }
        }

        combined
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RowType;

    fn make_row(start: &str, end: &str, row_type: RowType) -> ScheduleRow {
        ScheduleRow {
            start_time: Some(start.to_string()),
            end_time: Some(end.to_string()),
            row_type,
            trip_id: if row_type == RowType::Revenue {
                Some("T1".to_string())
            } else {
                None
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_trip_too_long() {
        let config = ValidationConfig::new();
        let checker = BusinessRuleChecker::new(&config);

        // 5-hour trip (exceeds default 4-hour max)
        let row = make_row("08:00:00", "13:00:00", RowType::Revenue);
        let result = checker.check_row(&row, 0);

        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == BusinessRuleErrorType::TripTooLong));
    }

    #[test]
    fn test_short_layover() {
        let config = ValidationConfig::new();
        let checker = BusinessRuleChecker::new(&config);

        let row1 = make_row("08:00:00", "09:00:00", RowType::Revenue);
        let row2 = make_row("09:02:00", "10:00:00", RowType::Revenue); // 2 min layover

        let result = checker.check_layover(&row1, &row2, 1);

        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == BusinessRuleErrorType::LayoverTooShort));
    }

    #[test]
    fn test_acceptable_layover() {
        let config = ValidationConfig::new();
        let checker = BusinessRuleChecker::new(&config);

        let row1 = make_row("08:00:00", "09:00:00", RowType::Revenue);
        let row2 = make_row("09:10:00", "10:00:00", RowType::Revenue); // 10 min layover

        let result = checker.check_layover(&row1, &row2, 1);

        assert!(result.is_valid());
    }

    #[test]
    fn test_duty_too_long() {
        let config = ValidationConfig::new();
        let checker = BusinessRuleChecker::new(&config);

        let mut duty = Duty::new("D1".to_string());
        duty.add_row(make_row("06:00:00", "16:00:00", RowType::Revenue)); // 10 hours

        let result = checker.check_duty(&duty);

        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == BusinessRuleErrorType::DutyTooLong));
    }

    #[test]
    fn test_continuous_driving_too_long() {
        let config = ValidationConfig::new();
        let checker = BusinessRuleChecker::new(&config);

        let mut duty = Duty::new("D1".to_string());
        // 5 hours continuous (exceeds 4.5 hour default max)
        duty.add_row(make_row("06:00:00", "11:00:00", RowType::Revenue));

        let result = checker.check_duty(&duty);

        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == BusinessRuleErrorType::ContinuousDrivingTooLong));
    }

    #[test]
    fn test_short_break() {
        let config = ValidationConfig::new();
        let checker = BusinessRuleChecker::new(&config);

        // 15-minute break (less than 30-min default minimum)
        let row = make_row("10:00:00", "10:15:00", RowType::Break);
        let result = checker.check_row(&row, 0);

        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == BusinessRuleErrorType::BreakTooShort));
    }
}

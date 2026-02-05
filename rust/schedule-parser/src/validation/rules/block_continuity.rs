//! Block continuity validation.

use crate::models::{Block, Schedule};
use crate::validation::config::ValidationConfig;

/// Error from block continuity validation.
#[derive(Debug, Clone)]
pub struct BlockContinuityError {
    pub error_type: BlockContinuityErrorType,
    pub block_id: String,
    pub row_index: Option<usize>,
    pub message: String,
}

/// Types of block continuity errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockContinuityErrorType {
    /// End place of trip N doesn't match start place of trip N+1.
    LocationDiscontinuity,
    /// Time gap between trips exceeds threshold.
    TimeGap,
    /// Trips are not in chronological order.
    ChronologyError,
    /// Block has no pull-out.
    MissingPullOut,
    /// Block has no pull-in.
    MissingPullIn,
    /// Block duration too short.
    DurationTooShort,
    /// Block duration too long.
    DurationTooLong,
}

/// Warning from block continuity check.
#[derive(Debug, Clone)]
pub struct BlockContinuityWarning {
    pub code: String,
    pub block_id: String,
    pub row_index: Option<usize>,
    pub message: String,
}

/// Result of block continuity checking.
#[derive(Debug, Clone, Default)]
pub struct BlockContinuityResult {
    pub errors: Vec<BlockContinuityError>,
    pub warnings: Vec<BlockContinuityWarning>,
}

impl BlockContinuityResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Checks block continuity rules.
pub struct BlockContinuityChecker<'a> {
    config: &'a ValidationConfig,
}

impl<'a> BlockContinuityChecker<'a> {
    /// Create a new block continuity checker.
    pub fn new(config: &'a ValidationConfig) -> Self {
        Self { config }
    }

    /// Check a single block.
    pub fn check_block(&self, block: &Block) -> BlockContinuityResult {
        let mut result = BlockContinuityResult::default();

        // Check chronological ordering
        let mut prev_end: Option<u32> = None;
        for (idx, row) in block.rows.iter().enumerate() {
            if let Some(start) = row.start_time_seconds() {
                if let Some(prev) = prev_end {
                    if start < prev {
                        result.errors.push(BlockContinuityError {
                            error_type: BlockContinuityErrorType::ChronologyError,
                            block_id: block.block_id.clone(),
                            row_index: Some(idx),
                            message: format!(
                                "Row {} starts at {} but previous row ends at {}",
                                idx,
                                start,
                                prev
                            ),
                        });
                    }
                }
            }
            prev_end = row.end_time_seconds();
        }

        // Check location continuity
        for disc_idx in block.find_location_discontinuities() {
            let from_place = block.rows[disc_idx].end_place.as_deref().unwrap_or("?");
            let to_place = block.rows[disc_idx + 1].start_place.as_deref().unwrap_or("?");

            // This could be an error in strict mode, warning otherwise
            result.warnings.push(BlockContinuityWarning {
                code: "W101".to_string(),
                block_id: block.block_id.clone(),
                row_index: Some(disc_idx),
                message: format!(
                    "Location discontinuity: row {} ends at '{}' but row {} starts at '{}'",
                    disc_idx, from_place, disc_idx + 1, to_place
                ),
            });
        }

        // Check time gaps
        let max_gap = self.config.business_rules.min_layover_seconds * 10; // 10x min layover as max gap
        for (gap_idx, gap_seconds) in block.find_gaps() {
            if gap_seconds > max_gap {
                result.warnings.push(BlockContinuityWarning {
                    code: "W102".to_string(),
                    block_id: block.block_id.clone(),
                    row_index: Some(gap_idx),
                    message: format!(
                        "Large time gap of {} seconds ({:.1} min) between rows {} and {}",
                        gap_seconds,
                        gap_seconds as f64 / 60.0,
                        gap_idx,
                        gap_idx + 1
                    ),
                });
            }
        }

        // Check block duration
        if let Some(duration) = block.duration_seconds() {
            let rules = &self.config.business_rules;

            if duration < rules.min_block_duration_seconds && rules.min_block_duration_seconds > 0 {
                result.errors.push(BlockContinuityError {
                    error_type: BlockContinuityErrorType::DurationTooShort,
                    block_id: block.block_id.clone(),
                    row_index: None,
                    message: format!(
                        "Block duration {} seconds ({:.1} hours) is less than minimum {} seconds",
                        duration,
                        duration as f64 / 3600.0,
                        rules.min_block_duration_seconds
                    ),
                });
            }

            if duration > rules.max_block_duration_seconds {
                result.errors.push(BlockContinuityError {
                    error_type: BlockContinuityErrorType::DurationTooLong,
                    block_id: block.block_id.clone(),
                    row_index: None,
                    message: format!(
                        "Block duration {} seconds ({:.1} hours) exceeds maximum {} seconds",
                        duration,
                        duration as f64 / 3600.0,
                        rules.max_block_duration_seconds
                    ),
                });
            }
        }

        // Check for pull-out/pull-in (as warnings)
        if block.pull_out().is_none() && self.config.generate_warnings {
            result.warnings.push(BlockContinuityWarning {
                code: "W103".to_string(),
                block_id: block.block_id.clone(),
                row_index: None,
                message: "Block has no explicit pull-out row".to_string(),
            });
        }

        if block.pull_in().is_none() && self.config.generate_warnings {
            result.warnings.push(BlockContinuityWarning {
                code: "W104".to_string(),
                block_id: block.block_id.clone(),
                row_index: None,
                message: "Block has no explicit pull-in row".to_string(),
            });
        }

        result
    }

    /// Check all blocks in a schedule.
    pub fn check_schedule(&self, schedule: &mut Schedule) -> BlockContinuityResult {
        let mut combined = BlockContinuityResult::default();

        // Get block IDs first to avoid borrow issues
        let block_ids: Vec<String> = schedule.block_ids();

        for block_id in block_ids {
            if let Some(block) = schedule.get_block(&block_id) {
                let block_owned = block.clone();
                let result = self.check_block(&block_owned);
                combined.errors.extend(result.errors);
                combined.warnings.extend(result.warnings);

                // Check max errors limit
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
    use crate::models::{RowType, ScheduleRow};

    fn make_block_row(start: &str, end: &str, start_place: &str, end_place: &str) -> ScheduleRow {
        ScheduleRow {
            start_time: Some(start.to_string()),
            end_time: Some(end.to_string()),
            start_place: Some(start_place.to_string()),
            end_place: Some(end_place.to_string()),
            row_type: RowType::Revenue,
            trip_id: Some("T1".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_continuous_block() {
        let mut block = Block::new("B1".to_string());
        block.add_row(make_block_row("08:00:00", "09:00:00", "A", "B"));
        block.add_row(make_block_row("09:00:00", "10:00:00", "B", "C"));
        block.add_row(make_block_row("10:00:00", "11:00:00", "C", "D"));

        let config = ValidationConfig::new();
        let checker = BlockContinuityChecker::new(&config);
        let result = checker.check_block(&block);

        assert!(result.is_valid());
        // Only warning should be about missing pull-out/pull-in
    }

    #[test]
    fn test_location_discontinuity() {
        let mut block = Block::new("B1".to_string());
        block.add_row(make_block_row("08:00:00", "09:00:00", "A", "B"));
        block.add_row(make_block_row("09:00:00", "10:00:00", "C", "D")); // B != C

        let config = ValidationConfig::new();
        let checker = BlockContinuityChecker::new(&config);
        let result = checker.check_block(&block);

        assert!(result.warnings.iter().any(|w| w.code == "W101"));
    }

    #[test]
    fn test_chronology_error() {
        let mut block = Block::new("B1".to_string());
        block.add_row(make_block_row("10:00:00", "11:00:00", "A", "B")); // Later
        block.add_row(make_block_row("08:00:00", "09:00:00", "B", "C")); // Earlier

        let config = ValidationConfig::new();
        let checker = BlockContinuityChecker::new(&config);
        let result = checker.check_block(&block);

        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == BlockContinuityErrorType::ChronologyError));
    }

    #[test]
    fn test_block_duration_too_long() {
        let mut block = Block::new("B1".to_string());
        block.add_row(make_block_row("06:00:00", "23:00:00", "A", "B")); // 17 hours

        let mut config = ValidationConfig::new();
        config.business_rules.max_block_duration_seconds = 57600; // 16 hours

        let checker = BlockContinuityChecker::new(&config);
        let result = checker.check_block(&block);

        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == BlockContinuityErrorType::DurationTooLong));
    }
}

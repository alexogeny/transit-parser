//! Schedule container - holds all schedule rows and derived data.

use super::block::Block;
use super::duty::Duty;
use super::schedule_row::ScheduleRow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete schedule containing all rows and derived rostering data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Schedule {
    /// All schedule rows (trips, deadheads, breaks, etc.).
    pub rows: Vec<ScheduleRow>,

    /// Vehicle blocks derived from rows (keyed by block ID).
    #[serde(skip)]
    blocks: Option<HashMap<String, Block>>,

    /// Driver duties derived from rows (keyed by duty ID).
    #[serde(skip)]
    duties: Option<HashMap<String, Duty>>,

    /// Metadata about the schedule.
    pub metadata: ScheduleMetadata,
}

/// Metadata about a schedule file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduleMetadata {
    /// Source filename.
    pub source_file: Option<String>,

    /// Schedule name/identifier.
    pub name: Option<String>,

    /// Service date range start.
    pub start_date: Option<String>,

    /// Service date range end.
    pub end_date: Option<String>,

    /// Operating company/agency.
    pub operator: Option<String>,

    /// Column mapping used during import.
    pub column_mapping: Option<HashMap<String, String>>,
}

impl Schedule {
    /// Create a new empty schedule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a schedule from rows.
    pub fn from_rows(rows: Vec<ScheduleRow>) -> Self {
        Self {
            rows,
            blocks: None,
            duties: None,
            metadata: ScheduleMetadata::default(),
        }
    }

    /// Number of rows in the schedule.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if the schedule is empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get all revenue (passenger-carrying) trips.
    pub fn revenue_trips(&self) -> impl Iterator<Item = &ScheduleRow> {
        self.rows.iter().filter(|r| r.is_revenue())
    }

    /// Get all deadhead movements.
    pub fn deadheads(&self) -> impl Iterator<Item = &ScheduleRow> {
        self.rows.iter().filter(|r| r.is_deadhead())
    }

    /// Get all rows for a specific block.
    pub fn rows_for_block(&self, block_id: &str) -> Vec<&ScheduleRow> {
        self.rows
            .iter()
            .filter(|r| r.block.as_deref() == Some(block_id))
            .collect()
    }

    /// Get all rows for a specific run.
    pub fn rows_for_run(&self, run_number: &str) -> Vec<&ScheduleRow> {
        self.rows
            .iter()
            .filter(|r| r.run_number.as_deref() == Some(run_number))
            .collect()
    }

    /// Get unique block IDs.
    pub fn block_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.rows.iter().filter_map(|r| r.block.clone()).collect();
        ids.sort();
        ids.dedup();
        ids
    }

    /// Get unique run numbers.
    pub fn run_numbers(&self) -> Vec<String> {
        let mut runs: Vec<String> = self
            .rows
            .iter()
            .filter_map(|r| r.run_number.clone())
            .collect();
        runs.sort();
        runs.dedup();
        runs
    }

    /// Get unique depot codes.
    pub fn depots(&self) -> Vec<String> {
        let mut depots: Vec<String> = self.rows.iter().filter_map(|r| r.depot.clone()).collect();
        depots.sort();
        depots.dedup();
        depots
    }

    /// Get unique trip IDs (revenue trips only).
    pub fn trip_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .rows
            .iter()
            .filter(|r| r.is_revenue())
            .filter_map(|r| r.trip_id.clone())
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }

    /// Derive blocks from schedule rows.
    ///
    /// Groups rows by block ID and creates Block objects.
    pub fn derive_blocks(&mut self) {
        let mut blocks: HashMap<String, Block> = HashMap::new();

        for row in &self.rows {
            if let Some(block_id) = &row.block {
                blocks
                    .entry(block_id.clone())
                    .or_insert_with(|| Block::new(block_id.clone()))
                    .add_row(row.clone());
            }
        }

        // Sort rows within each block by start time
        for block in blocks.values_mut() {
            block.sort_rows_by_time();
        }

        self.blocks = Some(blocks);
    }

    /// Get derived blocks (derives if not already done).
    pub fn blocks(&mut self) -> &HashMap<String, Block> {
        if self.blocks.is_none() {
            self.derive_blocks();
        }
        self.blocks.as_ref().unwrap()
    }

    /// Get a specific block by ID.
    pub fn get_block(&mut self, block_id: &str) -> Option<&Block> {
        self.blocks().get(block_id)
    }

    /// Derive duties from schedule rows.
    ///
    /// Groups rows by duty/run number and creates Duty objects.
    pub fn derive_duties(&mut self) {
        let mut duties: HashMap<String, Duty> = HashMap::new();

        for row in &self.rows {
            // Use duty_id if present, otherwise fall back to run_number
            let duty_key = row.duty_id.clone().or_else(|| row.run_number.clone());

            if let Some(duty_id) = duty_key {
                duties
                    .entry(duty_id.clone())
                    .or_insert_with(|| Duty::new(duty_id))
                    .add_row(row.clone());
            }
        }

        // Sort rows within each duty by start time
        for duty in duties.values_mut() {
            duty.sort_rows_by_time();
        }

        self.duties = Some(duties);
    }

    /// Get derived duties (derives if not already done).
    pub fn duties(&mut self) -> &HashMap<String, Duty> {
        if self.duties.is_none() {
            self.derive_duties();
        }
        self.duties.as_ref().unwrap()
    }

    /// Get a specific duty by ID.
    pub fn get_duty(&mut self, duty_id: &str) -> Option<&Duty> {
        self.duties().get(duty_id)
    }

    /// Add a row to the schedule.
    pub fn add_row(&mut self, row: ScheduleRow) {
        self.rows.push(row);
        // Invalidate caches
        self.blocks = None;
        self.duties = None;
    }

    /// Get summary statistics about the schedule.
    pub fn summary(&self) -> ScheduleSummary {
        let revenue_count = self.rows.iter().filter(|r| r.is_revenue()).count();
        let deadhead_count = self.rows.iter().filter(|r| r.is_deadhead()).count();
        let break_count = self.rows.iter().filter(|r| r.is_break_or_relief()).count();

        ScheduleSummary {
            total_rows: self.rows.len(),
            revenue_trips: revenue_count,
            deadheads: deadhead_count,
            breaks_and_reliefs: break_count,
            unique_blocks: self.block_ids().len(),
            unique_runs: self.run_numbers().len(),
            unique_depots: self.depots().len(),
        }
    }
}

/// Summary statistics for a schedule.
#[derive(Debug, Clone, Default)]
pub struct ScheduleSummary {
    pub total_rows: usize,
    pub revenue_trips: usize,
    pub deadheads: usize,
    pub breaks_and_reliefs: usize,
    pub unique_blocks: usize,
    pub unique_runs: usize,
    pub unique_depots: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::schedule_row::RowType;

    fn sample_row(block: &str, trip_id: Option<&str>, start: &str) -> ScheduleRow {
        ScheduleRow {
            block: Some(block.to_string()),
            trip_id: trip_id.map(|s| s.to_string()),
            start_time: Some(start.to_string()),
            row_type: if trip_id.is_some() {
                RowType::Revenue
            } else {
                RowType::Deadhead
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_schedule_summary() {
        let schedule = Schedule::from_rows(vec![
            sample_row("B1", Some("T1"), "08:00:00"),
            sample_row("B1", Some("T2"), "09:00:00"),
            sample_row("B1", None, "10:00:00"), // deadhead
            sample_row("B2", Some("T3"), "08:30:00"),
        ]);

        let summary = schedule.summary();
        assert_eq!(summary.total_rows, 4);
        assert_eq!(summary.revenue_trips, 3);
        assert_eq!(summary.deadheads, 1);
        assert_eq!(summary.unique_blocks, 2);
    }

    #[test]
    fn test_derive_blocks() {
        let mut schedule = Schedule::from_rows(vec![
            sample_row("B1", Some("T1"), "08:00:00"),
            sample_row("B1", Some("T2"), "09:00:00"),
            sample_row("B2", Some("T3"), "08:30:00"),
        ]);

        let blocks = schedule.blocks();
        assert_eq!(blocks.len(), 2);
        assert!(blocks.contains_key("B1"));
        assert!(blocks.contains_key("B2"));
        assert_eq!(blocks.get("B1").unwrap().rows.len(), 2);
    }
}

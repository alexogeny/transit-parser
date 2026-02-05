//! Block model - vehicle assignment grouping trips.

use super::schedule_row::{RowType, ScheduleRow};
use serde::{Deserialize, Serialize};

/// A vehicle block - a sequence of trips and deadheads assigned to a single vehicle.
///
/// A block represents the work assigned to one vehicle from pull-out to pull-in.
/// It typically includes:
/// - Pull-out from depot
/// - Multiple revenue trips with layovers between them
/// - Possible deadheads (repositioning movements)
/// - Pull-in to depot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Block {
    /// Block identifier.
    pub block_id: String,

    /// All rows (trips, deadheads, layovers) in this block.
    pub rows: Vec<ScheduleRow>,

    /// Assigned depot code.
    pub depot: Option<String>,

    /// Vehicle class required for this block.
    pub vehicle_class: Option<String>,

    /// Specific vehicle type.
    pub vehicle_type: Option<String>,
}

impl Block {
    /// Create a new block with the given ID.
    pub fn new(block_id: String) -> Self {
        Self {
            block_id,
            rows: Vec::new(),
            depot: None,
            vehicle_class: None,
            vehicle_type: None,
        }
    }

    /// Add a row to this block.
    pub fn add_row(&mut self, row: ScheduleRow) {
        // Extract depot/vehicle info from first row that has it
        if self.depot.is_none() {
            self.depot = row.depot.clone();
        }
        if self.vehicle_class.is_none() {
            self.vehicle_class = row.vehicle_class.clone();
        }
        if self.vehicle_type.is_none() {
            self.vehicle_type = row.vehicle_type.clone();
        }
        self.rows.push(row);
    }

    /// Sort rows by start time.
    pub fn sort_rows_by_time(&mut self) {
        self.rows.sort_by(|a, b| {
            let a_time = a.start_time_seconds().unwrap_or(0);
            let b_time = b.start_time_seconds().unwrap_or(0);
            a_time.cmp(&b_time)
        });
    }

    /// Get number of rows in this block.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if block is empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get all revenue trips in this block.
    pub fn revenue_trips(&self) -> impl Iterator<Item = &ScheduleRow> {
        self.rows.iter().filter(|r| r.is_revenue())
    }

    /// Count of revenue trips.
    pub fn revenue_trip_count(&self) -> usize {
        self.rows.iter().filter(|r| r.is_revenue()).count()
    }

    /// Get the pull-out row (first deadhead from depot).
    pub fn pull_out(&self) -> Option<&ScheduleRow> {
        self.rows.iter().find(|r| r.row_type == RowType::PullOut)
    }

    /// Get the pull-in row (last deadhead to depot).
    pub fn pull_in(&self) -> Option<&ScheduleRow> {
        self.rows
            .iter()
            .rev()
            .find(|r| r.row_type == RowType::PullIn)
    }

    /// Get the first row in the block.
    pub fn first_row(&self) -> Option<&ScheduleRow> {
        self.rows.first()
    }

    /// Get the last row in the block.
    pub fn last_row(&self) -> Option<&ScheduleRow> {
        self.rows.last()
    }

    /// Get block start time (earliest start_time).
    pub fn start_time_seconds(&self) -> Option<u32> {
        self.rows
            .iter()
            .filter_map(|r| r.start_time_seconds())
            .min()
    }

    /// Get block end time (latest end_time).
    pub fn end_time_seconds(&self) -> Option<u32> {
        self.rows.iter().filter_map(|r| r.end_time_seconds()).max()
    }

    /// Calculate total block duration in seconds.
    pub fn duration_seconds(&self) -> Option<u32> {
        match (self.start_time_seconds(), self.end_time_seconds()) {
            (Some(start), Some(end)) if end >= start => Some(end - start),
            _ => None,
        }
    }

    /// Calculate total revenue (in-service) time in seconds.
    pub fn revenue_time_seconds(&self) -> u32 {
        self.rows
            .iter()
            .filter(|r| r.is_revenue())
            .filter_map(|r| r.duration_seconds())
            .sum()
    }

    /// Calculate total deadhead time in seconds.
    pub fn deadhead_time_seconds(&self) -> u32 {
        self.rows
            .iter()
            .filter(|r| r.is_deadhead())
            .filter_map(|r| r.duration_seconds())
            .sum()
    }

    /// Check if there's a gap between consecutive rows.
    ///
    /// Returns pairs of (row_index, gap_seconds) where gaps exist.
    pub fn find_gaps(&self) -> Vec<(usize, u32)> {
        let mut gaps = Vec::new();

        for i in 0..self.rows.len().saturating_sub(1) {
            if let (Some(end), Some(start)) = (
                self.rows[i].end_time_seconds(),
                self.rows[i + 1].start_time_seconds(),
            ) {
                if start > end {
                    gaps.push((i, start - end));
                }
            }
        }

        gaps
    }

    /// Check if there's a location discontinuity between consecutive rows.
    ///
    /// Returns indices where end_place of row N != start_place of row N+1.
    pub fn find_location_discontinuities(&self) -> Vec<usize> {
        let mut discontinuities = Vec::new();

        for i in 0..self.rows.len().saturating_sub(1) {
            let end_place = &self.rows[i].end_place;
            let start_place = &self.rows[i + 1].start_place;

            if let (Some(end), Some(start)) = (end_place, start_place) {
                if end != start {
                    discontinuities.push(i);
                }
            }
        }

        discontinuities
    }

    /// Get summary statistics for this block.
    pub fn summary(&self) -> BlockSummary {
        BlockSummary {
            block_id: self.block_id.clone(),
            total_rows: self.rows.len(),
            revenue_trips: self.revenue_trip_count(),
            duration_seconds: self.duration_seconds(),
            revenue_time_seconds: self.revenue_time_seconds(),
            deadhead_time_seconds: self.deadhead_time_seconds(),
            depot: self.depot.clone(),
        }
    }
}

/// Summary statistics for a block.
#[derive(Debug, Clone)]
pub struct BlockSummary {
    pub block_id: String,
    pub total_rows: usize,
    pub revenue_trips: usize,
    pub duration_seconds: Option<u32>,
    pub revenue_time_seconds: u32,
    pub deadhead_time_seconds: u32,
    pub depot: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_row(
        start: &str,
        end: &str,
        row_type: RowType,
        start_place: Option<&str>,
        end_place: Option<&str>,
    ) -> ScheduleRow {
        ScheduleRow {
            start_time: Some(start.to_string()),
            end_time: Some(end.to_string()),
            row_type,
            start_place: start_place.map(String::from),
            end_place: end_place.map(String::from),
            trip_id: if row_type == RowType::Revenue {
                Some("T1".to_string())
            } else {
                None
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_block_duration() {
        let mut block = Block::new("B1".to_string());
        block.add_row(make_row(
            "08:00:00",
            "09:00:00",
            RowType::Revenue,
            None,
            None,
        ));
        block.add_row(make_row(
            "09:15:00",
            "10:00:00",
            RowType::Revenue,
            None,
            None,
        ));

        assert_eq!(block.start_time_seconds(), Some(28800)); // 08:00
        assert_eq!(block.end_time_seconds(), Some(36000)); // 10:00
        assert_eq!(block.duration_seconds(), Some(7200)); // 2 hours
    }

    #[test]
    fn test_find_gaps() {
        let mut block = Block::new("B1".to_string());
        block.add_row(make_row(
            "08:00:00",
            "09:00:00",
            RowType::Revenue,
            None,
            None,
        ));
        block.add_row(make_row(
            "09:30:00",
            "10:00:00",
            RowType::Revenue,
            None,
            None,
        )); // 30 min gap

        let gaps = block.find_gaps();
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0], (0, 1800)); // 30 minutes
    }

    #[test]
    fn test_location_discontinuities() {
        let mut block = Block::new("B1".to_string());
        block.add_row(make_row(
            "08:00:00",
            "09:00:00",
            RowType::Revenue,
            Some("A"),
            Some("B"),
        ));
        block.add_row(make_row(
            "09:00:00",
            "10:00:00",
            RowType::Revenue,
            Some("C"),
            Some("D"),
        )); // B != C

        let discs = block.find_location_discontinuities();
        assert_eq!(discs.len(), 1);
        assert_eq!(discs[0], 0);
    }

    #[test]
    fn test_revenue_time() {
        let mut block = Block::new("B1".to_string());
        block.add_row(make_row(
            "08:00:00",
            "09:00:00",
            RowType::PullOut,
            None,
            None,
        ));
        block.add_row(make_row(
            "09:00:00",
            "10:00:00",
            RowType::Revenue,
            None,
            None,
        )); // 1 hour
        block.add_row(make_row(
            "10:00:00",
            "11:00:00",
            RowType::Revenue,
            None,
            None,
        )); // 1 hour
        block.add_row(make_row(
            "11:00:00",
            "11:30:00",
            RowType::PullIn,
            None,
            None,
        ));

        assert_eq!(block.revenue_time_seconds(), 7200); // 2 hours
        assert_eq!(block.deadhead_time_seconds(), 5400); // 1.5 hours (pull-out + pull-in)
    }
}

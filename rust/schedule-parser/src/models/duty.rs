//! Duty model - driver work assignment.

use super::schedule_row::{RowType, ScheduleRow};
use super::shift::{Break, Shift};
use serde::{Deserialize, Serialize};

/// A driver duty - the work assigned to a single driver for a day.
///
/// A duty represents a driver's complete work assignment and may include:
/// - Multiple pieces of work across different blocks
/// - Breaks and meal periods
/// - Sign-on and sign-off times
/// - Relief points where drivers change
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Duty {
    /// Duty identifier.
    pub duty_id: String,

    /// All rows assigned to this duty.
    pub rows: Vec<ScheduleRow>,

    /// Run number (may be same as duty_id in simple cases).
    pub run_number: Option<String>,

    /// Depot where duty starts/ends.
    pub depot: Option<String>,

    /// Sign-on time (may be earlier than first trip).
    pub sign_on_time: Option<String>,

    /// Sign-off time (may be later than last trip).
    pub sign_off_time: Option<String>,
}

impl Duty {
    /// Create a new duty with the given ID.
    pub fn new(duty_id: String) -> Self {
        Self {
            duty_id,
            rows: Vec::new(),
            run_number: None,
            depot: None,
            sign_on_time: None,
            sign_off_time: None,
        }
    }

    /// Add a row to this duty.
    pub fn add_row(&mut self, row: ScheduleRow) {
        if self.run_number.is_none() {
            self.run_number = row.run_number.clone();
        }
        if self.depot.is_none() {
            self.depot = row.depot.clone();
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

    /// Get number of rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if duty is empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get the earliest start time (or sign_on_time if set).
    pub fn start_time_seconds(&self) -> Option<u32> {
        if let Some(ref sign_on) = self.sign_on_time {
            if let Some(t) = parse_time_to_seconds(sign_on) {
                return Some(t);
            }
        }
        self.rows.iter().filter_map(|r| r.start_time_seconds()).min()
    }

    /// Get the latest end time (or sign_off_time if set).
    pub fn end_time_seconds(&self) -> Option<u32> {
        if let Some(ref sign_off) = self.sign_off_time {
            if let Some(t) = parse_time_to_seconds(sign_off) {
                return Some(t);
            }
        }
        self.rows.iter().filter_map(|r| r.end_time_seconds()).max()
    }

    /// Calculate total duty length in seconds.
    pub fn duration_seconds(&self) -> Option<u32> {
        match (self.start_time_seconds(), self.end_time_seconds()) {
            (Some(start), Some(end)) if end >= start => Some(end - start),
            _ => None,
        }
    }

    /// Calculate total driving time (revenue + deadhead) in seconds.
    pub fn driving_time_seconds(&self) -> u32 {
        self.rows
            .iter()
            .filter(|r| r.is_revenue() || r.is_deadhead())
            .filter_map(|r| r.duration_seconds())
            .sum()
    }

    /// Calculate total break time in seconds.
    pub fn break_time_seconds(&self) -> u32 {
        self.rows
            .iter()
            .filter(|r| r.is_break_or_relief())
            .filter_map(|r| r.duration_seconds())
            .sum()
    }

    /// Get all breaks in this duty.
    pub fn breaks(&self) -> Vec<&ScheduleRow> {
        self.rows
            .iter()
            .filter(|r| matches!(r.row_type, RowType::Break))
            .collect()
    }

    /// Get all reliefs in this duty.
    pub fn reliefs(&self) -> Vec<&ScheduleRow> {
        self.rows
            .iter()
            .filter(|r| matches!(r.row_type, RowType::Relief))
            .collect()
    }

    /// Get unique block IDs this duty works on.
    pub fn block_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .rows
            .iter()
            .filter_map(|r| r.block.clone())
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }

    /// Check if this is a split duty (has a long gap in the middle).
    ///
    /// A split duty has a gap of `min_gap_seconds` or more between work periods.
    pub fn is_split_duty(&self, min_gap_seconds: u32) -> bool {
        for i in 0..self.rows.len().saturating_sub(1) {
            if let (Some(end), Some(start)) = (
                self.rows[i].end_time_seconds(),
                self.rows[i + 1].start_time_seconds(),
            ) {
                if start > end && (start - end) >= min_gap_seconds {
                    return true;
                }
            }
        }
        false
    }

    /// Get pieces of work (continuous driving segments).
    ///
    /// A piece of work is a continuous sequence of driving (trips + deadheads)
    /// without any breaks.
    pub fn pieces_of_work(&self) -> Vec<PieceOfWork> {
        let mut pieces = Vec::new();
        let mut current_piece: Option<PieceOfWork> = None;

        for row in &self.rows {
            if row.is_break_or_relief() {
                // End current piece
                if let Some(piece) = current_piece.take() {
                    pieces.push(piece);
                }
            } else {
                // Add to current piece or start new one
                match current_piece.as_mut() {
                    Some(piece) => {
                        piece.rows.push(row.clone());
                        if let Some(end) = row.end_time_seconds() {
                            piece.end_time_seconds = Some(end);
                        }
                    }
                    None => {
                        current_piece = Some(PieceOfWork {
                            rows: vec![row.clone()],
                            start_time_seconds: row.start_time_seconds(),
                            end_time_seconds: row.end_time_seconds(),
                        });
                    }
                }
            }
        }

        // Don't forget the last piece
        if let Some(piece) = current_piece {
            pieces.push(piece);
        }

        pieces
    }

    /// Convert this duty into a shift (adds break/relief tracking).
    pub fn to_shift(&self) -> Shift {
        let breaks: Vec<Break> = self
            .breaks()
            .iter()
            .filter_map(|row| {
                Some(Break {
                    start_time_seconds: row.start_time_seconds()?,
                    end_time_seconds: row.end_time_seconds()?,
                    location: row.start_place.clone(),
                    is_paid: false, // Default - could be derived from rules
                })
            })
            .collect();

        Shift {
            shift_id: self.duty_id.clone(),
            duty_id: self.duty_id.clone(),
            sign_on_seconds: self.start_time_seconds(),
            sign_off_seconds: self.end_time_seconds(),
            breaks,
            depot: self.depot.clone(),
        }
    }

    /// Get summary statistics.
    pub fn summary(&self) -> DutySummary {
        DutySummary {
            duty_id: self.duty_id.clone(),
            total_rows: self.rows.len(),
            duration_seconds: self.duration_seconds(),
            driving_time_seconds: self.driving_time_seconds(),
            break_time_seconds: self.break_time_seconds(),
            pieces_of_work: self.pieces_of_work().len(),
            blocks_worked: self.block_ids().len(),
        }
    }
}

/// A piece of work - continuous driving segment within a duty.
#[derive(Debug, Clone)]
pub struct PieceOfWork {
    pub rows: Vec<ScheduleRow>,
    pub start_time_seconds: Option<u32>,
    pub end_time_seconds: Option<u32>,
}

impl PieceOfWork {
    /// Duration of this piece of work in seconds.
    pub fn duration_seconds(&self) -> Option<u32> {
        match (self.start_time_seconds, self.end_time_seconds) {
            (Some(start), Some(end)) if end >= start => Some(end - start),
            _ => None,
        }
    }
}

/// Summary statistics for a duty.
#[derive(Debug, Clone)]
pub struct DutySummary {
    pub duty_id: String,
    pub total_rows: usize,
    pub duration_seconds: Option<u32>,
    pub driving_time_seconds: u32,
    pub break_time_seconds: u32,
    pub pieces_of_work: usize,
    pub blocks_worked: usize,
}

/// Parse a time string to seconds (same as in schedule_row, but we need it here too).
fn parse_time_to_seconds(time: &str) -> Option<u32> {
    if let Ok(secs) = time.parse::<u32>() {
        return Some(secs);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_duty_duration() {
        let mut duty = Duty::new("D1".to_string());
        duty.add_row(make_row("06:00:00", "10:00:00", RowType::Revenue));
        duty.add_row(make_row("10:30:00", "14:00:00", RowType::Revenue));

        assert_eq!(duty.duration_seconds(), Some(28800)); // 8 hours
    }

    #[test]
    fn test_split_duty() {
        let mut duty = Duty::new("D1".to_string());
        duty.add_row(make_row("06:00:00", "10:00:00", RowType::Revenue));
        // 4-hour gap
        duty.add_row(make_row("14:00:00", "18:00:00", RowType::Revenue));

        assert!(duty.is_split_duty(7200)); // 2-hour threshold
        assert!(!duty.is_split_duty(18000)); // 5-hour threshold
    }

    #[test]
    fn test_pieces_of_work() {
        let mut duty = Duty::new("D1".to_string());
        duty.add_row(make_row("06:00:00", "10:00:00", RowType::Revenue));
        duty.add_row(make_row("10:00:00", "10:30:00", RowType::Break));
        duty.add_row(make_row("10:30:00", "14:00:00", RowType::Revenue));

        let pieces = duty.pieces_of_work();
        assert_eq!(pieces.len(), 2);
    }

    #[test]
    fn test_break_time() {
        let mut duty = Duty::new("D1".to_string());
        duty.add_row(make_row("06:00:00", "10:00:00", RowType::Revenue));
        duty.add_row(make_row("10:00:00", "10:30:00", RowType::Break)); // 30 min
        duty.add_row(make_row("10:30:00", "14:00:00", RowType::Revenue));
        duty.add_row(make_row("14:00:00", "14:15:00", RowType::Break)); // 15 min

        assert_eq!(duty.break_time_seconds(), 2700); // 45 minutes
    }
}

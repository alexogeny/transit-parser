//! Shift model - sign-on to sign-off period with breaks.

use serde::{Deserialize, Serialize};

/// A shift - a driver's complete work period from sign-on to sign-off.
///
/// A shift represents the full span of time a driver is "on duty" and includes:
/// - Sign-on time (when they report)
/// - Sign-off time (when they're released)
/// - Break periods (paid and unpaid)
/// - Location information for sign-on/off
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Shift {
    /// Shift identifier.
    pub shift_id: String,

    /// Associated duty ID.
    pub duty_id: String,

    /// Sign-on time in seconds since midnight.
    pub sign_on_seconds: Option<u32>,

    /// Sign-off time in seconds since midnight.
    pub sign_off_seconds: Option<u32>,

    /// Breaks within the shift.
    pub breaks: Vec<Break>,

    /// Depot where shift starts/ends.
    pub depot: Option<String>,
}

/// A break period within a shift.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Break {
    /// Start time in seconds since midnight.
    pub start_time_seconds: u32,

    /// End time in seconds since midnight.
    pub end_time_seconds: u32,

    /// Location where break is taken.
    pub location: Option<String>,

    /// Whether this is a paid break.
    pub is_paid: bool,
}

impl Shift {
    /// Create a new shift.
    pub fn new(shift_id: String, duty_id: String) -> Self {
        Self {
            shift_id,
            duty_id,
            sign_on_seconds: None,
            sign_off_seconds: None,
            breaks: Vec::new(),
            depot: None,
        }
    }

    /// Total shift length in seconds (sign-on to sign-off).
    pub fn total_duration_seconds(&self) -> Option<u32> {
        match (self.sign_on_seconds, self.sign_off_seconds) {
            (Some(on), Some(off)) if off >= on => Some(off - on),
            _ => None,
        }
    }

    /// Total paid time in seconds.
    ///
    /// This is total duration minus unpaid breaks.
    pub fn paid_time_seconds(&self) -> Option<u32> {
        let total = self.total_duration_seconds()?;
        let unpaid_break_time: u32 = self
            .breaks
            .iter()
            .filter(|b| !b.is_paid)
            .map(|b| b.duration_seconds())
            .sum();
        Some(total.saturating_sub(unpaid_break_time))
    }

    /// Total break time in seconds.
    pub fn total_break_time_seconds(&self) -> u32 {
        self.breaks.iter().map(|b| b.duration_seconds()).sum()
    }

    /// Total paid break time.
    pub fn paid_break_time_seconds(&self) -> u32 {
        self.breaks
            .iter()
            .filter(|b| b.is_paid)
            .map(|b| b.duration_seconds())
            .sum()
    }

    /// Total unpaid break time.
    pub fn unpaid_break_time_seconds(&self) -> u32 {
        self.breaks
            .iter()
            .filter(|b| !b.is_paid)
            .map(|b| b.duration_seconds())
            .sum()
    }

    /// Add a break to this shift.
    pub fn add_break(&mut self, brk: Break) {
        self.breaks.push(brk);
    }

    /// Check if breaks are properly ordered and don't overlap.
    pub fn breaks_valid(&self) -> bool {
        if self.breaks.is_empty() {
            return true;
        }

        // Check each break is within shift bounds
        if let (Some(on), Some(off)) = (self.sign_on_seconds, self.sign_off_seconds) {
            for brk in &self.breaks {
                if brk.start_time_seconds < on || brk.end_time_seconds > off {
                    return false;
                }
            }
        }

        // Check breaks don't overlap
        let mut sorted: Vec<_> = self.breaks.iter().collect();
        sorted.sort_by_key(|b| b.start_time_seconds);

        for i in 0..sorted.len().saturating_sub(1) {
            if sorted[i].end_time_seconds > sorted[i + 1].start_time_seconds {
                return false;
            }
        }

        true
    }

    /// Get summary statistics.
    pub fn summary(&self) -> ShiftSummary {
        ShiftSummary {
            shift_id: self.shift_id.clone(),
            total_duration_seconds: self.total_duration_seconds(),
            paid_time_seconds: self.paid_time_seconds(),
            break_count: self.breaks.len(),
            total_break_time_seconds: self.total_break_time_seconds(),
        }
    }
}

impl Break {
    /// Create a new break.
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start_time_seconds: start,
            end_time_seconds: end,
            location: None,
            is_paid: false,
        }
    }

    /// Create a paid break.
    pub fn paid(start: u32, end: u32) -> Self {
        Self {
            start_time_seconds: start,
            end_time_seconds: end,
            location: None,
            is_paid: true,
        }
    }

    /// Duration of this break in seconds.
    pub fn duration_seconds(&self) -> u32 {
        self.end_time_seconds
            .saturating_sub(self.start_time_seconds)
    }

    /// Set the location.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

/// Summary statistics for a shift.
#[derive(Debug, Clone)]
pub struct ShiftSummary {
    pub shift_id: String,
    pub total_duration_seconds: Option<u32>,
    pub paid_time_seconds: Option<u32>,
    pub break_count: usize,
    pub total_break_time_seconds: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift_duration() {
        let shift = Shift {
            shift_id: "S1".to_string(),
            duty_id: "D1".to_string(),
            sign_on_seconds: Some(21600),  // 06:00
            sign_off_seconds: Some(54000), // 15:00
            breaks: vec![],
            depot: None,
        };

        assert_eq!(shift.total_duration_seconds(), Some(32400)); // 9 hours
    }

    #[test]
    fn test_paid_time_with_unpaid_break() {
        let shift = Shift {
            shift_id: "S1".to_string(),
            duty_id: "D1".to_string(),
            sign_on_seconds: Some(21600),           // 06:00
            sign_off_seconds: Some(54000),          // 15:00
            breaks: vec![Break::new(36000, 37800)], // 30 min unpaid break at 10:00
            depot: None,
        };

        assert_eq!(shift.total_duration_seconds(), Some(32400)); // 9 hours
        assert_eq!(shift.paid_time_seconds(), Some(30600)); // 8.5 hours
    }

    #[test]
    fn test_mixed_breaks() {
        let shift = Shift {
            shift_id: "S1".to_string(),
            duty_id: "D1".to_string(),
            sign_on_seconds: Some(21600),
            sign_off_seconds: Some(54000),
            breaks: vec![
                Break::paid(28800, 29700), // 15 min paid at 08:00
                Break::new(36000, 37800),  // 30 min unpaid at 10:00
            ],
            depot: None,
        };

        assert_eq!(shift.paid_break_time_seconds(), 900);
        assert_eq!(shift.unpaid_break_time_seconds(), 1800);
        assert_eq!(shift.total_break_time_seconds(), 2700);
    }

    #[test]
    fn test_breaks_valid() {
        let mut shift = Shift {
            shift_id: "S1".to_string(),
            duty_id: "D1".to_string(),
            sign_on_seconds: Some(21600),
            sign_off_seconds: Some(54000),
            breaks: vec![
                Break::new(28800, 29700), // 08:00-08:15
                Break::new(36000, 37800), // 10:00-10:30
            ],
            depot: None,
        };

        assert!(shift.breaks_valid());

        // Add overlapping break
        shift.breaks.push(Break::new(29000, 29500)); // overlaps with first
        assert!(!shift.breaks_valid());
    }
}

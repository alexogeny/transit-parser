//! Calendar and service pattern models.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A service pattern defining when service is available.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Calendar {
    /// Unique identifier for the service.
    pub service_id: String,

    /// Service availability on Monday.
    pub monday: ServiceAvailability,

    /// Service availability on Tuesday.
    pub tuesday: ServiceAvailability,

    /// Service availability on Wednesday.
    pub wednesday: ServiceAvailability,

    /// Service availability on Thursday.
    pub thursday: ServiceAvailability,

    /// Service availability on Friday.
    pub friday: ServiceAvailability,

    /// Service availability on Saturday.
    pub saturday: ServiceAvailability,

    /// Service availability on Sunday.
    pub sunday: ServiceAvailability,

    /// Start date of the service period.
    pub start_date: NaiveDate,

    /// End date of the service period.
    pub end_date: NaiveDate,
}

impl Calendar {
    pub fn new(service_id: impl Into<String>, start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            service_id: service_id.into(),
            monday: ServiceAvailability::NotAvailable,
            tuesday: ServiceAvailability::NotAvailable,
            wednesday: ServiceAvailability::NotAvailable,
            thursday: ServiceAvailability::NotAvailable,
            friday: ServiceAvailability::NotAvailable,
            saturday: ServiceAvailability::NotAvailable,
            sunday: ServiceAvailability::NotAvailable,
            start_date,
            end_date,
        }
    }

    /// Create a calendar with weekday service only.
    pub fn weekdays(
        service_id: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Self {
        Self {
            service_id: service_id.into(),
            monday: ServiceAvailability::Available,
            tuesday: ServiceAvailability::Available,
            wednesday: ServiceAvailability::Available,
            thursday: ServiceAvailability::Available,
            friday: ServiceAvailability::Available,
            saturday: ServiceAvailability::NotAvailable,
            sunday: ServiceAvailability::NotAvailable,
            start_date,
            end_date,
        }
    }

    /// Create a calendar with weekend service only.
    pub fn weekends(
        service_id: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Self {
        Self {
            service_id: service_id.into(),
            monday: ServiceAvailability::NotAvailable,
            tuesday: ServiceAvailability::NotAvailable,
            wednesday: ServiceAvailability::NotAvailable,
            thursday: ServiceAvailability::NotAvailable,
            friday: ServiceAvailability::NotAvailable,
            saturday: ServiceAvailability::Available,
            sunday: ServiceAvailability::Available,
            start_date,
            end_date,
        }
    }

    /// Create a calendar with daily service.
    pub fn daily(
        service_id: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Self {
        Self {
            service_id: service_id.into(),
            monday: ServiceAvailability::Available,
            tuesday: ServiceAvailability::Available,
            wednesday: ServiceAvailability::Available,
            thursday: ServiceAvailability::Available,
            friday: ServiceAvailability::Available,
            saturday: ServiceAvailability::Available,
            sunday: ServiceAvailability::Available,
            start_date,
            end_date,
        }
    }

    /// Set availability for a specific day.
    pub fn set_day(&mut self, day: chrono::Weekday, available: bool) {
        let value = if available {
            ServiceAvailability::Available
        } else {
            ServiceAvailability::NotAvailable
        };
        match day {
            chrono::Weekday::Mon => self.monday = value,
            chrono::Weekday::Tue => self.tuesday = value,
            chrono::Weekday::Wed => self.wednesday = value,
            chrono::Weekday::Thu => self.thursday = value,
            chrono::Weekday::Fri => self.friday = value,
            chrono::Weekday::Sat => self.saturday = value,
            chrono::Weekday::Sun => self.sunday = value,
        }
    }
}

/// An exception to regular service (addition or removal).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalendarDate {
    /// Service identifier.
    pub service_id: String,

    /// Date of the exception.
    pub date: NaiveDate,

    /// Type of exception.
    pub exception_type: ExceptionType,
}

impl CalendarDate {
    pub fn added(service_id: impl Into<String>, date: NaiveDate) -> Self {
        Self {
            service_id: service_id.into(),
            date,
            exception_type: ExceptionType::Added,
        }
    }

    pub fn removed(service_id: impl Into<String>, date: NaiveDate) -> Self {
        Self {
            service_id: service_id.into(),
            date,
            exception_type: ExceptionType::Removed,
        }
    }
}

/// Whether service is available.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ServiceAvailability {
    /// Service is not available.
    #[default]
    NotAvailable = 0,

    /// Service is available.
    Available = 1,
}

impl ServiceAvailability {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::NotAvailable),
            1 => Some(Self::Available),
            _ => None,
        }
    }

    pub fn is_available(self) -> bool {
        matches!(self, Self::Available)
    }
}

/// Type of calendar exception.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum ExceptionType {
    /// Service added for this date.
    Added = 1,

    /// Service removed for this date.
    Removed = 2,
}

impl ExceptionType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Added),
            2 => Some(Self::Removed),
            _ => None,
        }
    }
}

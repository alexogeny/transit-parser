//! TXC Service types.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A service in TXC (collection of related journeys).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcService {
    /// Service code.
    pub service_code: String,
    /// Private code.
    pub private_code: Option<String>,
    /// Lines belonging to this service.
    pub lines: Vec<TxcLine>,
    /// Operating period.
    pub operating_period: Option<TxcOperatingPeriod>,
    /// Operating profile.
    pub operating_profile: Option<TxcOperatingProfile>,
    /// Registered operator reference.
    pub registered_operator_ref: Option<String>,
    /// Public use flag.
    pub public_use: Option<bool>,
    /// Service description.
    pub description: Option<String>,
    /// Standard service information.
    pub standard_service: Option<TxcStandardService>,
}

impl TxcService {
    pub fn new(service_code: impl Into<String>) -> Self {
        Self {
            service_code: service_code.into(),
            private_code: None,
            lines: Vec::new(),
            operating_period: None,
            operating_profile: None,
            registered_operator_ref: None,
            public_use: None,
            description: None,
            standard_service: None,
        }
    }
}

/// A line within a service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcLine {
    /// Line ID.
    pub id: String,
    /// Line name.
    pub line_name: String,
    /// Outbound description.
    pub outbound_description: Option<String>,
    /// Inbound description.
    pub inbound_description: Option<String>,
}

/// Operating period for a service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcOperatingPeriod {
    /// Start date.
    pub start_date: NaiveDate,
    /// End date (if specified).
    pub end_date: Option<NaiveDate>,
}

/// Operating profile defining when a service runs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TxcOperatingProfile {
    /// Regular day types when service operates.
    pub regular_day_type: Option<TxcRegularDayType>,
    /// Bank holiday operations.
    pub bank_holiday_operation: Option<TxcBankHolidayOperation>,
    /// Special days of operation.
    pub special_days_operation: Option<TxcSpecialDaysOperation>,
}

/// Regular day type (days of the week).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TxcRegularDayType {
    /// Days of the week (using DaysOfWeek pattern).
    pub days_of_week: Option<TxcDaysOfWeek>,
    /// Holidays only flag.
    pub holidays_only: bool,
}

/// Days of the week pattern.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TxcDaysOfWeek {
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
    /// Monday to Friday shorthand.
    pub monday_to_friday: bool,
    /// Monday to Saturday shorthand.
    pub monday_to_saturday: bool,
    /// Monday to Sunday shorthand.
    pub monday_to_sunday: bool,
    /// Weekend shorthand.
    pub weekend: bool,
}

impl TxcDaysOfWeek {
    /// Expand shorthand patterns to individual days.
    pub fn expand(&self) -> (bool, bool, bool, bool, bool, bool, bool) {
        let mut mon = self.monday;
        let mut tue = self.tuesday;
        let mut wed = self.wednesday;
        let mut thu = self.thursday;
        let mut fri = self.friday;
        let mut sat = self.saturday;
        let mut sun = self.sunday;

        if self.monday_to_friday {
            mon = true;
            tue = true;
            wed = true;
            thu = true;
            fri = true;
        }

        if self.monday_to_saturday {
            mon = true;
            tue = true;
            wed = true;
            thu = true;
            fri = true;
            sat = true;
        }

        if self.monday_to_sunday {
            mon = true;
            tue = true;
            wed = true;
            thu = true;
            fri = true;
            sat = true;
            sun = true;
        }

        if self.weekend {
            sat = true;
            sun = true;
        }

        (mon, tue, wed, thu, fri, sat, sun)
    }
}

/// Bank holiday operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TxcBankHolidayOperation {
    /// Days of operation on bank holidays.
    pub days_of_operation: Vec<TxcBankHolidayDay>,
    /// Days of non-operation on bank holidays.
    pub days_of_non_operation: Vec<TxcBankHolidayDay>,
}

/// Bank holiday day types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TxcBankHolidayDay {
    AllBankHolidays,
    NewYearsDay,
    Jan2ndScotland,
    GoodFriday,
    EasterMonday,
    MayDay,
    SpringBank,
    LateSummerBankHolidayNotScotland,
    AugustBankHolidayScotland,
    ChristmasDay,
    BoxingDay,
    ChristmasDayHoliday,
    BoxingDayHoliday,
    NewYearsEve,
    StAndrewsDay,
    Other(u8),
}

/// Special days operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TxcSpecialDaysOperation {
    /// Days of operation.
    pub days_of_operation: Vec<TxcDateRange>,
    /// Days of non-operation.
    pub days_of_non_operation: Vec<TxcDateRange>,
}

/// A date range.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcDateRange {
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

/// Standard service information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcStandardService {
    /// Origin of the service.
    pub origin: Option<String>,
    /// Destination of the service.
    pub destination: Option<String>,
    /// Via points.
    pub vias: Vec<String>,
    /// Journey pattern references.
    pub journey_pattern_refs: Vec<String>,
}

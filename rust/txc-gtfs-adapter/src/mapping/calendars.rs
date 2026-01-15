//! Calendar mapping from TXC OperatingProfiles.

use super::MappingContext;
use crate::ConversionOptions;
use chrono::{Datelike, NaiveDate};
use transit_core::{AdapterError, Calendar, CalendarDate, ExceptionType, ServiceAvailability};
use txc_parser::{TxcDocument, TxcOperatingProfile, TxcService};

/// Map TXC operating profiles to GTFS calendars.
pub fn map_calendars(
    doc: &TxcDocument,
    options: &ConversionOptions,
    ctx: &mut MappingContext,
) -> Result<(Vec<Calendar>, Vec<CalendarDate>), AdapterError> {
    let mut calendars = Vec::new();
    let mut calendar_dates = Vec::new();

    for service in &doc.services {
        let (start_date, end_date) = get_date_range(service, options);

        if let Some(ref profile) = service.operating_profile {
            let service_id = ctx.next_service_id();

            // Map to service for vehicle journeys
            ctx.service_mapping
                .insert(service.service_code.clone(), service_id.clone());

            let calendar = profile_to_calendar(&service_id, profile, start_date, end_date);
            calendars.push(calendar);

            // Add calendar exceptions for bank holidays
            let exceptions = map_bank_holiday_exceptions(&service_id, profile, start_date, end_date);
            calendar_dates.extend(exceptions);
        }
    }

    Ok((calendars, calendar_dates))
}

/// Get the operating period date range.
fn get_date_range(service: &TxcService, options: &ConversionOptions) -> (NaiveDate, NaiveDate) {
    let start = options
        .calendar_start
        .or_else(|| service.operating_period.as_ref().map(|p| p.start_date))
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

    let end = options
        .calendar_end
        .or_else(|| {
            service
                .operating_period
                .as_ref()
                .and_then(|p| p.end_date)
        })
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());

    (start, end)
}

/// Convert operating profile to GTFS calendar.
fn profile_to_calendar(
    service_id: &str,
    profile: &TxcOperatingProfile,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Calendar {
    let (mon, tue, wed, thu, fri, sat, sun) = profile
        .regular_day_type
        .as_ref()
        .and_then(|rdt| rdt.days_of_week.as_ref())
        .map(|dow| dow.expand())
        .unwrap_or((true, true, true, true, true, true, true));

    Calendar {
        service_id: service_id.to_string(),
        monday: to_availability(mon),
        tuesday: to_availability(tue),
        wednesday: to_availability(wed),
        thursday: to_availability(thu),
        friday: to_availability(fri),
        saturday: to_availability(sat),
        sunday: to_availability(sun),
        start_date,
        end_date,
    }
}

fn to_availability(available: bool) -> ServiceAvailability {
    if available {
        ServiceAvailability::Available
    } else {
        ServiceAvailability::NotAvailable
    }
}

/// Map bank holiday operations to calendar date exceptions.
fn map_bank_holiday_exceptions(
    service_id: &str,
    profile: &TxcOperatingProfile,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Vec<CalendarDate> {
    let mut exceptions = Vec::new();

    if let Some(ref bh_op) = profile.bank_holiday_operation {
        // Get bank holiday dates in the range
        let bank_holidays = get_uk_bank_holidays(start_date.year(), end_date.year());

        // Add service on bank holidays if in days_of_operation
        if !bh_op.days_of_operation.is_empty() {
            for date in &bank_holidays {
                if *date >= start_date && *date <= end_date {
                    exceptions.push(CalendarDate {
                        service_id: service_id.to_string(),
                        date: *date,
                        exception_type: ExceptionType::Added,
                    });
                }
            }
        }

        // Remove service on bank holidays if in days_of_non_operation
        if !bh_op.days_of_non_operation.is_empty() {
            for date in &bank_holidays {
                if *date >= start_date && *date <= end_date {
                    exceptions.push(CalendarDate {
                        service_id: service_id.to_string(),
                        date: *date,
                        exception_type: ExceptionType::Removed,
                    });
                }
            }
        }
    }

    exceptions
}

/// Get UK bank holidays for a range of years.
/// This is a simplified version - production code should use a proper calendar library.
fn get_uk_bank_holidays(start_year: i32, end_year: i32) -> Vec<NaiveDate> {
    let mut holidays = Vec::new();

    for year in start_year..=end_year {
        // Fixed holidays
        if let Some(d) = NaiveDate::from_ymd_opt(year, 1, 1) {
            holidays.push(d); // New Year's Day
        }
        if let Some(d) = NaiveDate::from_ymd_opt(year, 12, 25) {
            holidays.push(d); // Christmas Day
        }
        if let Some(d) = NaiveDate::from_ymd_opt(year, 12, 26) {
            holidays.push(d); // Boxing Day
        }

        // Approximate variable holidays
        // Early May bank holiday (first Monday of May)
        if let Some(d) = first_monday_of_month(year, 5) {
            holidays.push(d);
        }

        // Spring bank holiday (last Monday of May)
        if let Some(d) = last_monday_of_month(year, 5) {
            holidays.push(d);
        }

        // Summer bank holiday (last Monday of August)
        if let Some(d) = last_monday_of_month(year, 8) {
            holidays.push(d);
        }
    }

    holidays
}

fn first_monday_of_month(year: i32, month: u32) -> Option<NaiveDate> {
    let first = NaiveDate::from_ymd_opt(year, month, 1)?;
    let weekday = first.weekday().num_days_from_monday();
    let days_to_add = if weekday == 0 { 0 } else { 7 - weekday };
    NaiveDate::from_ymd_opt(year, month, 1 + days_to_add)
}

fn last_monday_of_month(year: i32, month: u32) -> Option<NaiveDate> {
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let first_of_next = NaiveDate::from_ymd_opt(next_year, next_month, 1)?;
    let last_of_month = first_of_next.pred_opt()?;

    let weekday = last_of_month.weekday().num_days_from_monday();
    let days_to_sub = weekday;
    last_of_month.checked_sub_signed(chrono::Duration::days(days_to_sub as i64))
}

//! Raw GTFS types for CSV deserialization.

use serde::{Deserialize, Serialize};

/// Raw agency record from agency.txt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAgency {
    pub agency_id: Option<String>,
    pub agency_name: String,
    pub agency_url: String,
    pub agency_timezone: String,
    pub agency_lang: Option<String>,
    pub agency_phone: Option<String>,
    pub agency_fare_url: Option<String>,
    pub agency_email: Option<String>,
}

/// Raw stop record from stops.txt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawStop {
    pub stop_id: String,
    pub stop_code: Option<String>,
    pub stop_name: Option<String>,
    pub stop_desc: Option<String>,
    pub stop_lat: Option<f64>,
    pub stop_lon: Option<f64>,
    pub zone_id: Option<String>,
    pub stop_url: Option<String>,
    pub location_type: Option<u8>,
    pub parent_station: Option<String>,
    pub stop_timezone: Option<String>,
    pub wheelchair_boarding: Option<u8>,
    pub platform_code: Option<String>,
}

/// Raw route record from routes.txt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRoute {
    pub route_id: String,
    pub agency_id: Option<String>,
    pub route_short_name: Option<String>,
    pub route_long_name: Option<String>,
    pub route_desc: Option<String>,
    pub route_type: u16,
    pub route_url: Option<String>,
    pub route_color: Option<String>,
    pub route_text_color: Option<String>,
    pub route_sort_order: Option<u32>,
    pub continuous_pickup: Option<u8>,
    pub continuous_drop_off: Option<u8>,
    pub network_id: Option<String>,
}

/// Raw trip record from trips.txt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTrip {
    pub route_id: String,
    pub service_id: String,
    pub trip_id: String,
    pub trip_headsign: Option<String>,
    pub trip_short_name: Option<String>,
    pub direction_id: Option<u8>,
    pub block_id: Option<String>,
    pub shape_id: Option<String>,
    pub wheelchair_accessible: Option<u8>,
    pub bikes_allowed: Option<u8>,
}

/// Raw stop time record from stop_times.txt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawStopTime {
    pub trip_id: String,
    pub arrival_time: Option<String>,
    pub departure_time: Option<String>,
    pub stop_id: String,
    pub stop_sequence: u32,
    pub stop_headsign: Option<String>,
    pub pickup_type: Option<u8>,
    pub drop_off_type: Option<u8>,
    pub continuous_pickup: Option<u8>,
    pub continuous_drop_off: Option<u8>,
    pub shape_dist_traveled: Option<f64>,
    pub timepoint: Option<u8>,
}

/// Raw calendar record from calendar.txt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCalendar {
    pub service_id: String,
    pub monday: u8,
    pub tuesday: u8,
    pub wednesday: u8,
    pub thursday: u8,
    pub friday: u8,
    pub saturday: u8,
    pub sunday: u8,
    pub start_date: String,
    pub end_date: String,
}

/// Raw calendar date record from calendar_dates.txt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCalendarDate {
    pub service_id: String,
    pub date: String,
    pub exception_type: u8,
}

/// Raw shape record from shapes.txt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawShape {
    pub shape_id: String,
    pub shape_pt_lat: f64,
    pub shape_pt_lon: f64,
    pub shape_pt_sequence: u32,
    pub shape_dist_traveled: Option<f64>,
}

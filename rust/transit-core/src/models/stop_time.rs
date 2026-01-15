//! Stop time model.

use serde::{Deserialize, Serialize};

/// A time at which a vehicle arrives at and departs from a stop.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StopTime {
    /// Trip this stop time belongs to.
    pub trip_id: String,

    /// Arrival time (seconds from midnight, can exceed 24:00:00 for overnight).
    pub arrival_time: Option<u32>,

    /// Departure time (seconds from midnight).
    pub departure_time: Option<u32>,

    /// Stop identifier.
    pub stop_id: String,

    /// Order of stop within the trip (0-indexed).
    pub stop_sequence: u32,

    /// Text displayed at the stop for this arrival.
    pub stop_headsign: Option<String>,

    /// Pickup type at this stop.
    pub pickup_type: PickupDropoffType,

    /// Drop-off type at this stop.
    pub drop_off_type: PickupDropoffType,

    /// Continuous pickup behavior.
    pub continuous_pickup: Option<u8>,

    /// Continuous drop-off behavior.
    pub continuous_drop_off: Option<u8>,

    /// Distance traveled from first stop (meters).
    pub shape_dist_traveled: Option<f64>,

    /// Whether times are exact or approximate.
    pub timepoint: Timepoint,
}

impl StopTime {
    pub fn new(trip_id: impl Into<String>, stop_id: impl Into<String>, stop_sequence: u32) -> Self {
        Self {
            trip_id: trip_id.into(),
            arrival_time: None,
            departure_time: None,
            stop_id: stop_id.into(),
            stop_sequence,
            stop_headsign: None,
            pickup_type: PickupDropoffType::default(),
            drop_off_type: PickupDropoffType::default(),
            continuous_pickup: None,
            continuous_drop_off: None,
            shape_dist_traveled: None,
            timepoint: Timepoint::default(),
        }
    }

    pub fn with_times(mut self, arrival: u32, departure: u32) -> Self {
        self.arrival_time = Some(arrival);
        self.departure_time = Some(departure);
        self
    }

    /// Format seconds as HH:MM:SS string.
    pub fn format_time(seconds: u32) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    }

    /// Parse HH:MM:SS string to seconds.
    pub fn parse_time(time_str: &str) -> Option<u32> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 3 {
            return None;
        }
        let hours: u32 = parts[0].parse().ok()?;
        let minutes: u32 = parts[1].parse().ok()?;
        let seconds: u32 = parts[2].parse().ok()?;
        Some(hours * 3600 + minutes * 60 + seconds)
    }
}

/// Pickup or drop-off type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum PickupDropoffType {
    /// Regularly scheduled pickup/drop-off.
    #[default]
    Regular = 0,

    /// No pickup/drop-off available.
    None = 1,

    /// Must phone agency to arrange.
    PhoneAgency = 2,

    /// Must coordinate with driver.
    CoordinateWithDriver = 3,
}

impl PickupDropoffType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Regular),
            1 => Some(Self::None),
            2 => Some(Self::PhoneAgency),
            3 => Some(Self::CoordinateWithDriver),
            _ => None,
        }
    }
}

/// Whether times are exact.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Timepoint {
    /// Times are approximate.
    Approximate = 0,

    /// Times are exact.
    #[default]
    Exact = 1,
}

impl Timepoint {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Approximate),
            1 => Some(Self::Exact),
            _ => None,
        }
    }
}

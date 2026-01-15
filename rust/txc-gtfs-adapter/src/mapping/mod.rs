//! Mapping modules for TXC to GTFS conversion.

pub mod agencies;
pub mod calendars;
pub mod routes;
pub mod shapes;
pub mod stop_times;
pub mod stops;
pub mod trips;

use std::collections::HashMap;

/// Context accumulated during conversion.
#[derive(Debug, Default)]
pub struct MappingContext {
    /// Stop ID mapping: ATCO code -> GTFS stop_id.
    pub stop_mapping: HashMap<String, String>,
    /// Route ID mapping: TXC line ID -> GTFS route_id.
    pub route_mapping: HashMap<String, String>,
    /// Service ID mapping: generated service IDs.
    pub service_mapping: HashMap<String, String>,
    /// Journey pattern to section refs.
    pub journey_pattern_sections: HashMap<String, Vec<String>>,
    /// Trip ID mapping: VehicleJourneyCode -> trip_id.
    pub trip_mapping: HashMap<String, String>,
    /// Service ID counter.
    service_counter: u64,
    /// Trip ID counter.
    trip_counter: u64,
    /// Shape ID counter.
    shape_counter: u64,
}

impl MappingContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a new unique service ID.
    pub fn next_service_id(&mut self) -> String {
        self.service_counter += 1;
        format!("SVC{:06}", self.service_counter)
    }

    /// Generate a new unique trip ID.
    pub fn next_trip_id(&mut self) -> String {
        self.trip_counter += 1;
        format!("TRIP{:08}", self.trip_counter)
    }

    /// Generate a new unique shape ID.
    pub fn next_shape_id(&mut self) -> String {
        self.shape_counter += 1;
        format!("SHAPE{:06}", self.shape_counter)
    }
}

/// Parse ISO 8601 duration (PT...) to seconds.
pub fn parse_duration(duration: &str) -> Option<u32> {
    // Handle PT2M, PT30S, PT1H30M, etc.
    if !duration.starts_with("PT") {
        return None;
    }

    let s = &duration[2..];
    let mut total_seconds = 0u32;
    let mut current_num = String::new();

    for c in s.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else {
            let num: u32 = current_num.parse().ok()?;
            current_num.clear();

            match c {
                'H' => total_seconds += num * 3600,
                'M' => total_seconds += num * 60,
                'S' => total_seconds += num,
                _ => return None,
            }
        }
    }

    Some(total_seconds)
}

/// Parse time string (HH:MM:SS or HH:MM) to seconds from midnight.
pub fn parse_time_to_seconds(time: &str) -> Option<u32> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let hours: u32 = parts[0].parse().ok()?;
    let minutes: u32 = parts[1].parse().ok()?;
    let seconds: u32 = if parts.len() > 2 {
        parts[2].parse().ok()?
    } else {
        0
    };

    Some(hours * 3600 + minutes * 60 + seconds)
}

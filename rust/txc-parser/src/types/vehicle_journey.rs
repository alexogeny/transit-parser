//! TXC Vehicle Journey types.

use super::service::TxcOperatingProfile;
use serde::{Deserialize, Serialize};

/// A vehicle journey (single trip).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcVehicleJourney {
    /// Private code (unique identifier).
    pub private_code: Option<String>,
    /// Vehicle journey code.
    pub vehicle_journey_code: String,
    /// Service reference.
    pub service_ref: String,
    /// Line reference.
    pub line_ref: String,
    /// Journey pattern reference.
    pub journey_pattern_ref: Option<String>,
    /// Departure time (HH:MM:SS).
    pub departure_time: String,
    /// Days of operation (can override service-level profile).
    pub operating_profile: Option<TxcOperatingProfile>,
    /// Destination display.
    pub destination_display: Option<String>,
    /// Direction (inbound/outbound).
    pub direction: Option<String>,
    /// Block reference (for vehicle assignment).
    pub block_ref: Option<String>,
    /// Timing links with specific times (for flexible timing).
    pub vehicle_journey_timing_links: Vec<TxcVehicleJourneyTimingLink>,
    /// Notes about the journey.
    pub note: Option<String>,
    /// Operational flag.
    pub operational: Option<bool>,
}

impl TxcVehicleJourney {
    pub fn new(
        vehicle_journey_code: impl Into<String>,
        service_ref: impl Into<String>,
        line_ref: impl Into<String>,
        departure_time: impl Into<String>,
    ) -> Self {
        Self {
            private_code: None,
            vehicle_journey_code: vehicle_journey_code.into(),
            service_ref: service_ref.into(),
            line_ref: line_ref.into(),
            journey_pattern_ref: None,
            departure_time: departure_time.into(),
            operating_profile: None,
            destination_display: None,
            direction: None,
            block_ref: None,
            vehicle_journey_timing_links: Vec::new(),
            note: None,
            operational: None,
        }
    }
}

/// Timing link within a vehicle journey (for specific stop times).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcVehicleJourneyTimingLink {
    /// Journey pattern timing link reference.
    pub journey_pattern_timing_link_ref: String,
    /// Run time override (ISO 8601 duration).
    pub run_time: Option<String>,
    /// From stop wait time override.
    pub from_wait_time: Option<String>,
    /// To stop wait time override.
    pub to_wait_time: Option<String>,
}

//! Trip model.

use serde::{Deserialize, Serialize};

/// A sequence of stops at scheduled times for a vehicle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trip {
    /// Unique identifier for the trip.
    pub id: String,

    /// Route this trip belongs to.
    pub route_id: String,

    /// Service pattern (calendar) for this trip.
    pub service_id: String,

    /// Text displayed to riders identifying the trip's destination.
    pub headsign: Option<String>,

    /// Short name for the trip.
    pub short_name: Option<String>,

    /// Direction of travel for a trip.
    pub direction_id: Option<DirectionId>,

    /// Block identifier for vehicle assignment.
    pub block_id: Option<String>,

    /// Shape identifier for geographic path.
    pub shape_id: Option<String>,

    /// Wheelchair accessibility of the trip.
    pub wheelchair_accessible: Option<WheelchairAccessible>,

    /// Bike allowance for the trip.
    pub bikes_allowed: Option<BikesAllowed>,

    // TXC-specific field
    /// Vehicle journey code from TXC.
    pub vehicle_journey_code: Option<String>,
}

impl Trip {
    pub fn new(
        id: impl Into<String>,
        route_id: impl Into<String>,
        service_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            route_id: route_id.into(),
            service_id: service_id.into(),
            headsign: None,
            short_name: None,
            direction_id: None,
            block_id: None,
            shape_id: None,
            wheelchair_accessible: None,
            bikes_allowed: None,
            vehicle_journey_code: None,
        }
    }

    pub fn with_headsign(mut self, headsign: impl Into<String>) -> Self {
        self.headsign = Some(headsign.into());
        self
    }
}

/// Direction of travel.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum DirectionId {
    /// Outbound travel.
    Outbound = 0,

    /// Inbound travel.
    Inbound = 1,
}

impl DirectionId {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Outbound),
            1 => Some(Self::Inbound),
            _ => None,
        }
    }
}

/// Wheelchair accessibility.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum WheelchairAccessible {
    /// No accessibility information.
    Unknown = 0,

    /// At least one wheelchair can be accommodated.
    Accessible = 1,

    /// No wheelchairs can be accommodated.
    NotAccessible = 2,
}

impl WheelchairAccessible {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::Accessible),
            2 => Some(Self::NotAccessible),
            _ => None,
        }
    }
}

/// Bike allowance.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum BikesAllowed {
    /// No bike information.
    Unknown = 0,

    /// At least one bike can be accommodated.
    Allowed = 1,

    /// No bikes are allowed.
    NotAllowed = 2,
}

impl BikesAllowed {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::Allowed),
            2 => Some(Self::NotAllowed),
            _ => None,
        }
    }
}

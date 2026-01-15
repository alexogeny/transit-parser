//! Stop/station model.

use serde::{Deserialize, Serialize};

/// A location where vehicles stop to pick up or drop off riders.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stop {
    /// Unique identifier for the stop.
    pub id: String,

    /// Short text or number identifying the stop for riders.
    pub code: Option<String>,

    /// Name of the stop as displayed to riders.
    pub name: String,

    /// Description of the stop location.
    pub description: Option<String>,

    /// Latitude of the stop (WGS 84).
    pub latitude: f64,

    /// Longitude of the stop (WGS 84).
    pub longitude: f64,

    /// Fare zone identifier.
    pub zone_id: Option<String>,

    /// URL of a web page about the stop.
    pub url: Option<String>,

    /// Type of location.
    pub location_type: LocationType,

    /// Station that contains this stop (if applicable).
    pub parent_station: Option<String>,

    /// Timezone of the stop.
    pub timezone: Option<String>,

    /// Wheelchair boarding accessibility.
    pub wheelchair_boarding: Option<WheelchairBoarding>,

    /// Platform identifier for a stop within a station.
    pub platform_code: Option<String>,

    // UK-specific fields for TXC compatibility
    /// NaPTAN code for UK stops.
    pub naptan_code: Option<String>,

    /// ATCO code for UK stops.
    pub atco_code: Option<String>,
}

impl Stop {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        latitude: f64,
        longitude: f64,
    ) -> Self {
        Self {
            id: id.into(),
            code: None,
            name: name.into(),
            description: None,
            latitude,
            longitude,
            zone_id: None,
            url: None,
            location_type: LocationType::default(),
            parent_station: None,
            timezone: None,
            wheelchair_boarding: None,
            platform_code: None,
            naptan_code: None,
            atco_code: None,
        }
    }
}

/// Type of location.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum LocationType {
    /// A location where passengers board or disembark.
    #[default]
    Stop = 0,

    /// A physical structure or area containing one or more stops.
    Station = 1,

    /// A location where passengers can enter or exit a station.
    EntranceExit = 2,

    /// A location within a station used in pathways.
    GenericNode = 3,

    /// A specific boarding area within a platform.
    BoardingArea = 4,
}

impl LocationType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Stop),
            1 => Some(Self::Station),
            2 => Some(Self::EntranceExit),
            3 => Some(Self::GenericNode),
            4 => Some(Self::BoardingArea),
            _ => None,
        }
    }
}

/// Wheelchair boarding accessibility.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum WheelchairBoarding {
    /// No accessibility information.
    Unknown = 0,

    /// Some vehicles can be boarded by wheelchair.
    Accessible = 1,

    /// Wheelchair boarding not possible.
    NotAccessible = 2,
}

impl WheelchairBoarding {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::Accessible),
            2 => Some(Self::NotAccessible),
            _ => None,
        }
    }
}

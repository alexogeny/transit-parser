//! Route model.

use serde::{Deserialize, Serialize};

/// A transit route (e.g., a bus line).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Route {
    /// Unique identifier for the route.
    pub id: String,

    /// Agency that operates this route.
    pub agency_id: Option<String>,

    /// Short name of the route (e.g., "32", "A").
    pub short_name: Option<String>,

    /// Full name of the route.
    pub long_name: Option<String>,

    /// Description of the route.
    pub description: Option<String>,

    /// Type of transportation used on the route.
    pub route_type: RouteType,

    /// URL of a web page about the route.
    pub url: Option<String>,

    /// Color for the route (hex without #, e.g., "FF0000").
    pub color: Option<String>,

    /// Color for text on route (hex without #).
    pub text_color: Option<String>,

    /// Order to display routes.
    pub sort_order: Option<u32>,

    /// Continuous pickup behavior.
    pub continuous_pickup: Option<u8>,

    /// Continuous drop-off behavior.
    pub continuous_drop_off: Option<u8>,

    /// Network identifier.
    pub network_id: Option<String>,
}

impl Route {
    pub fn new(id: impl Into<String>, route_type: RouteType) -> Self {
        Self {
            id: id.into(),
            agency_id: None,
            short_name: None,
            long_name: None,
            description: None,
            route_type,
            url: None,
            color: None,
            text_color: None,
            sort_order: None,
            continuous_pickup: None,
            continuous_drop_off: None,
            network_id: None,
        }
    }

    pub fn with_names(
        mut self,
        short_name: impl Into<String>,
        long_name: impl Into<String>,
    ) -> Self {
        self.short_name = Some(short_name.into());
        self.long_name = Some(long_name.into());
        self
    }
}

/// Type of transportation on a route.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[repr(u16)]
pub enum RouteType {
    /// Tram, streetcar, light rail.
    Tram = 0,

    /// Subway, metro.
    Subway = 1,

    /// Rail (intercity or long-distance).
    Rail = 2,

    /// Bus.
    #[default]
    Bus = 3,

    /// Ferry.
    Ferry = 4,

    /// Cable tram.
    CableTram = 5,

    /// Aerial lift (gondola, cable car).
    AerialLift = 6,

    /// Funicular.
    Funicular = 7,

    /// Trolleybus.
    Trolleybus = 11,

    /// Monorail.
    Monorail = 12,
}

impl RouteType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::Tram),
            1 => Some(Self::Subway),
            2 => Some(Self::Rail),
            3 => Some(Self::Bus),
            4 => Some(Self::Ferry),
            5 => Some(Self::CableTram),
            6 => Some(Self::AerialLift),
            7 => Some(Self::Funicular),
            11 => Some(Self::Trolleybus),
            12 => Some(Self::Monorail),
            _ => None,
        }
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

//! TXC Route types.

use serde::{Deserialize, Serialize};

/// A route section in TXC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcRouteSection {
    /// Unique identifier.
    pub id: String,
    /// Route links in this section.
    pub route_links: Vec<TxcRouteLink>,
}

/// A route link connecting two stops.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcRouteLink {
    /// Unique identifier.
    pub id: String,
    /// From stop reference.
    pub from_stop_ref: String,
    /// To stop reference.
    pub to_stop_ref: String,
    /// Direction (inbound/outbound).
    pub direction: Option<String>,
    /// Distance in meters.
    pub distance: Option<u32>,
    /// Track geometry points.
    pub track: Option<Vec<TxcTrackPoint>>,
}

/// A point in track geometry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcTrackPoint {
    pub latitude: f64,
    pub longitude: f64,
}

/// A route combining route sections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcRoute {
    /// Unique identifier.
    pub id: String,
    /// Private code.
    pub private_code: Option<String>,
    /// Route description.
    pub description: Option<String>,
    /// Route section references.
    pub route_section_refs: Vec<String>,
}

//! TXC StopPoint types.

use serde::{Deserialize, Serialize};

/// A stop point in TXC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcStopPoint {
    /// ATCO code (unique UK stop identifier).
    pub atco_code: String,
    /// NaPTAN stop type.
    pub naptan_stop_type: Option<String>,
    /// Common name of the stop.
    pub common_name: Option<String>,
    /// Short common name.
    pub short_common_name: Option<String>,
    /// Landmark near the stop.
    pub landmark: Option<String>,
    /// Street the stop is on.
    pub street: Option<String>,
    /// Indicator (e.g., "Stop A", "Bay 1").
    pub indicator: Option<String>,
    /// Bearing of the stop.
    pub bearing: Option<String>,
    /// Locality name.
    pub locality_name: Option<String>,
    /// Locality qualifier.
    pub locality_qualifier: Option<String>,
    /// Parent locality reference.
    pub parent_locality_ref: Option<String>,
    /// Latitude (if available).
    pub latitude: Option<f64>,
    /// Longitude (if available).
    pub longitude: Option<f64>,
    /// Easting (UK grid).
    pub easting: Option<i32>,
    /// Northing (UK grid).
    pub northing: Option<i32>,
}

impl TxcStopPoint {
    pub fn new(atco_code: impl Into<String>) -> Self {
        Self {
            atco_code: atco_code.into(),
            naptan_stop_type: None,
            common_name: None,
            short_common_name: None,
            landmark: None,
            street: None,
            indicator: None,
            bearing: None,
            locality_name: None,
            locality_qualifier: None,
            parent_locality_ref: None,
            latitude: None,
            longitude: None,
            easting: None,
            northing: None,
        }
    }

    /// Get the display name for this stop.
    pub fn display_name(&self) -> String {
        let base_name = self
            .common_name
            .as_deref()
            .or(self.short_common_name.as_deref())
            .unwrap_or(&self.atco_code);

        if let Some(indicator) = &self.indicator {
            format!("{} ({})", base_name, indicator)
        } else {
            base_name.to_string()
        }
    }
}

/// A reference to a stop point with timing information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcStopPointRef {
    /// Reference to the stop's ATCO code.
    pub stop_point_ref: String,
    /// Activity at this stop.
    pub activity: Option<StopActivity>,
    /// Whether this is a timing point.
    pub timing_status: Option<TimingStatus>,
    /// Wait time at this stop (ISO 8601 duration).
    pub wait_time: Option<String>,
}

/// Activity at a stop.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StopActivity {
    PickUp,
    SetDown,
    PickUpAndSetDown,
    Pass,
}

/// Timing status at a stop.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimingStatus {
    PrincipalTimingPoint,
    TimingPoint,
    OtherPoint,
}

//! TXC Journey Pattern types.

use super::stop_point::{StopActivity, TimingStatus};
use serde::{Deserialize, Serialize};

/// A journey pattern (inside a Service).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcJourneyPattern {
    /// Unique identifier.
    pub id: String,
    /// Destination display text.
    pub destination_display: Option<String>,
    /// Direction (outbound/inbound).
    pub direction: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Route reference.
    pub route_ref: Option<String>,
    /// Journey pattern section references.
    pub section_refs: Vec<String>,
}

/// A journey pattern section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcJourneyPatternSection {
    /// Unique identifier.
    pub id: String,
    /// Timing links in this section.
    pub timing_links: Vec<TxcJourneyPatternTimingLink>,
}

/// A timing link within a journey pattern section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcJourneyPatternTimingLink {
    /// Unique identifier.
    pub id: String,
    /// From stop reference.
    pub from: TxcJourneyPatternStopUsage,
    /// To stop reference.
    pub to: TxcJourneyPatternStopUsage,
    /// Route link reference.
    pub route_link_ref: Option<String>,
    /// Run time (ISO 8601 duration, e.g., "PT2M" for 2 minutes).
    pub run_time: Option<String>,
}

/// Stop usage within a journey pattern.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxcJourneyPatternStopUsage {
    /// Stop point reference (ATCO code).
    pub stop_point_ref: String,
    /// Timing status at this point.
    pub timing_status: Option<TimingStatus>,
    /// Activity at this stop.
    pub activity: Option<StopActivity>,
    /// Wait time (ISO 8601 duration).
    pub wait_time: Option<String>,
    /// Sequence number.
    pub sequence_number: Option<u32>,
}

impl TxcJourneyPatternStopUsage {
    pub fn new(stop_point_ref: impl Into<String>) -> Self {
        Self {
            stop_point_ref: stop_point_ref.into(),
            timing_status: None,
            activity: None,
            wait_time: None,
            sequence_number: None,
        }
    }
}

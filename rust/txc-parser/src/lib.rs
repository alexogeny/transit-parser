//! TransXChange (TXC) format parser.
//!
//! This crate provides parsing functionality for UK TransXChange XML files.

mod reader;
pub mod schema;
pub mod types;

pub use reader::{TxcReader, ReadOptions};
pub use types::*;

use std::path::Path;
use transit_core::ParseError;

/// A parsed TransXChange document.
#[derive(Debug, Clone, Default)]
pub struct TxcDocument {
    /// Schema version of the document.
    pub schema_version: String,
    /// Creation timestamp.
    pub creation_datetime: Option<String>,
    /// Modification timestamp.
    pub modification_datetime: Option<String>,
    /// Filename (if loaded from file).
    pub filename: Option<String>,

    /// Operators in the document.
    pub operators: Vec<TxcOperator>,
    /// Services defined in the document.
    pub services: Vec<TxcService>,
    /// Stop points referenced in the document.
    pub stop_points: Vec<TxcStopPoint>,
    /// Route sections defining route geometry.
    pub route_sections: Vec<TxcRouteSection>,
    /// Routes combining route sections.
    pub routes: Vec<TxcRoute>,
    /// Journey pattern sections.
    pub journey_pattern_sections: Vec<TxcJourneyPatternSection>,
    /// Journey patterns (defined in Services, map to sections).
    pub journey_patterns: Vec<TxcJourneyPattern>,
    /// Vehicle journeys (individual trips).
    pub vehicle_journeys: Vec<TxcVehicleJourney>,
}

impl TxcDocument {
    /// Create a new empty document.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a TXC document from a file path.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        TxcReader::read_path(path.as_ref(), ReadOptions::default())
    }

    /// Parse a TXC document from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        TxcReader::read_bytes(bytes, ReadOptions::default())
    }

    /// Parse a TXC document from a string.
    pub fn from_str(xml: &str) -> Result<Self, ParseError> {
        TxcReader::read_str(xml, ReadOptions::default())
    }
}

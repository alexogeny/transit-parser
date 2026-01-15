//! GTFS Static format parser.
//!
//! This crate provides parsing and writing functionality for GTFS Static feeds.

mod lazy;
mod reader;
mod types;
mod writer;

pub use lazy::LazyGtfsFeed;
pub use reader::{GtfsReader, ReadOptions};
pub use types::*;
pub use writer::{GtfsWriter, WriteOptions};

use std::path::Path;
use transit_core::{ParseError, TransitFeed};

/// A complete GTFS feed.
#[derive(Debug, Clone, Default)]
pub struct GtfsFeed {
    pub feed: TransitFeed,
}

impl GtfsFeed {
    /// Create a new empty GTFS feed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Read a GTFS feed from a directory.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        GtfsReader::read_path(path.as_ref(), ReadOptions::default())
    }

    /// Read a GTFS feed from a ZIP file.
    pub fn from_zip(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        GtfsReader::read_zip(path.as_ref(), ReadOptions::default())
    }

    /// Read a GTFS feed from bytes (ZIP format).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        GtfsReader::read_bytes(bytes, ReadOptions::default())
    }

    /// Write the GTFS feed to a directory.
    pub fn to_path(&self, path: impl AsRef<Path>) -> Result<(), ParseError> {
        GtfsWriter::write_path(&self.feed, path.as_ref(), WriteOptions::default())
    }

    /// Write the GTFS feed to a ZIP file.
    pub fn to_zip(&self, path: impl AsRef<Path>) -> Result<(), ParseError> {
        GtfsWriter::write_zip(&self.feed, path.as_ref(), WriteOptions::default())
    }

    /// Export the GTFS feed as ZIP bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ParseError> {
        GtfsWriter::write_bytes(&self.feed, WriteOptions::default())
    }
}

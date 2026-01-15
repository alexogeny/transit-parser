//! TXC to GTFS conversion adapter.
//!
//! This crate provides functionality to convert TransXChange documents to GTFS feeds.

mod converter;
pub mod mapping;

pub use converter::{
    ConversionOptions, ConversionResult, ConversionStats, TxcToGtfsConverter, UkRegion,
};

use gtfs_parser::GtfsFeed;
use transit_core::AdapterError;
use txc_parser::TxcDocument;

/// Convert a TXC document to a GTFS feed.
pub fn convert(doc: TxcDocument) -> Result<GtfsFeed, AdapterError> {
    let converter = TxcToGtfsConverter::new(ConversionOptions::default());
    let result = converter.convert(doc)?;
    Ok(result.feed)
}

/// Convert a TXC document to a GTFS feed with options.
pub fn convert_with_options(
    doc: TxcDocument,
    options: ConversionOptions,
) -> Result<ConversionResult, AdapterError> {
    let converter = TxcToGtfsConverter::new(options);
    converter.convert(doc)
}

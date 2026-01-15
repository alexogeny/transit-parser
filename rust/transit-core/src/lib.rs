//! Core data models and traits for transit data parsing.
//!
//! This crate provides unified data structures that represent transit data
//! in a format-agnostic way, allowing conversion between TXC, GTFS, and other formats.

pub mod error;
pub mod models;
pub mod traits;

pub use error::{AdapterError, ParseError, ValidationError, Warning};
pub use models::*;
pub use traits::*;

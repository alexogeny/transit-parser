//! Schedule parsing, validation, and generation for transit data.
//!
//! This crate provides tools for working with transit schedules:
//!
//! - **Parsing**: Load schedules from CSV files with flexible column mapping
//! - **Validation**: Validate schedules against GTFS data and business rules
//! - **Generation**: Export schedules in various formats (Optibus-like, Hastus-like, custom)
//! - **Rostering**: Work with blocks (vehicle assignments) and duties (driver assignments)
//! - **Deadheads**: Infer missing deadhead movements (pull-out, pull-in, interlining)
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use schedule_parser::{Schedule, ScheduleReader, ReadOptions};
//! use schedule_parser::validation::{Validator, ValidationConfig};
//! use gtfs_parser::GtfsFeed;
//!
//! // Load a schedule
//! let schedule = ScheduleReader::read_path("schedule.csv", ReadOptions::new())?;
//!
//! // Validate against GTFS
//! let gtfs = GtfsFeed::from_path("gtfs/")?;
//! let validator = Validator::new(ValidationConfig::new());
//! let result = validator.validate(&mut schedule, &gtfs);
//!
//! if !result.is_valid() {
//!     for error in &result.errors {
//!         println!("Error: {}", error.message);
//!     }
//! }
//! ```

pub mod deadhead;
pub mod formats;
pub mod models;
pub mod reader;
pub mod validation;

// Re-exports
pub use deadhead::{DeadheadInferrer, inferrer::InferenceConfig};
pub use formats::{CsvExporter, ExportConfig, ExportPreset};
pub use models::{
    Block, BlockSummary, Break, Deadhead, DeadheadInferenceResult, DeadheadType, Duty,
    DutySummary, PieceOfWork, RowType, Schedule, ScheduleMetadata, ScheduleRow, ScheduleSummary,
    Shift, ShiftSummary,
};
pub use reader::{ColumnMapping, ReadOptions, ScheduleReader};
pub use validation::{
    BusinessRules, GtfsComplianceLevel, ValidationConfig, ValidationResult, Validator,
};

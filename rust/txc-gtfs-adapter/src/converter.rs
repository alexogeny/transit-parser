//! Main TXC to GTFS converter.

use crate::mapping::{self, MappingContext};
use chrono::NaiveDate;
use gtfs_parser::GtfsFeed;
use transit_core::{AdapterError, TransitFeed, Warning};
use txc_parser::TxcDocument;

/// Options for TXC to GTFS conversion.
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Include shapes from route sections.
    pub include_shapes: bool,
    /// Start date for calendar generation (defaults to operating period start).
    pub calendar_start: Option<NaiveDate>,
    /// End date for calendar generation (defaults to operating period end).
    pub calendar_end: Option<NaiveDate>,
    /// UK region for bank holiday handling.
    pub region: UkRegion,
    /// Default agency timezone.
    pub default_timezone: String,
    /// Default agency URL.
    pub default_agency_url: String,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            include_shapes: false,
            calendar_start: None,
            calendar_end: None,
            region: UkRegion::England,
            default_timezone: "Europe/London".to_string(),
            default_agency_url: "https://example.com".to_string(),
        }
    }
}

/// UK regions for bank holiday determination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UkRegion {
    #[default]
    England,
    Scotland,
    Wales,
    NorthernIreland,
}

/// Result of a TXC to GTFS conversion.
#[derive(Debug)]
pub struct ConversionResult {
    /// The converted GTFS feed.
    pub feed: GtfsFeed,
    /// Warnings generated during conversion.
    pub warnings: Vec<Warning>,
    /// Conversion statistics.
    pub stats: ConversionStats,
}

/// Statistics from the conversion process.
#[derive(Debug, Default, Clone)]
pub struct ConversionStats {
    pub agencies_converted: usize,
    pub stops_converted: usize,
    pub routes_converted: usize,
    pub trips_converted: usize,
    pub stop_times_generated: usize,
    pub calendar_entries: usize,
    pub calendar_exceptions: usize,
    pub shapes_generated: usize,
}

/// TXC to GTFS converter.
pub struct TxcToGtfsConverter {
    options: ConversionOptions,
}

impl TxcToGtfsConverter {
    /// Create a new converter with the given options.
    pub fn new(options: ConversionOptions) -> Self {
        Self { options }
    }

    /// Convert a TXC document to a GTFS feed.
    pub fn convert(&self, doc: TxcDocument) -> Result<ConversionResult, AdapterError> {
        let mut ctx = MappingContext::new();
        let mut warnings = Vec::new();
        let mut feed = TransitFeed::new();

        // Phase 1: Map agencies from operators
        feed.agencies = mapping::agencies::map_agencies(&doc, &self.options, &mut ctx)?;

        // Phase 2: Map stops from stop points
        feed.stops = mapping::stops::map_stops(&doc, &mut ctx)?;

        // Phase 3: Map routes from services/lines
        feed.routes = mapping::routes::map_routes(&doc, &mut ctx)?;

        // Phase 4: Map calendars from operating profiles
        let (calendars, calendar_dates) =
            mapping::calendars::map_calendars(&doc, &self.options, &mut ctx)?;
        feed.calendars = calendars;
        feed.calendar_dates = calendar_dates;

        // Phase 5: Map trips from vehicle journeys
        feed.trips = mapping::trips::map_trips(&doc, &mut ctx)?;

        // Phase 6: Generate stop times from journey patterns
        feed.stop_times = mapping::stop_times::map_stop_times(&doc, &mut ctx, &mut warnings)?;

        // Phase 7: Generate shapes if requested
        if self.options.include_shapes {
            feed.shapes = mapping::shapes::map_shapes(&doc, &mut ctx)?;
        }

        // Collect statistics
        let stats = ConversionStats {
            agencies_converted: feed.agencies.len(),
            stops_converted: feed.stops.len(),
            routes_converted: feed.routes.len(),
            trips_converted: feed.trips.len(),
            stop_times_generated: feed.stop_times.len(),
            calendar_entries: feed.calendars.len(),
            calendar_exceptions: feed.calendar_dates.len(),
            shapes_generated: feed.shapes.len(),
        };

        Ok(ConversionResult {
            feed: GtfsFeed { feed },
            warnings,
            stats,
        })
    }

    /// Convert multiple TXC documents into a single GTFS feed.
    pub fn convert_batch(&self, docs: Vec<TxcDocument>) -> Result<ConversionResult, AdapterError> {
        let mut ctx = MappingContext::new();
        let mut all_warnings = Vec::new();
        let mut feed = TransitFeed::new();

        for doc in docs {
            // Map each document and merge
            feed.agencies.extend(mapping::agencies::map_agencies(
                &doc,
                &self.options,
                &mut ctx,
            )?);
            feed.stops
                .extend(mapping::stops::map_stops(&doc, &mut ctx)?);
            feed.routes
                .extend(mapping::routes::map_routes(&doc, &mut ctx)?);

            let (calendars, calendar_dates) =
                mapping::calendars::map_calendars(&doc, &self.options, &mut ctx)?;
            feed.calendars.extend(calendars);
            feed.calendar_dates.extend(calendar_dates);

            feed.trips
                .extend(mapping::trips::map_trips(&doc, &mut ctx)?);

            let mut warnings = Vec::new();
            feed.stop_times.extend(mapping::stop_times::map_stop_times(
                &doc,
                &mut ctx,
                &mut warnings,
            )?);
            all_warnings.extend(warnings);

            if self.options.include_shapes {
                feed.shapes
                    .extend(mapping::shapes::map_shapes(&doc, &mut ctx)?);
            }
        }

        // Deduplicate agencies by ID
        feed.agencies.sort_by(|a, b| a.id.cmp(&b.id));
        feed.agencies.dedup_by(|a, b| a.id == b.id);

        let stats = ConversionStats {
            agencies_converted: feed.agencies.len(),
            stops_converted: feed.stops.len(),
            routes_converted: feed.routes.len(),
            trips_converted: feed.trips.len(),
            stop_times_generated: feed.stop_times.len(),
            calendar_entries: feed.calendars.len(),
            calendar_exceptions: feed.calendar_dates.len(),
            shapes_generated: feed.shapes.len(),
        };

        Ok(ConversionResult {
            feed: GtfsFeed { feed },
            warnings: all_warnings,
            stats,
        })
    }
}

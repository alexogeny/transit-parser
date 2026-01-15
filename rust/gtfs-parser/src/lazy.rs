//! Lazy-loading GTFS feed implementation.
//!
//! This module provides a lazy-loading version of GtfsFeed that defers
//! CSV parsing until first access, similar to partridge's approach.

use crate::reader::{GtfsReader, ReadOptions};
use crate::types::*;
use chrono::NaiveDate;
use csv::ReaderBuilder;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use transit_core::{
    Agency, Calendar, CalendarDate, ExceptionType, LocationType, ParseError, PickupDropoffType,
    Route, RouteType, ServiceAvailability, Shape, ShapePoint, Stop, StopTime, Timepoint,
    TransitFeed, Trip, WheelchairBoarding,
};
use zip::ZipArchive;

/// Source of GTFS data for lazy loading.
enum GtfsSource {
    /// Directory path containing GTFS files.
    Directory(PathBuf),
    /// In-memory ZIP archive bytes.
    Bytes(Vec<u8>),
}

/// A lazy-loading GTFS feed that defers CSV parsing until first access.
///
/// This provides fast initial load times (just directory scan) with
/// on-demand parsing of individual files when accessed.
pub struct LazyGtfsFeed {
    source: GtfsSource,
    options: ReadOptions,

    // Cached parsed data (lazily populated)
    agencies: Mutex<Option<Vec<Agency>>>,
    stops: Mutex<Option<Vec<Stop>>>,
    routes: Mutex<Option<Vec<Route>>>,
    trips: Mutex<Option<Vec<Trip>>>,
    stop_times: Mutex<Option<Vec<StopTime>>>,
    calendars: Mutex<Option<Vec<Calendar>>>,
    calendar_dates: Mutex<Option<Vec<CalendarDate>>>,
    shapes: Mutex<Option<Vec<Shape>>>,

    // Track which files exist (for optional files)
    has_calendar: bool,
    has_calendar_dates: bool,
    has_shapes: bool,
}

impl LazyGtfsFeed {
    /// Create a lazy feed from a directory path.
    ///
    /// This only scans the directory to discover files - no CSV parsing happens.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        let path = path.as_ref().to_path_buf();

        // Just check which files exist
        let has_calendar = path.join("calendar.txt").exists();
        let has_calendar_dates = path.join("calendar_dates.txt").exists();
        let has_shapes = path.join("shapes.txt").exists();

        // Verify required files exist
        for required in &["agency.txt", "stops.txt", "routes.txt", "trips.txt", "stop_times.txt"] {
            if !path.join(required).exists() {
                return Err(ParseError::MissingField(required.to_string()));
            }
        }

        if !has_calendar && !has_calendar_dates {
            return Err(ParseError::MissingField("calendar.txt or calendar_dates.txt".to_string()));
        }

        Ok(Self {
            source: GtfsSource::Directory(path),
            options: ReadOptions::default(),
            agencies: Mutex::new(None),
            stops: Mutex::new(None),
            routes: Mutex::new(None),
            trips: Mutex::new(None),
            stop_times: Mutex::new(None),
            calendars: Mutex::new(None),
            calendar_dates: Mutex::new(None),
            shapes: Mutex::new(None),
            has_calendar,
            has_calendar_dates,
            has_shapes,
        })
    }

    /// Create a lazy feed from a ZIP file path.
    ///
    /// This reads the ZIP file into memory and scans entries - no CSV parsing happens.
    pub fn from_zip(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let mut bytes = Vec::new();
        reader.into_inner().read_to_end(&mut bytes)?;
        Self::from_bytes(bytes)
    }

    /// Create a lazy feed from in-memory ZIP bytes.
    ///
    /// This scans the ZIP entries - no CSV parsing happens.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, ParseError> {
        // Scan the ZIP to see what files exist
        let cursor = Cursor::new(&bytes);
        let archive = ZipArchive::new(cursor)
            .map_err(|e| ParseError::Zip(e.to_string()))?;

        let file_names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();

        let has_calendar = file_names.iter().any(|f| f.ends_with("calendar.txt"));
        let has_calendar_dates = file_names.iter().any(|f| f.ends_with("calendar_dates.txt"));
        let has_shapes = file_names.iter().any(|f| f.ends_with("shapes.txt"));

        // Verify required files
        for required in &["agency.txt", "stops.txt", "routes.txt", "trips.txt", "stop_times.txt"] {
            if !file_names.iter().any(|f| f.ends_with(required)) {
                return Err(ParseError::MissingField(required.to_string()));
            }
        }

        if !has_calendar && !has_calendar_dates {
            return Err(ParseError::MissingField("calendar.txt or calendar_dates.txt".to_string()));
        }

        Ok(Self {
            source: GtfsSource::Bytes(bytes),
            options: ReadOptions::default(),
            agencies: Mutex::new(None),
            stops: Mutex::new(None),
            routes: Mutex::new(None),
            trips: Mutex::new(None),
            stop_times: Mutex::new(None),
            calendars: Mutex::new(None),
            calendar_dates: Mutex::new(None),
            shapes: Mutex::new(None),
            has_calendar,
            has_calendar_dates,
            has_shapes,
        })
    }

    // ========================================
    // Count accessors (fast, parse only header)
    // ========================================

    /// Count agencies without parsing full data.
    pub fn agency_count(&self) -> Result<usize, ParseError> {
        // If already cached, return from cache
        if let Some(ref agencies) = *self.agencies.lock().unwrap() {
            return Ok(agencies.len());
        }
        self.count_records("agency.txt")
    }

    /// Count stops without parsing full data.
    pub fn stop_count(&self) -> Result<usize, ParseError> {
        if let Some(ref stops) = *self.stops.lock().unwrap() {
            return Ok(stops.len());
        }
        self.count_records("stops.txt")
    }

    /// Count routes without parsing full data.
    pub fn route_count(&self) -> Result<usize, ParseError> {
        if let Some(ref routes) = *self.routes.lock().unwrap() {
            return Ok(routes.len());
        }
        self.count_records("routes.txt")
    }

    /// Count trips without parsing full data.
    pub fn trip_count(&self) -> Result<usize, ParseError> {
        if let Some(ref trips) = *self.trips.lock().unwrap() {
            return Ok(trips.len());
        }
        self.count_records("trips.txt")
    }

    /// Count stop_times without parsing full data.
    pub fn stop_time_count(&self) -> Result<usize, ParseError> {
        if let Some(ref stop_times) = *self.stop_times.lock().unwrap() {
            return Ok(stop_times.len());
        }
        self.count_records("stop_times.txt")
    }

    /// Count calendars without parsing full data.
    pub fn calendar_count(&self) -> Result<usize, ParseError> {
        if !self.has_calendar {
            return Ok(0);
        }
        if let Some(ref calendars) = *self.calendars.lock().unwrap() {
            return Ok(calendars.len());
        }
        self.count_records("calendar.txt")
    }

    /// Count calendar_dates without parsing full data.
    pub fn calendar_date_count(&self) -> Result<usize, ParseError> {
        if !self.has_calendar_dates {
            return Ok(0);
        }
        if let Some(ref dates) = *self.calendar_dates.lock().unwrap() {
            return Ok(dates.len());
        }
        self.count_records("calendar_dates.txt")
    }

    /// Count shapes without parsing full data.
    pub fn shape_count(&self) -> Result<usize, ParseError> {
        if !self.has_shapes {
            return Ok(0);
        }
        if let Some(ref shapes) = *self.shapes.lock().unwrap() {
            return Ok(shapes.len());
        }
        // For shapes, we need to count unique shape_ids, not rows
        // Fall back to full parse for accurate count
        Ok(self.shapes()?.len())
    }

    fn count_records(&self, filename: &str) -> Result<usize, ParseError> {
        match &self.source {
            GtfsSource::Directory(path) => {
                let file = File::open(path.join(filename))?;
                let reader = BufReader::new(file);
                Self::count_csv_records(reader)
            }
            GtfsSource::Bytes(bytes) => {
                let cursor = Cursor::new(bytes);
                let mut archive = ZipArchive::new(cursor)
                    .map_err(|e| ParseError::Zip(e.to_string()))?;
                let file = archive.by_name(filename)
                    .map_err(|_| ParseError::MissingField(filename.to_string()))?;
                Self::count_csv_records(file)
            }
        }
    }

    fn count_csv_records<R: Read>(reader: R) -> Result<usize, ParseError> {
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        // Count records without deserializing
        let count = csv_reader.records().count();
        Ok(count)
    }

    // ========================================
    // Data accessors (lazy parse + cache)
    // ========================================

    /// Get all agencies (parses on first access).
    pub fn agencies(&self) -> Result<Vec<Agency>, ParseError> {
        let mut cache = self.agencies.lock().unwrap();
        if let Some(ref agencies) = *cache {
            return Ok(agencies.clone());
        }

        let agencies = self.parse_file("agency.txt")?;
        *cache = Some(agencies.clone());
        Ok(agencies)
    }

    /// Get all stops (parses on first access).
    pub fn stops(&self) -> Result<Vec<Stop>, ParseError> {
        let mut cache = self.stops.lock().unwrap();
        if let Some(ref stops) = *cache {
            return Ok(stops.clone());
        }

        let stops = self.parse_file("stops.txt")?;
        *cache = Some(stops.clone());
        Ok(stops)
    }

    /// Get all routes (parses on first access).
    pub fn routes(&self) -> Result<Vec<Route>, ParseError> {
        let mut cache = self.routes.lock().unwrap();
        if let Some(ref routes) = *cache {
            return Ok(routes.clone());
        }

        let routes = self.parse_file("routes.txt")?;
        *cache = Some(routes.clone());
        Ok(routes)
    }

    /// Get all trips (parses on first access).
    pub fn trips(&self) -> Result<Vec<Trip>, ParseError> {
        let mut cache = self.trips.lock().unwrap();
        if let Some(ref trips) = *cache {
            return Ok(trips.clone());
        }

        let trips = self.parse_file("trips.txt")?;
        *cache = Some(trips.clone());
        Ok(trips)
    }

    /// Get all stop_times (parses on first access).
    pub fn stop_times(&self) -> Result<Vec<StopTime>, ParseError> {
        let mut cache = self.stop_times.lock().unwrap();
        if let Some(ref stop_times) = *cache {
            return Ok(stop_times.clone());
        }

        let stop_times = self.parse_file("stop_times.txt")?;
        *cache = Some(stop_times.clone());
        Ok(stop_times)
    }

    /// Get all calendars (parses on first access).
    pub fn calendars(&self) -> Result<Vec<Calendar>, ParseError> {
        if !self.has_calendar {
            return Ok(vec![]);
        }

        let mut cache = self.calendars.lock().unwrap();
        if let Some(ref calendars) = *cache {
            return Ok(calendars.clone());
        }

        let calendars = self.parse_file("calendar.txt")?;
        *cache = Some(calendars.clone());
        Ok(calendars)
    }

    /// Get all calendar_dates (parses on first access).
    pub fn calendar_dates(&self) -> Result<Vec<CalendarDate>, ParseError> {
        if !self.has_calendar_dates {
            return Ok(vec![]);
        }

        let mut cache = self.calendar_dates.lock().unwrap();
        if let Some(ref dates) = *cache {
            return Ok(dates.clone());
        }

        let dates = self.parse_file("calendar_dates.txt")?;
        *cache = Some(dates.clone());
        Ok(dates)
    }

    /// Get all shapes (parses on first access).
    pub fn shapes(&self) -> Result<Vec<Shape>, ParseError> {
        if !self.has_shapes {
            return Ok(vec![]);
        }

        let mut cache = self.shapes.lock().unwrap();
        if let Some(ref shapes) = *cache {
            return Ok(shapes.clone());
        }

        let shapes = self.parse_shapes()?;
        *cache = Some(shapes.clone());
        Ok(shapes)
    }

    /// Materialize all data into a regular TransitFeed.
    ///
    /// This parses all files that haven't been accessed yet.
    pub fn materialize(&self) -> Result<TransitFeed, ParseError> {
        Ok(TransitFeed {
            agencies: self.agencies()?,
            stops: self.stops()?,
            routes: self.routes()?,
            trips: self.trips()?,
            stop_times: self.stop_times()?,
            calendars: self.calendars()?,
            calendar_dates: self.calendar_dates()?,
            shapes: self.shapes()?,
        })
    }

    // ========================================
    // Internal parsing helpers
    // ========================================

    fn parse_file<T>(&self, filename: &str) -> Result<Vec<T>, ParseError>
    where
        T: FromRawLazy,
        T::Raw: for<'de> serde::Deserialize<'de>,
    {
        match &self.source {
            GtfsSource::Directory(path) => {
                let file = File::open(path.join(filename))?;
                Self::parse_csv::<_, T>(file, &self.options)
            }
            GtfsSource::Bytes(bytes) => {
                let cursor = Cursor::new(bytes);
                let mut archive = ZipArchive::new(cursor)
                    .map_err(|e| ParseError::Zip(e.to_string()))?;
                let file = archive.by_name(filename)
                    .map_err(|_| ParseError::MissingField(filename.to_string()))?;
                Self::parse_csv::<_, T>(file, &self.options)
            }
        }
    }

    fn parse_csv<R: Read, T>(reader: R, options: &ReadOptions) -> Result<Vec<T>, ParseError>
    where
        T: FromRawLazy,
        T::Raw: for<'de> serde::Deserialize<'de>,
    {
        let mut csv_reader = ReaderBuilder::new()
            .flexible(options.lenient)
            .trim(csv::Trim::All)
            .from_reader(reader);

        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            match result {
                Ok(raw) => records.push(T::from_raw(raw)?),
                Err(e) if options.lenient => {
                    eprintln!("Warning: skipping malformed record: {}", e);
                }
                Err(e) => return Err(ParseError::Csv(e.to_string())),
            }
        }
        Ok(records)
    }

    fn parse_shapes(&self) -> Result<Vec<Shape>, ParseError> {
        let raw_shapes: Vec<RawShape> = match &self.source {
            GtfsSource::Directory(path) => {
                let file = File::open(path.join("shapes.txt"))?;
                Self::parse_csv_raw(file, &self.options)?
            }
            GtfsSource::Bytes(bytes) => {
                let cursor = Cursor::new(bytes);
                let mut archive = ZipArchive::new(cursor)
                    .map_err(|e| ParseError::Zip(e.to_string()))?;
                let file = archive.by_name("shapes.txt")
                    .map_err(|_| ParseError::MissingField("shapes.txt".to_string()))?;
                Self::parse_csv_raw(file, &self.options)?
            }
        };

        let mut shapes_map: HashMap<String, Vec<ShapePoint>> = HashMap::new();
        for raw in raw_shapes {
            let point = ShapePoint {
                latitude: raw.shape_pt_lat,
                longitude: raw.shape_pt_lon,
                sequence: raw.shape_pt_sequence,
                dist_traveled: raw.shape_dist_traveled,
            };
            shapes_map
                .entry(raw.shape_id)
                .or_default()
                .push(point);
        }

        let shapes = shapes_map
            .into_iter()
            .map(|(id, mut points)| {
                points.sort_by_key(|p| p.sequence);
                Shape { id, points }
            })
            .collect();

        Ok(shapes)
    }

    fn parse_csv_raw<R: Read, T>(reader: R, options: &ReadOptions) -> Result<Vec<T>, ParseError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let mut csv_reader = ReaderBuilder::new()
            .flexible(options.lenient)
            .trim(csv::Trim::All)
            .from_reader(reader);

        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            match result {
                Ok(record) => records.push(record),
                Err(e) if options.lenient => {
                    eprintln!("Warning: skipping malformed record: {}", e);
                }
                Err(e) => return Err(ParseError::Csv(e.to_string())),
            }
        }
        Ok(records)
    }
}

// Re-use FromRaw trait pattern for lazy loading
trait FromRawLazy: Sized {
    type Raw;
    fn from_raw(raw: Self::Raw) -> Result<Self, ParseError>;
}

impl FromRawLazy for Agency {
    type Raw = RawAgency;

    fn from_raw(raw: RawAgency) -> Result<Self, ParseError> {
        Ok(Agency {
            id: raw.agency_id,
            name: raw.agency_name,
            url: raw.agency_url,
            timezone: raw.agency_timezone,
            lang: raw.agency_lang,
            phone: raw.agency_phone,
            fare_url: raw.agency_fare_url,
            email: raw.agency_email,
        })
    }
}

impl FromRawLazy for Stop {
    type Raw = RawStop;

    fn from_raw(raw: RawStop) -> Result<Self, ParseError> {
        Ok(Stop {
            id: raw.stop_id,
            code: raw.stop_code,
            name: raw.stop_name.unwrap_or_default(),
            description: raw.stop_desc,
            latitude: raw.stop_lat.unwrap_or(0.0),
            longitude: raw.stop_lon.unwrap_or(0.0),
            zone_id: raw.zone_id,
            url: raw.stop_url,
            location_type: raw.location_type.and_then(LocationType::from_u8).unwrap_or_default(),
            parent_station: raw.parent_station,
            timezone: raw.stop_timezone,
            wheelchair_boarding: raw.wheelchair_boarding.and_then(WheelchairBoarding::from_u8),
            platform_code: raw.platform_code,
            naptan_code: None,
            atco_code: None,
        })
    }
}

impl FromRawLazy for Route {
    type Raw = RawRoute;

    fn from_raw(raw: RawRoute) -> Result<Self, ParseError> {
        Ok(Route {
            id: raw.route_id,
            agency_id: raw.agency_id,
            short_name: raw.route_short_name,
            long_name: raw.route_long_name,
            description: raw.route_desc,
            route_type: RouteType::from_u16(raw.route_type).unwrap_or_default(),
            url: raw.route_url,
            color: raw.route_color,
            text_color: raw.route_text_color,
            sort_order: raw.route_sort_order,
            continuous_pickup: raw.continuous_pickup,
            continuous_drop_off: raw.continuous_drop_off,
            network_id: raw.network_id,
        })
    }
}

impl FromRawLazy for Trip {
    type Raw = RawTrip;

    fn from_raw(raw: RawTrip) -> Result<Self, ParseError> {
        Ok(Trip {
            id: raw.trip_id,
            route_id: raw.route_id,
            service_id: raw.service_id,
            headsign: raw.trip_headsign,
            short_name: raw.trip_short_name,
            direction_id: raw.direction_id.and_then(transit_core::DirectionId::from_u8),
            block_id: raw.block_id,
            shape_id: raw.shape_id,
            wheelchair_accessible: raw.wheelchair_accessible.and_then(transit_core::WheelchairAccessible::from_u8),
            bikes_allowed: raw.bikes_allowed.and_then(transit_core::BikesAllowed::from_u8),
            vehicle_journey_code: None,
        })
    }
}

impl FromRawLazy for StopTime {
    type Raw = RawStopTime;

    fn from_raw(raw: RawStopTime) -> Result<Self, ParseError> {
        Ok(StopTime {
            trip_id: raw.trip_id,
            arrival_time: raw.arrival_time.as_deref().and_then(StopTime::parse_time),
            departure_time: raw.departure_time.as_deref().and_then(StopTime::parse_time),
            stop_id: raw.stop_id,
            stop_sequence: raw.stop_sequence,
            stop_headsign: raw.stop_headsign,
            pickup_type: raw.pickup_type.and_then(PickupDropoffType::from_u8).unwrap_or_default(),
            drop_off_type: raw.drop_off_type.and_then(PickupDropoffType::from_u8).unwrap_or_default(),
            continuous_pickup: raw.continuous_pickup,
            continuous_drop_off: raw.continuous_drop_off,
            shape_dist_traveled: raw.shape_dist_traveled,
            timepoint: raw.timepoint.and_then(Timepoint::from_u8).unwrap_or_default(),
        })
    }
}

impl FromRawLazy for Calendar {
    type Raw = RawCalendar;

    fn from_raw(raw: RawCalendar) -> Result<Self, ParseError> {
        let start_date = parse_gtfs_date(&raw.start_date)?;
        let end_date = parse_gtfs_date(&raw.end_date)?;

        Ok(Calendar {
            service_id: raw.service_id,
            monday: ServiceAvailability::from_u8(raw.monday).unwrap_or_default(),
            tuesday: ServiceAvailability::from_u8(raw.tuesday).unwrap_or_default(),
            wednesday: ServiceAvailability::from_u8(raw.wednesday).unwrap_or_default(),
            thursday: ServiceAvailability::from_u8(raw.thursday).unwrap_or_default(),
            friday: ServiceAvailability::from_u8(raw.friday).unwrap_or_default(),
            saturday: ServiceAvailability::from_u8(raw.saturday).unwrap_or_default(),
            sunday: ServiceAvailability::from_u8(raw.sunday).unwrap_or_default(),
            start_date,
            end_date,
        })
    }
}

impl FromRawLazy for CalendarDate {
    type Raw = RawCalendarDate;

    fn from_raw(raw: RawCalendarDate) -> Result<Self, ParseError> {
        let date = parse_gtfs_date(&raw.date)?;
        let exception_type = ExceptionType::from_u8(raw.exception_type)
            .ok_or_else(|| ParseError::InvalidData(format!("Invalid exception_type: {}", raw.exception_type)))?;

        Ok(CalendarDate {
            service_id: raw.service_id,
            date,
            exception_type,
        })
    }
}

fn parse_gtfs_date(s: &str) -> Result<NaiveDate, ParseError> {
    NaiveDate::parse_from_str(s, "%Y%m%d")
        .map_err(|_| ParseError::InvalidDate(s.to_string()))
}

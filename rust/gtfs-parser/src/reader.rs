//! GTFS feed reader.

use crate::types::*;
use crate::GtfsFeed;
use chrono::NaiveDate;
use csv::ReaderBuilder;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use transit_core::{
    Agency, Calendar, CalendarDate, ExceptionType, LocationType, ParseError, PickupDropoffType,
    Route, RouteType, ServiceAvailability, Shape, ShapePoint, Stop, StopTime, Timepoint,
    TransitFeed, Trip, WheelchairBoarding,
};
use zip::ZipArchive;

/// Options for reading GTFS feeds.
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    /// Whether to be lenient with malformed data.
    pub lenient: bool,
}

/// GTFS feed reader.
pub struct GtfsReader;

impl GtfsReader {
    /// Read a GTFS feed from a directory.
    pub fn read_path(path: &Path, options: ReadOptions) -> Result<GtfsFeed, ParseError> {
        let mut feed = TransitFeed::new();

        // Read required files
        feed.agencies = Self::read_agencies(&path.join("agency.txt"), &options)?;
        feed.stops = Self::read_stops(&path.join("stops.txt"), &options)?;
        feed.routes = Self::read_routes(&path.join("routes.txt"), &options)?;
        feed.trips = Self::read_trips(&path.join("trips.txt"), &options)?;
        feed.stop_times = Self::read_stop_times(&path.join("stop_times.txt"), &options)?;

        // Calendar can be either calendar.txt or calendar_dates.txt (or both)
        let calendar_path = path.join("calendar.txt");
        if calendar_path.exists() {
            feed.calendars = Self::read_calendars(&calendar_path, &options)?;
        }

        let calendar_dates_path = path.join("calendar_dates.txt");
        if calendar_dates_path.exists() {
            feed.calendar_dates = Self::read_calendar_dates(&calendar_dates_path, &options)?;
        }

        // Optional files
        let shapes_path = path.join("shapes.txt");
        if shapes_path.exists() {
            feed.shapes = Self::read_shapes(&shapes_path, &options)?;
        }

        Ok(GtfsFeed { feed })
    }

    /// Read a GTFS feed from a ZIP file.
    pub fn read_zip(path: &Path, options: ReadOptions) -> Result<GtfsFeed, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::read_zip_archive(reader, options)
    }

    /// Read a GTFS feed from bytes (ZIP format).
    pub fn read_bytes(bytes: &[u8], options: ReadOptions) -> Result<GtfsFeed, ParseError> {
        let cursor = std::io::Cursor::new(bytes);
        Self::read_zip_archive(cursor, options)
    }

    fn read_zip_archive<R: Read + std::io::Seek>(
        reader: R,
        options: ReadOptions,
    ) -> Result<GtfsFeed, ParseError> {
        let mut archive =
            ZipArchive::new(reader).map_err(|e| ParseError::Zip(e.to_string()))?;
        let mut feed = TransitFeed::new();

        // Read required files
        feed.agencies = Self::read_csv_from_zip(&mut archive, "agency.txt", &options)?;
        feed.stops = Self::read_csv_from_zip(&mut archive, "stops.txt", &options)?;
        feed.routes = Self::read_csv_from_zip(&mut archive, "routes.txt", &options)?;
        feed.trips = Self::read_csv_from_zip(&mut archive, "trips.txt", &options)?;
        feed.stop_times = Self::read_csv_from_zip(&mut archive, "stop_times.txt", &options)?;

        // Calendar files (at least one required)
        if let Ok(calendars) = Self::read_csv_from_zip::<_, Calendar>(&mut archive, "calendar.txt", &options) {
            feed.calendars = calendars;
        }
        if let Ok(dates) = Self::read_csv_from_zip::<_, CalendarDate>(&mut archive, "calendar_dates.txt", &options) {
            feed.calendar_dates = dates;
        }

        // Optional files
        if let Ok(shapes) = Self::read_csv_from_zip::<_, Shape>(&mut archive, "shapes.txt", &options) {
            feed.shapes = shapes;
        }

        Ok(GtfsFeed { feed })
    }

    fn read_csv_from_zip<R: Read + std::io::Seek, T>(
        archive: &mut ZipArchive<R>,
        filename: &str,
        options: &ReadOptions,
    ) -> Result<Vec<T>, ParseError>
    where
        T: FromRaw,
        T::Raw: for<'de> serde::Deserialize<'de>,
    {
        let file = archive
            .by_name(filename)
            .map_err(|_| ParseError::MissingField(filename.to_string()))?;

        Self::parse_csv(file, options)
    }

    fn read_agencies(path: &Path, options: &ReadOptions) -> Result<Vec<Agency>, ParseError> {
        let file = File::open(path)?;
        Self::parse_csv::<_, Agency>(file, options)
    }

    fn read_stops(path: &Path, options: &ReadOptions) -> Result<Vec<Stop>, ParseError> {
        let file = File::open(path)?;
        Self::parse_csv::<_, Stop>(file, options)
    }

    fn read_routes(path: &Path, options: &ReadOptions) -> Result<Vec<Route>, ParseError> {
        let file = File::open(path)?;
        Self::parse_csv::<_, Route>(file, options)
    }

    fn read_trips(path: &Path, options: &ReadOptions) -> Result<Vec<Trip>, ParseError> {
        let file = File::open(path)?;
        Self::parse_csv::<_, Trip>(file, options)
    }

    fn read_stop_times(path: &Path, options: &ReadOptions) -> Result<Vec<StopTime>, ParseError> {
        let file = File::open(path)?;
        Self::parse_csv::<_, StopTime>(file, options)
    }

    fn read_calendars(path: &Path, options: &ReadOptions) -> Result<Vec<Calendar>, ParseError> {
        let file = File::open(path)?;
        Self::parse_csv::<_, Calendar>(file, options)
    }

    fn read_calendar_dates(
        path: &Path,
        options: &ReadOptions,
    ) -> Result<Vec<CalendarDate>, ParseError> {
        let file = File::open(path)?;
        Self::parse_csv::<_, CalendarDate>(file, options)
    }

    fn read_shapes(path: &Path, options: &ReadOptions) -> Result<Vec<Shape>, ParseError> {
        let file = File::open(path)?;
        // Shapes need special handling - group by shape_id
        let raw_shapes: Vec<RawShape> = Self::parse_csv_raw(file, options)?;

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

    fn parse_csv<R: Read, T>(reader: R, options: &ReadOptions) -> Result<Vec<T>, ParseError>
    where
        T: FromRaw,
        T::Raw: for<'de> serde::Deserialize<'de>,
    {
        let raw_records: Vec<T::Raw> = Self::parse_csv_raw(reader, options)?;
        raw_records
            .into_iter()
            .map(|raw| T::from_raw(raw))
            .collect()
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

/// Trait for converting raw CSV types to domain types.
trait FromRaw: Sized {
    type Raw;
    fn from_raw(raw: Self::Raw) -> Result<Self, ParseError>;
}

impl FromRaw for Agency {
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

impl FromRaw for Stop {
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
            location_type: raw
                .location_type
                .and_then(LocationType::from_u8)
                .unwrap_or_default(),
            parent_station: raw.parent_station,
            timezone: raw.stop_timezone,
            wheelchair_boarding: raw.wheelchair_boarding.and_then(WheelchairBoarding::from_u8),
            platform_code: raw.platform_code,
            naptan_code: None,
            atco_code: None,
        })
    }
}

impl FromRaw for Route {
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

impl FromRaw for Trip {
    type Raw = RawTrip;

    fn from_raw(raw: RawTrip) -> Result<Self, ParseError> {
        Ok(Trip {
            id: raw.trip_id,
            route_id: raw.route_id,
            service_id: raw.service_id,
            headsign: raw.trip_headsign,
            short_name: raw.trip_short_name,
            direction_id: raw
                .direction_id
                .and_then(transit_core::DirectionId::from_u8),
            block_id: raw.block_id,
            shape_id: raw.shape_id,
            wheelchair_accessible: raw
                .wheelchair_accessible
                .and_then(transit_core::WheelchairAccessible::from_u8),
            bikes_allowed: raw
                .bikes_allowed
                .and_then(transit_core::BikesAllowed::from_u8),
            vehicle_journey_code: None,
        })
    }
}

impl FromRaw for StopTime {
    type Raw = RawStopTime;

    fn from_raw(raw: RawStopTime) -> Result<Self, ParseError> {
        Ok(StopTime {
            trip_id: raw.trip_id,
            arrival_time: raw.arrival_time.as_deref().and_then(StopTime::parse_time),
            departure_time: raw.departure_time.as_deref().and_then(StopTime::parse_time),
            stop_id: raw.stop_id,
            stop_sequence: raw.stop_sequence,
            stop_headsign: raw.stop_headsign,
            pickup_type: raw
                .pickup_type
                .and_then(PickupDropoffType::from_u8)
                .unwrap_or_default(),
            drop_off_type: raw
                .drop_off_type
                .and_then(PickupDropoffType::from_u8)
                .unwrap_or_default(),
            continuous_pickup: raw.continuous_pickup,
            continuous_drop_off: raw.continuous_drop_off,
            shape_dist_traveled: raw.shape_dist_traveled,
            timepoint: raw.timepoint.and_then(Timepoint::from_u8).unwrap_or_default(),
        })
    }
}

impl FromRaw for Calendar {
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

impl FromRaw for CalendarDate {
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

impl FromRaw for Shape {
    type Raw = RawShape;

    fn from_raw(raw: RawShape) -> Result<Self, ParseError> {
        Ok(Shape {
            id: raw.shape_id,
            points: vec![ShapePoint {
                latitude: raw.shape_pt_lat,
                longitude: raw.shape_pt_lon,
                sequence: raw.shape_pt_sequence,
                dist_traveled: raw.shape_dist_traveled,
            }],
        })
    }
}

/// Parse GTFS date format (YYYYMMDD).
fn parse_gtfs_date(s: &str) -> Result<NaiveDate, ParseError> {
    NaiveDate::parse_from_str(s, "%Y%m%d")
        .map_err(|_| ParseError::InvalidDate(s.to_string()))
}

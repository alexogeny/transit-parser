//! GTFS feed writer.

use csv::WriterBuilder;
use std::fs::{self, File};
use std::io::{Cursor, Write};
use std::path::Path;
use transit_core::{
    Agency, Calendar, CalendarDate, ParseError, Route, Shape, Stop, StopTime, TransitFeed, Trip,
};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Options for writing GTFS feeds.
#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    /// Whether to include optional empty fields.
    pub include_empty_fields: bool,
}

/// GTFS feed writer.
pub struct GtfsWriter;

impl GtfsWriter {
    /// Write a GTFS feed to a directory.
    pub fn write_path(
        feed: &TransitFeed,
        path: &Path,
        _options: WriteOptions,
    ) -> Result<(), ParseError> {
        fs::create_dir_all(path)?;

        Self::write_agencies(&feed.agencies, &path.join("agency.txt"))?;
        Self::write_stops(&feed.stops, &path.join("stops.txt"))?;
        Self::write_routes(&feed.routes, &path.join("routes.txt"))?;
        Self::write_trips(&feed.trips, &path.join("trips.txt"))?;
        Self::write_stop_times(&feed.stop_times, &path.join("stop_times.txt"))?;

        if !feed.calendars.is_empty() {
            Self::write_calendars(&feed.calendars, &path.join("calendar.txt"))?;
        }

        if !feed.calendar_dates.is_empty() {
            Self::write_calendar_dates(&feed.calendar_dates, &path.join("calendar_dates.txt"))?;
        }

        if !feed.shapes.is_empty() {
            Self::write_shapes(&feed.shapes, &path.join("shapes.txt"))?;
        }

        Ok(())
    }

    /// Write a GTFS feed to a ZIP file.
    pub fn write_zip(
        feed: &TransitFeed,
        path: &Path,
        options: WriteOptions,
    ) -> Result<(), ParseError> {
        let bytes = Self::write_bytes(feed, options)?;
        fs::write(path, bytes)?;
        Ok(())
    }

    /// Write a GTFS feed to bytes (ZIP format).
    pub fn write_bytes(feed: &TransitFeed, _options: WriteOptions) -> Result<Vec<u8>, ParseError> {
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buffer);
            let file_options = SimpleFileOptions::default();

            // agency.txt
            zip.start_file("agency.txt", file_options)
                .map_err(|e| ParseError::Zip(e.to_string()))?;
            let agency_csv = Self::agencies_to_csv(&feed.agencies)?;
            zip.write_all(agency_csv.as_bytes())?;

            // stops.txt
            zip.start_file("stops.txt", file_options)
                .map_err(|e| ParseError::Zip(e.to_string()))?;
            let stops_csv = Self::stops_to_csv(&feed.stops)?;
            zip.write_all(stops_csv.as_bytes())?;

            // routes.txt
            zip.start_file("routes.txt", file_options)
                .map_err(|e| ParseError::Zip(e.to_string()))?;
            let routes_csv = Self::routes_to_csv(&feed.routes)?;
            zip.write_all(routes_csv.as_bytes())?;

            // trips.txt
            zip.start_file("trips.txt", file_options)
                .map_err(|e| ParseError::Zip(e.to_string()))?;
            let trips_csv = Self::trips_to_csv(&feed.trips)?;
            zip.write_all(trips_csv.as_bytes())?;

            // stop_times.txt
            zip.start_file("stop_times.txt", file_options)
                .map_err(|e| ParseError::Zip(e.to_string()))?;
            let stop_times_csv = Self::stop_times_to_csv(&feed.stop_times)?;
            zip.write_all(stop_times_csv.as_bytes())?;

            // calendar.txt
            if !feed.calendars.is_empty() {
                zip.start_file("calendar.txt", file_options)
                    .map_err(|e| ParseError::Zip(e.to_string()))?;
                let calendars_csv = Self::calendars_to_csv(&feed.calendars)?;
                zip.write_all(calendars_csv.as_bytes())?;
            }

            // calendar_dates.txt
            if !feed.calendar_dates.is_empty() {
                zip.start_file("calendar_dates.txt", file_options)
                    .map_err(|e| ParseError::Zip(e.to_string()))?;
                let calendar_dates_csv = Self::calendar_dates_to_csv(&feed.calendar_dates)?;
                zip.write_all(calendar_dates_csv.as_bytes())?;
            }

            // shapes.txt
            if !feed.shapes.is_empty() {
                zip.start_file("shapes.txt", file_options)
                    .map_err(|e| ParseError::Zip(e.to_string()))?;
                let shapes_csv = Self::shapes_to_csv(&feed.shapes)?;
                zip.write_all(shapes_csv.as_bytes())?;
            }

            zip.finish().map_err(|e| ParseError::Zip(e.to_string()))?;
        }

        Ok(buffer.into_inner())
    }

    fn write_agencies(agencies: &[Agency], path: &Path) -> Result<(), ParseError> {
        let csv = Self::agencies_to_csv(agencies)?;
        fs::write(path, csv)?;
        Ok(())
    }

    fn agencies_to_csv(agencies: &[Agency]) -> Result<String, ParseError> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        wtr.write_record([
            "agency_id",
            "agency_name",
            "agency_url",
            "agency_timezone",
            "agency_lang",
            "agency_phone",
            "agency_fare_url",
            "agency_email",
        ])
        .map_err(|e| ParseError::Csv(e.to_string()))?;

        for agency in agencies {
            wtr.write_record([
                agency.id.as_deref().unwrap_or(""),
                &agency.name,
                &agency.url,
                &agency.timezone,
                agency.lang.as_deref().unwrap_or(""),
                agency.phone.as_deref().unwrap_or(""),
                agency.fare_url.as_deref().unwrap_or(""),
                agency.email.as_deref().unwrap_or(""),
            ])
            .map_err(|e| ParseError::Csv(e.to_string()))?;
        }

        let data = wtr.into_inner().map_err(|e| ParseError::Csv(e.to_string()))?;
        String::from_utf8(data).map_err(|e| ParseError::Csv(e.to_string()))
    }

    fn write_stops(stops: &[Stop], path: &Path) -> Result<(), ParseError> {
        let csv = Self::stops_to_csv(stops)?;
        fs::write(path, csv)?;
        Ok(())
    }

    fn stops_to_csv(stops: &[Stop]) -> Result<String, ParseError> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        wtr.write_record([
            "stop_id",
            "stop_code",
            "stop_name",
            "stop_desc",
            "stop_lat",
            "stop_lon",
            "zone_id",
            "stop_url",
            "location_type",
            "parent_station",
            "stop_timezone",
            "wheelchair_boarding",
            "platform_code",
        ])
        .map_err(|e| ParseError::Csv(e.to_string()))?;

        for stop in stops {
            wtr.write_record([
                &stop.id,
                stop.code.as_deref().unwrap_or(""),
                &stop.name,
                stop.description.as_deref().unwrap_or(""),
                &stop.latitude.to_string(),
                &stop.longitude.to_string(),
                stop.zone_id.as_deref().unwrap_or(""),
                stop.url.as_deref().unwrap_or(""),
                &(stop.location_type as u8).to_string(),
                stop.parent_station.as_deref().unwrap_or(""),
                stop.timezone.as_deref().unwrap_or(""),
                &stop
                    .wheelchair_boarding
                    .map(|w| (w as u8).to_string())
                    .unwrap_or_default(),
                stop.platform_code.as_deref().unwrap_or(""),
            ])
            .map_err(|e| ParseError::Csv(e.to_string()))?;
        }

        let data = wtr.into_inner().map_err(|e| ParseError::Csv(e.to_string()))?;
        String::from_utf8(data).map_err(|e| ParseError::Csv(e.to_string()))
    }

    fn write_routes(routes: &[Route], path: &Path) -> Result<(), ParseError> {
        let csv = Self::routes_to_csv(routes)?;
        fs::write(path, csv)?;
        Ok(())
    }

    fn routes_to_csv(routes: &[Route]) -> Result<String, ParseError> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        wtr.write_record([
            "route_id",
            "agency_id",
            "route_short_name",
            "route_long_name",
            "route_desc",
            "route_type",
            "route_url",
            "route_color",
            "route_text_color",
            "route_sort_order",
        ])
        .map_err(|e| ParseError::Csv(e.to_string()))?;

        for route in routes {
            wtr.write_record([
                &route.id,
                route.agency_id.as_deref().unwrap_or(""),
                route.short_name.as_deref().unwrap_or(""),
                route.long_name.as_deref().unwrap_or(""),
                route.description.as_deref().unwrap_or(""),
                &route.route_type.as_u16().to_string(),
                route.url.as_deref().unwrap_or(""),
                route.color.as_deref().unwrap_or(""),
                route.text_color.as_deref().unwrap_or(""),
                &route
                    .sort_order
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            ])
            .map_err(|e| ParseError::Csv(e.to_string()))?;
        }

        let data = wtr.into_inner().map_err(|e| ParseError::Csv(e.to_string()))?;
        String::from_utf8(data).map_err(|e| ParseError::Csv(e.to_string()))
    }

    fn write_trips(trips: &[Trip], path: &Path) -> Result<(), ParseError> {
        let csv = Self::trips_to_csv(trips)?;
        fs::write(path, csv)?;
        Ok(())
    }

    fn trips_to_csv(trips: &[Trip]) -> Result<String, ParseError> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        wtr.write_record([
            "route_id",
            "service_id",
            "trip_id",
            "trip_headsign",
            "trip_short_name",
            "direction_id",
            "block_id",
            "shape_id",
            "wheelchair_accessible",
            "bikes_allowed",
        ])
        .map_err(|e| ParseError::Csv(e.to_string()))?;

        for trip in trips {
            wtr.write_record([
                &trip.route_id,
                &trip.service_id,
                &trip.id,
                trip.headsign.as_deref().unwrap_or(""),
                trip.short_name.as_deref().unwrap_or(""),
                &trip
                    .direction_id
                    .map(|d| (d as u8).to_string())
                    .unwrap_or_default(),
                trip.block_id.as_deref().unwrap_or(""),
                trip.shape_id.as_deref().unwrap_or(""),
                &trip
                    .wheelchair_accessible
                    .map(|w| (w as u8).to_string())
                    .unwrap_or_default(),
                &trip
                    .bikes_allowed
                    .map(|b| (b as u8).to_string())
                    .unwrap_or_default(),
            ])
            .map_err(|e| ParseError::Csv(e.to_string()))?;
        }

        let data = wtr.into_inner().map_err(|e| ParseError::Csv(e.to_string()))?;
        String::from_utf8(data).map_err(|e| ParseError::Csv(e.to_string()))
    }

    fn write_stop_times(stop_times: &[StopTime], path: &Path) -> Result<(), ParseError> {
        let csv = Self::stop_times_to_csv(stop_times)?;
        fs::write(path, csv)?;
        Ok(())
    }

    fn stop_times_to_csv(stop_times: &[StopTime]) -> Result<String, ParseError> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        wtr.write_record([
            "trip_id",
            "arrival_time",
            "departure_time",
            "stop_id",
            "stop_sequence",
            "stop_headsign",
            "pickup_type",
            "drop_off_type",
            "shape_dist_traveled",
            "timepoint",
        ])
        .map_err(|e| ParseError::Csv(e.to_string()))?;

        for st in stop_times {
            wtr.write_record([
                &st.trip_id,
                &st.arrival_time
                    .map(StopTime::format_time)
                    .unwrap_or_default(),
                &st.departure_time
                    .map(StopTime::format_time)
                    .unwrap_or_default(),
                &st.stop_id,
                &st.stop_sequence.to_string(),
                st.stop_headsign.as_deref().unwrap_or(""),
                &(st.pickup_type as u8).to_string(),
                &(st.drop_off_type as u8).to_string(),
                &st.shape_dist_traveled
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
                &(st.timepoint as u8).to_string(),
            ])
            .map_err(|e| ParseError::Csv(e.to_string()))?;
        }

        let data = wtr.into_inner().map_err(|e| ParseError::Csv(e.to_string()))?;
        String::from_utf8(data).map_err(|e| ParseError::Csv(e.to_string()))
    }

    fn write_calendars(calendars: &[Calendar], path: &Path) -> Result<(), ParseError> {
        let csv = Self::calendars_to_csv(calendars)?;
        fs::write(path, csv)?;
        Ok(())
    }

    fn calendars_to_csv(calendars: &[Calendar]) -> Result<String, ParseError> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        wtr.write_record([
            "service_id",
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
            "start_date",
            "end_date",
        ])
        .map_err(|e| ParseError::Csv(e.to_string()))?;

        for cal in calendars {
            wtr.write_record([
                &cal.service_id,
                &(cal.monday as u8).to_string(),
                &(cal.tuesday as u8).to_string(),
                &(cal.wednesday as u8).to_string(),
                &(cal.thursday as u8).to_string(),
                &(cal.friday as u8).to_string(),
                &(cal.saturday as u8).to_string(),
                &(cal.sunday as u8).to_string(),
                &cal.start_date.format("%Y%m%d").to_string(),
                &cal.end_date.format("%Y%m%d").to_string(),
            ])
            .map_err(|e| ParseError::Csv(e.to_string()))?;
        }

        let data = wtr.into_inner().map_err(|e| ParseError::Csv(e.to_string()))?;
        String::from_utf8(data).map_err(|e| ParseError::Csv(e.to_string()))
    }

    fn write_calendar_dates(
        calendar_dates: &[CalendarDate],
        path: &Path,
    ) -> Result<(), ParseError> {
        let csv = Self::calendar_dates_to_csv(calendar_dates)?;
        fs::write(path, csv)?;
        Ok(())
    }

    fn calendar_dates_to_csv(calendar_dates: &[CalendarDate]) -> Result<String, ParseError> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        wtr.write_record(["service_id", "date", "exception_type"])
            .map_err(|e| ParseError::Csv(e.to_string()))?;

        for cd in calendar_dates {
            wtr.write_record([
                &cd.service_id,
                &cd.date.format("%Y%m%d").to_string(),
                &(cd.exception_type as u8).to_string(),
            ])
            .map_err(|e| ParseError::Csv(e.to_string()))?;
        }

        let data = wtr.into_inner().map_err(|e| ParseError::Csv(e.to_string()))?;
        String::from_utf8(data).map_err(|e| ParseError::Csv(e.to_string()))
    }

    fn write_shapes(shapes: &[Shape], path: &Path) -> Result<(), ParseError> {
        let csv = Self::shapes_to_csv(shapes)?;
        fs::write(path, csv)?;
        Ok(())
    }

    fn shapes_to_csv(shapes: &[Shape]) -> Result<String, ParseError> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        wtr.write_record([
            "shape_id",
            "shape_pt_lat",
            "shape_pt_lon",
            "shape_pt_sequence",
            "shape_dist_traveled",
        ])
        .map_err(|e| ParseError::Csv(e.to_string()))?;

        for shape in shapes {
            for point in &shape.points {
                wtr.write_record([
                    &shape.id,
                    &point.latitude.to_string(),
                    &point.longitude.to_string(),
                    &point.sequence.to_string(),
                    &point
                        .dist_traveled
                        .map(|d| d.to_string())
                        .unwrap_or_default(),
                ])
                .map_err(|e| ParseError::Csv(e.to_string()))?;
            }
        }

        let data = wtr.into_inner().map_err(|e| ParseError::Csv(e.to_string()))?;
        String::from_utf8(data).map_err(|e| ParseError::Csv(e.to_string()))
    }
}

//! Python-exposed transit data models.

use pyo3::prelude::*;
use transit_core::{Agency, Calendar, CalendarDate, Route, Shape, Stop, StopTime, Trip};

/// Python wrapper for Agency.
#[pyclass(name = "Agency")]
#[derive(Clone)]
pub struct PyAgency {
    pub inner: Agency,
}

#[pymethods]
impl PyAgency {
    #[new]
    #[pyo3(signature = (name, url, timezone, id=None))]
    fn new(name: String, url: String, timezone: String, id: Option<String>) -> Self {
        let mut agency = Agency::new(name, url, timezone);
        agency.id = id;
        Self { inner: agency }
    }

    #[getter]
    fn id(&self) -> Option<String> {
        self.inner.id.clone()
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn url(&self) -> String {
        self.inner.url.clone()
    }

    #[getter]
    fn timezone(&self) -> String {
        self.inner.timezone.clone()
    }

    fn __repr__(&self) -> String {
        format!("Agency(id={:?}, name={:?})", self.inner.id, self.inner.name)
    }
}

/// Python wrapper for Stop.
#[pyclass(name = "Stop")]
#[derive(Clone)]
pub struct PyStop {
    pub inner: Stop,
}

#[pymethods]
impl PyStop {
    #[new]
    fn new(id: String, name: String, latitude: f64, longitude: f64) -> Self {
        Self {
            inner: Stop::new(id, name, latitude, longitude),
        }
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn latitude(&self) -> f64 {
        self.inner.latitude
    }

    #[getter]
    fn longitude(&self) -> f64 {
        self.inner.longitude
    }

    #[getter]
    fn code(&self) -> Option<String> {
        self.inner.code.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "Stop(id={:?}, name={:?}, lat={}, lon={})",
            self.inner.id, self.inner.name, self.inner.latitude, self.inner.longitude
        )
    }
}

/// Python wrapper for Route.
#[pyclass(name = "Route")]
#[derive(Clone)]
pub struct PyRoute {
    pub inner: Route,
}

#[pymethods]
impl PyRoute {
    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    fn short_name(&self) -> Option<String> {
        self.inner.short_name.clone()
    }

    #[getter]
    fn long_name(&self) -> Option<String> {
        self.inner.long_name.clone()
    }

    #[getter]
    fn route_type(&self) -> u16 {
        self.inner.route_type as u16
    }

    fn __repr__(&self) -> String {
        format!(
            "Route(id={:?}, short_name={:?})",
            self.inner.id, self.inner.short_name
        )
    }
}

/// Python wrapper for Trip.
#[pyclass(name = "Trip")]
#[derive(Clone)]
pub struct PyTrip {
    pub inner: Trip,
}

#[pymethods]
impl PyTrip {
    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    fn route_id(&self) -> String {
        self.inner.route_id.clone()
    }

    #[getter]
    fn service_id(&self) -> String {
        self.inner.service_id.clone()
    }

    #[getter]
    fn headsign(&self) -> Option<String> {
        self.inner.headsign.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "Trip(id={:?}, route_id={:?})",
            self.inner.id, self.inner.route_id
        )
    }
}

/// Python wrapper for StopTime.
#[pyclass(name = "StopTime")]
#[derive(Clone)]
pub struct PyStopTime {
    pub inner: StopTime,
}

#[pymethods]
impl PyStopTime {
    #[getter]
    fn trip_id(&self) -> String {
        self.inner.trip_id.clone()
    }

    #[getter]
    fn stop_id(&self) -> String {
        self.inner.stop_id.clone()
    }

    #[getter]
    fn stop_sequence(&self) -> u32 {
        self.inner.stop_sequence
    }

    #[getter]
    fn arrival_time(&self) -> Option<String> {
        self.inner.arrival_time.map(StopTime::format_time)
    }

    #[getter]
    fn departure_time(&self) -> Option<String> {
        self.inner.departure_time.map(StopTime::format_time)
    }

    fn __repr__(&self) -> String {
        format!(
            "StopTime(trip_id={:?}, stop_id={:?}, seq={})",
            self.inner.trip_id, self.inner.stop_id, self.inner.stop_sequence
        )
    }
}

/// Python wrapper for Calendar.
#[pyclass(name = "Calendar")]
#[derive(Clone)]
pub struct PyCalendar {
    pub inner: Calendar,
}

#[pymethods]
impl PyCalendar {
    #[getter]
    fn service_id(&self) -> String {
        self.inner.service_id.clone()
    }

    #[getter]
    fn monday(&self) -> bool {
        self.inner.monday.is_available()
    }

    #[getter]
    fn tuesday(&self) -> bool {
        self.inner.tuesday.is_available()
    }

    #[getter]
    fn wednesday(&self) -> bool {
        self.inner.wednesday.is_available()
    }

    #[getter]
    fn thursday(&self) -> bool {
        self.inner.thursday.is_available()
    }

    #[getter]
    fn friday(&self) -> bool {
        self.inner.friday.is_available()
    }

    #[getter]
    fn saturday(&self) -> bool {
        self.inner.saturday.is_available()
    }

    #[getter]
    fn sunday(&self) -> bool {
        self.inner.sunday.is_available()
    }

    #[getter]
    fn start_date(&self) -> String {
        self.inner.start_date.format("%Y%m%d").to_string()
    }

    #[getter]
    fn end_date(&self) -> String {
        self.inner.end_date.format("%Y%m%d").to_string()
    }

    fn __repr__(&self) -> String {
        format!("Calendar(service_id={:?})", self.inner.service_id)
    }
}

/// Python wrapper for CalendarDate.
#[pyclass(name = "CalendarDate")]
#[derive(Clone)]
pub struct PyCalendarDate {
    pub inner: CalendarDate,
}

#[pymethods]
impl PyCalendarDate {
    #[getter]
    fn service_id(&self) -> String {
        self.inner.service_id.clone()
    }

    #[getter]
    fn date(&self) -> String {
        self.inner.date.format("%Y%m%d").to_string()
    }

    #[getter]
    fn exception_type(&self) -> u8 {
        self.inner.exception_type as u8
    }

    fn __repr__(&self) -> String {
        format!(
            "CalendarDate(service_id={:?}, date={:?})",
            self.inner.service_id, self.inner.date
        )
    }
}

/// Python wrapper for Shape.
#[pyclass(name = "Shape")]
#[derive(Clone)]
pub struct PyShape {
    pub inner: Shape,
}

#[pymethods]
impl PyShape {
    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    fn points(&self) -> Vec<(f64, f64, u32)> {
        self.inner
            .points
            .iter()
            .map(|p| (p.latitude, p.longitude, p.sequence))
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "Shape(id={:?}, points={})",
            self.inner.id,
            self.inner.points.len()
        )
    }
}

// Conversion helpers
impl From<Agency> for PyAgency {
    fn from(inner: Agency) -> Self {
        Self { inner }
    }
}

impl From<Stop> for PyStop {
    fn from(inner: Stop) -> Self {
        Self { inner }
    }
}

impl From<Route> for PyRoute {
    fn from(inner: Route) -> Self {
        Self { inner }
    }
}

impl From<Trip> for PyTrip {
    fn from(inner: Trip) -> Self {
        Self { inner }
    }
}

impl From<StopTime> for PyStopTime {
    fn from(inner: StopTime) -> Self {
        Self { inner }
    }
}

impl From<Calendar> for PyCalendar {
    fn from(inner: Calendar) -> Self {
        Self { inner }
    }
}

impl From<CalendarDate> for PyCalendarDate {
    fn from(inner: CalendarDate) -> Self {
        Self { inner }
    }
}

impl From<Shape> for PyShape {
    fn from(inner: Shape) -> Self {
        Self { inner }
    }
}

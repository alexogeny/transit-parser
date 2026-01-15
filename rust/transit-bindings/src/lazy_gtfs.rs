//! Lazy GTFS Python bindings.

use crate::gtfs::PyGtfsFeed;
use crate::models::*;
use gtfs_parser::LazyGtfsFeed;
use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use std::sync::Arc;

/// Python wrapper for lazy-loading GTFS feed.
///
/// This provides fast initial load times by deferring CSV parsing
/// until first access. Similar to partridge's approach.
#[pyclass(name = "LazyGtfsFeed")]
pub struct PyLazyGtfsFeed {
    inner: Arc<LazyGtfsFeed>,
    // Cached Python conversions
    agencies_cache: Option<Py<pyo3::types::PyList>>,
    stops_cache: Option<Py<pyo3::types::PyList>>,
    routes_cache: Option<Py<pyo3::types::PyList>>,
    trips_cache: Option<Py<pyo3::types::PyList>>,
    stop_times_cache: Option<Py<pyo3::types::PyList>>,
    calendars_cache: Option<Py<pyo3::types::PyList>>,
    calendar_dates_cache: Option<Py<pyo3::types::PyList>>,
    shapes_cache: Option<Py<pyo3::types::PyList>>,
}

#[pymethods]
impl PyLazyGtfsFeed {
    /// Load a lazy GTFS feed from a directory path.
    ///
    /// This only scans the directory structure - no CSV parsing happens until
    /// you access a specific property like `stops` or `stop_times`.
    #[staticmethod]
    fn from_path(path: &str) -> PyResult<Self> {
        LazyGtfsFeed::from_path(path)
            .map(Self::new_from_inner)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Load a lazy GTFS feed from a ZIP file.
    ///
    /// This reads the ZIP into memory and scans entries - no CSV parsing
    /// happens until you access a specific property.
    #[staticmethod]
    fn from_zip(path: &str) -> PyResult<Self> {
        LazyGtfsFeed::from_zip(path)
            .map(Self::new_from_inner)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Load a lazy GTFS feed from bytes (ZIP format).
    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        LazyGtfsFeed::from_bytes(data.to_vec())
            .map(Self::new_from_inner)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    // ========================================
    // Fast count properties (no full parse)
    // ========================================

    /// Number of agencies (fast, counts CSV rows without full parse).
    #[getter]
    fn agency_count(&self) -> PyResult<usize> {
        self.inner
            .agency_count()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Number of stops (fast, counts CSV rows without full parse).
    #[getter]
    fn stop_count(&self) -> PyResult<usize> {
        self.inner
            .stop_count()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Number of routes (fast, counts CSV rows without full parse).
    #[getter]
    fn route_count(&self) -> PyResult<usize> {
        self.inner
            .route_count()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Number of trips (fast, counts CSV rows without full parse).
    #[getter]
    fn trip_count(&self) -> PyResult<usize> {
        self.inner
            .trip_count()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Number of stop_times (fast, counts CSV rows without full parse).
    #[getter]
    fn stop_time_count(&self) -> PyResult<usize> {
        self.inner
            .stop_time_count()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Number of calendars (fast, counts CSV rows without full parse).
    #[getter]
    fn calendar_count(&self) -> PyResult<usize> {
        self.inner
            .calendar_count()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Number of calendar dates (fast, counts CSV rows without full parse).
    #[getter]
    fn calendar_date_count(&self) -> PyResult<usize> {
        self.inner
            .calendar_date_count()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Number of shapes (requires full parse of shapes.txt).
    #[getter]
    fn shape_count(&self) -> PyResult<usize> {
        self.inner
            .shape_count()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    // ========================================
    // Data accessors (lazy parse + cache)
    // ========================================

    /// Get all agencies (parses agency.txt on first access).
    #[getter]
    fn agencies(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.agencies_cache {
            return Ok(cached.clone_ref(py));
        }

        let agencies = self
            .inner
            .agencies()
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        let list = pyo3::types::PyList::new(py, agencies.into_iter().map(PyAgency::from))?;
        let cached = list.into();
        self.agencies_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all stops (parses stops.txt on first access).
    #[getter]
    fn stops(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.stops_cache {
            return Ok(cached.clone_ref(py));
        }

        let stops = self
            .inner
            .stops()
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        let list = pyo3::types::PyList::new(py, stops.into_iter().map(PyStop::from))?;
        let cached = list.into();
        self.stops_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all routes (parses routes.txt on first access).
    #[getter]
    fn routes(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.routes_cache {
            return Ok(cached.clone_ref(py));
        }

        let routes = self
            .inner
            .routes()
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        let list = pyo3::types::PyList::new(py, routes.into_iter().map(PyRoute::from))?;
        let cached = list.into();
        self.routes_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all trips (parses trips.txt on first access).
    #[getter]
    fn trips(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.trips_cache {
            return Ok(cached.clone_ref(py));
        }

        let trips = self
            .inner
            .trips()
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        let list = pyo3::types::PyList::new(py, trips.into_iter().map(PyTrip::from))?;
        let cached = list.into();
        self.trips_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all stop_times (parses stop_times.txt on first access).
    #[getter]
    fn stop_times(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.stop_times_cache {
            return Ok(cached.clone_ref(py));
        }

        let stop_times = self
            .inner
            .stop_times()
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        let list = pyo3::types::PyList::new(py, stop_times.into_iter().map(PyStopTime::from))?;
        let cached = list.into();
        self.stop_times_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all calendars (parses calendar.txt on first access).
    #[getter]
    fn calendars(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.calendars_cache {
            return Ok(cached.clone_ref(py));
        }

        let calendars = self
            .inner
            .calendars()
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        let list = pyo3::types::PyList::new(py, calendars.into_iter().map(PyCalendar::from))?;
        let cached = list.into();
        self.calendars_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all calendar_dates (parses calendar_dates.txt on first access).
    #[getter]
    fn calendar_dates(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.calendar_dates_cache {
            return Ok(cached.clone_ref(py));
        }

        let dates = self
            .inner
            .calendar_dates()
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        let list = pyo3::types::PyList::new(py, dates.into_iter().map(PyCalendarDate::from))?;
        let cached = list.into();
        self.calendar_dates_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all shapes (parses shapes.txt on first access).
    #[getter]
    fn shapes(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.shapes_cache {
            return Ok(cached.clone_ref(py));
        }

        let shapes = self
            .inner
            .shapes()
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        let list = pyo3::types::PyList::new(py, shapes.into_iter().map(PyShape::from))?;
        let cached = list.into();
        self.shapes_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Materialize the lazy feed into a regular GtfsFeed.
    ///
    /// This parses all files and returns a fully-loaded GtfsFeed.
    fn materialize(&self) -> PyResult<PyGtfsFeed> {
        let feed = self
            .inner
            .materialize()
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        Ok(PyGtfsFeed::from_inner(gtfs_parser::GtfsFeed { feed }))
    }

    fn __repr__(&self) -> String {
        // Use counts that don't require full parse
        let agency_count = self.inner.agency_count().unwrap_or(0);
        let stop_count = self.inner.stop_count().unwrap_or(0);
        let route_count = self.inner.route_count().unwrap_or(0);
        let trip_count = self.inner.trip_count().unwrap_or(0);
        let stop_time_count = self.inner.stop_time_count().unwrap_or(0);

        format!(
            "LazyGtfsFeed(agencies~{}, stops~{}, routes~{}, trips~{}, stop_times~{})",
            agency_count, stop_count, route_count, trip_count, stop_time_count
        )
    }
}

impl PyLazyGtfsFeed {
    fn new_from_inner(inner: LazyGtfsFeed) -> Self {
        Self {
            inner: Arc::new(inner),
            agencies_cache: None,
            stops_cache: None,
            routes_cache: None,
            trips_cache: None,
            stop_times_cache: None,
            calendars_cache: None,
            calendar_dates_cache: None,
            shapes_cache: None,
        }
    }
}

//! GTFS Python bindings.

use crate::models::*;
use gtfs_parser::GtfsFeed;
use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use std::sync::Arc;

/// Python wrapper for GTFS feed.
///
/// Uses caching to avoid re-converting Rust data to Python on each access.
#[pyclass(name = "GtfsFeed")]
pub struct PyGtfsFeed {
    pub(crate) inner: GtfsFeed,
    // Cached Python conversions (lazily populated)
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
impl PyGtfsFeed {
    /// Create a new empty GTFS feed.
    #[new]
    fn new() -> Self {
        Self {
            inner: GtfsFeed::new(),
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

    /// Load a GTFS feed from a directory path.
    #[staticmethod]
    fn from_path(path: &str) -> PyResult<Self> {
        GtfsFeed::from_path(path)
            .map(|inner| Self::from_inner(inner))
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Load a GTFS feed from a ZIP file.
    #[staticmethod]
    fn from_zip(path: &str) -> PyResult<Self> {
        GtfsFeed::from_zip(path)
            .map(|inner| Self::from_inner(inner))
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Load a GTFS feed from bytes (ZIP format).
    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        GtfsFeed::from_bytes(data)
            .map(|inner| Self::from_inner(inner))
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Write the GTFS feed to a directory.
    fn to_path(&self, path: &str) -> PyResult<()> {
        self.inner
            .to_path(path)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Write the GTFS feed to a ZIP file.
    fn to_zip(&self, path: &str) -> PyResult<()> {
        self.inner
            .to_zip(path)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Export the GTFS feed as ZIP bytes.
    fn to_bytes(&self) -> PyResult<Vec<u8>> {
        self.inner
            .to_bytes()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    // ========================================
    // Count properties (fast, no conversion)
    // ========================================

    /// Number of agencies.
    #[getter]
    fn agency_count(&self) -> usize {
        self.inner.feed.agencies.len()
    }

    /// Number of stops.
    #[getter]
    fn stop_count(&self) -> usize {
        self.inner.feed.stops.len()
    }

    /// Number of routes.
    #[getter]
    fn route_count(&self) -> usize {
        self.inner.feed.routes.len()
    }

    /// Number of trips.
    #[getter]
    fn trip_count(&self) -> usize {
        self.inner.feed.trips.len()
    }

    /// Number of stop times.
    #[getter]
    fn stop_time_count(&self) -> usize {
        self.inner.feed.stop_times.len()
    }

    /// Number of calendars.
    #[getter]
    fn calendar_count(&self) -> usize {
        self.inner.feed.calendars.len()
    }

    /// Number of calendar dates.
    #[getter]
    fn calendar_date_count(&self) -> usize {
        self.inner.feed.calendar_dates.len()
    }

    /// Number of shapes.
    #[getter]
    fn shape_count(&self) -> usize {
        self.inner.feed.shapes.len()
    }

    // ========================================
    // Data accessors (cached conversion)
    // ========================================

    /// Get all agencies.
    #[getter]
    fn agencies(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.agencies_cache {
            return Ok(cached.clone_ref(py));
        }
        let list = pyo3::types::PyList::new(
            py,
            self.inner.feed.agencies.iter().cloned().map(PyAgency::from),
        )?;
        let cached = list.into();
        self.agencies_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all stops.
    #[getter]
    fn stops(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.stops_cache {
            return Ok(cached.clone_ref(py));
        }
        let list = pyo3::types::PyList::new(
            py,
            self.inner.feed.stops.iter().cloned().map(PyStop::from),
        )?;
        let cached = list.into();
        self.stops_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all routes.
    #[getter]
    fn routes(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.routes_cache {
            return Ok(cached.clone_ref(py));
        }
        let list = pyo3::types::PyList::new(
            py,
            self.inner.feed.routes.iter().cloned().map(PyRoute::from),
        )?;
        let cached = list.into();
        self.routes_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all trips.
    #[getter]
    fn trips(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.trips_cache {
            return Ok(cached.clone_ref(py));
        }
        let list = pyo3::types::PyList::new(
            py,
            self.inner.feed.trips.iter().cloned().map(PyTrip::from),
        )?;
        let cached = list.into();
        self.trips_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all stop times.
    #[getter]
    fn stop_times(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.stop_times_cache {
            return Ok(cached.clone_ref(py));
        }
        let list = pyo3::types::PyList::new(
            py,
            self.inner.feed.stop_times.iter().cloned().map(PyStopTime::from),
        )?;
        let cached = list.into();
        self.stop_times_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all calendars.
    #[getter]
    fn calendars(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.calendars_cache {
            return Ok(cached.clone_ref(py));
        }
        let list = pyo3::types::PyList::new(
            py,
            self.inner.feed.calendars.iter().cloned().map(PyCalendar::from),
        )?;
        let cached = list.into();
        self.calendars_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all calendar dates.
    #[getter]
    fn calendar_dates(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.calendar_dates_cache {
            return Ok(cached.clone_ref(py));
        }
        let list = pyo3::types::PyList::new(
            py,
            self.inner.feed.calendar_dates.iter().cloned().map(PyCalendarDate::from),
        )?;
        let cached = list.into();
        self.calendar_dates_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    /// Get all shapes.
    #[getter]
    fn shapes(&mut self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        if let Some(ref cached) = self.shapes_cache {
            return Ok(cached.clone_ref(py));
        }
        let list = pyo3::types::PyList::new(
            py,
            self.inner.feed.shapes.iter().cloned().map(PyShape::from),
        )?;
        let cached = list.into();
        self.shapes_cache = Some(Py::clone_ref(&cached, py));
        Ok(cached)
    }

    fn __repr__(&self) -> String {
        format!(
            "GtfsFeed(agencies={}, stops={}, routes={}, trips={}, stop_times={})",
            self.inner.feed.agencies.len(),
            self.inner.feed.stops.len(),
            self.inner.feed.routes.len(),
            self.inner.feed.trips.len(),
            self.inner.feed.stop_times.len()
        )
    }
}

impl PyGtfsFeed {
    pub fn from_inner(inner: GtfsFeed) -> Self {
        Self {
            inner,
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

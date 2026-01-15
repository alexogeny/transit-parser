//! Adapter Python bindings.

use crate::gtfs::PyGtfsFeed;
use crate::txc::PyTxcDocument;
use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use txc_gtfs_adapter::{ConversionOptions, ConversionStats, TxcToGtfsConverter, UkRegion};

/// Python wrapper for conversion options.
#[pyclass(name = "ConversionOptions")]
#[derive(Clone)]
pub struct PyConversionOptions {
    inner: ConversionOptions,
}

#[pymethods]
impl PyConversionOptions {
    #[new]
    #[pyo3(signature = (
        include_shapes=false,
        calendar_start=None,
        calendar_end=None,
        region="england",
        default_timezone="Europe/London",
        default_agency_url="https://example.com"
    ))]
    fn new(
        include_shapes: bool,
        calendar_start: Option<String>,
        calendar_end: Option<String>,
        region: &str,
        default_timezone: &str,
        default_agency_url: &str,
    ) -> PyResult<Self> {
        let start = calendar_start
            .map(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d"))
            .transpose()
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid start date: {}", e)))?;

        let end = calendar_end
            .map(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d"))
            .transpose()
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid end date: {}", e)))?;

        let uk_region = match region.to_lowercase().as_str() {
            "england" => UkRegion::England,
            "scotland" => UkRegion::Scotland,
            "wales" => UkRegion::Wales,
            "northern_ireland" | "northernireland" => UkRegion::NorthernIreland,
            _ => UkRegion::England,
        };

        Ok(Self {
            inner: ConversionOptions {
                include_shapes,
                calendar_start: start,
                calendar_end: end,
                region: uk_region,
                default_timezone: default_timezone.to_string(),
                default_agency_url: default_agency_url.to_string(),
            },
        })
    }

    #[getter]
    fn include_shapes(&self) -> bool {
        self.inner.include_shapes
    }

    fn __repr__(&self) -> String {
        format!(
            "ConversionOptions(include_shapes={}, region={:?})",
            self.inner.include_shapes, self.inner.region
        )
    }
}

/// Python wrapper for conversion result.
#[pyclass(name = "ConversionResult")]
pub struct PyConversionResult {
    feed: PyGtfsFeed,
    warnings: Vec<String>,
    stats: PyConversionStats,
}

#[pymethods]
impl PyConversionResult {
    #[getter]
    fn feed(&self) -> PyGtfsFeed {
        // Clone is needed since we can't move out of self
        PyGtfsFeed::from_inner(gtfs_parser::GtfsFeed {
            feed: self.feed.inner.feed.clone(),
        })
    }

    #[getter]
    fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }

    #[getter]
    fn stats(&self) -> PyConversionStats {
        self.stats.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "ConversionResult(trips={}, warnings={})",
            self.stats.trips_converted(),
            self.warnings.len()
        )
    }
}

/// Python wrapper for conversion stats.
#[pyclass(name = "ConversionStats")]
#[derive(Clone)]
pub struct PyConversionStats {
    inner: ConversionStats,
}

#[pymethods]
impl PyConversionStats {
    #[getter]
    fn agencies_converted(&self) -> usize {
        self.inner.agencies_converted
    }

    #[getter]
    fn stops_converted(&self) -> usize {
        self.inner.stops_converted
    }

    #[getter]
    fn routes_converted(&self) -> usize {
        self.inner.routes_converted
    }

    #[getter]
    fn trips_converted(&self) -> usize {
        self.inner.trips_converted
    }

    #[getter]
    fn stop_times_generated(&self) -> usize {
        self.inner.stop_times_generated
    }

    #[getter]
    fn calendar_entries(&self) -> usize {
        self.inner.calendar_entries
    }

    #[getter]
    fn calendar_exceptions(&self) -> usize {
        self.inner.calendar_exceptions
    }

    #[getter]
    fn shapes_generated(&self) -> usize {
        self.inner.shapes_generated
    }

    fn __repr__(&self) -> String {
        format!(
            "ConversionStats(agencies={}, stops={}, routes={}, trips={})",
            self.inner.agencies_converted,
            self.inner.stops_converted,
            self.inner.routes_converted,
            self.inner.trips_converted
        )
    }
}

/// TXC to GTFS converter.
#[pyclass(name = "TxcToGtfsConverter")]
pub struct PyTxcToGtfsConverter {
    inner: TxcToGtfsConverter,
}

#[pymethods]
impl PyTxcToGtfsConverter {
    #[new]
    #[pyo3(signature = (options=None))]
    fn new(options: Option<PyConversionOptions>) -> Self {
        let opts = options.map(|o| o.inner).unwrap_or_default();
        Self {
            inner: TxcToGtfsConverter::new(opts),
        }
    }

    /// Convert a TXC document to GTFS.
    fn convert(&self, py: Python<'_>, document: &PyTxcDocument) -> PyResult<PyConversionResult> {
        // Release GIL during conversion
        let doc = document.inner.clone();
        let result = py.allow_threads(|| self.inner.convert(doc));

        match result {
            Ok(r) => Ok(PyConversionResult {
                feed: PyGtfsFeed::from_inner(r.feed),
                warnings: r.warnings.iter().map(|w| w.message.clone()).collect(),
                stats: PyConversionStats { inner: r.stats },
            }),
            Err(e) => Err(PyRuntimeError::new_err(e.to_string())),
        }
    }

    /// Convert multiple TXC documents to a single GTFS feed.
    fn convert_batch(
        &self,
        py: Python<'_>,
        documents: Vec<PyRef<PyTxcDocument>>,
    ) -> PyResult<PyConversionResult> {
        let docs: Vec<_> = documents.iter().map(|d| d.inner.clone()).collect();

        let result = py.allow_threads(|| self.inner.convert_batch(docs));

        match result {
            Ok(r) => Ok(PyConversionResult {
                feed: PyGtfsFeed::from_inner(r.feed),
                warnings: r.warnings.iter().map(|w| w.message.clone()).collect(),
                stats: PyConversionStats { inner: r.stats },
            }),
            Err(e) => Err(PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn __repr__(&self) -> String {
        "TxcToGtfsConverter()".to_string()
    }
}


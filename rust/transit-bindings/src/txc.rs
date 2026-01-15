//! TXC Python bindings.

use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use txc_parser::TxcDocument;

/// Python wrapper for TXC document.
#[pyclass(name = "TxcDocument")]
pub struct PyTxcDocument {
    pub(crate) inner: TxcDocument,
}

#[pymethods]
impl PyTxcDocument {
    /// Parse a TXC document from a file path.
    #[staticmethod]
    fn from_path(path: &str) -> PyResult<Self> {
        TxcDocument::from_path(path)
            .map(|inner| Self { inner })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Parse a TXC document from bytes.
    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        TxcDocument::from_bytes(data)
            .map(|inner| Self { inner })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Parse a TXC document from a string.
    #[staticmethod]
    fn from_string(xml: &str) -> PyResult<Self> {
        TxcDocument::from_str(xml)
            .map(|inner| Self { inner })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Schema version of the document.
    #[getter]
    fn schema_version(&self) -> String {
        self.inner.schema_version.clone()
    }

    /// Filename (if loaded from file).
    #[getter]
    fn filename(&self) -> Option<String> {
        self.inner.filename.clone()
    }

    /// Number of operators.
    #[getter]
    fn operator_count(&self) -> usize {
        self.inner.operators.len()
    }

    /// Number of services.
    #[getter]
    fn service_count(&self) -> usize {
        self.inner.services.len()
    }

    /// Number of stop points.
    #[getter]
    fn stop_point_count(&self) -> usize {
        self.inner.stop_points.len()
    }

    /// Number of vehicle journeys.
    #[getter]
    fn vehicle_journey_count(&self) -> usize {
        self.inner.vehicle_journeys.len()
    }

    /// Number of journey pattern sections.
    #[getter]
    fn journey_pattern_section_count(&self) -> usize {
        self.inner.journey_pattern_sections.len()
    }

    /// Get operator names.
    fn get_operator_names(&self) -> Vec<String> {
        self.inner
            .operators
            .iter()
            .map(|op| op.display_name().to_string())
            .collect()
    }

    /// Get service codes.
    fn get_service_codes(&self) -> Vec<String> {
        self.inner
            .services
            .iter()
            .map(|s| s.service_code.clone())
            .collect()
    }

    /// Get stop ATCO codes.
    fn get_stop_codes(&self) -> Vec<String> {
        self.inner
            .stop_points
            .iter()
            .map(|s| s.atco_code.clone())
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "TxcDocument(version={:?}, operators={}, services={}, stops={}, journeys={})",
            self.inner.schema_version,
            self.inner.operators.len(),
            self.inner.services.len(),
            self.inner.stop_points.len(),
            self.inner.vehicle_journeys.len()
        )
    }
}

//! CSV Python bindings.

use csv_parser::CsvDocument;
use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;

/// Python wrapper for CSV document.
#[pyclass(name = "CsvDocument")]
pub struct PyCsvDocument {
    inner: CsvDocument,
}

#[pymethods]
impl PyCsvDocument {
    /// Create a new empty CSV document.
    #[new]
    fn new() -> Self {
        Self {
            inner: CsvDocument::new(),
        }
    }

    /// Parse a CSV file from path.
    #[staticmethod]
    fn from_path(path: &str) -> PyResult<Self> {
        CsvDocument::from_path(path)
            .map(|inner| Self { inner })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Parse CSV from bytes.
    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        CsvDocument::from_bytes(data)
            .map(|inner| Self { inner })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Parse CSV from string.
    #[staticmethod]
    fn from_string(csv: &str) -> PyResult<Self> {
        CsvDocument::from_str(csv)
            .map(|inner| Self { inner })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Write CSV to path.
    fn to_path(&self, path: &str) -> PyResult<()> {
        self.inner
            .to_path(path)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Write CSV to string.
    fn to_string(&self) -> PyResult<String> {
        self.inner
            .to_string()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Get the number of rows.
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Get column names.
    #[getter]
    fn columns(&self) -> Vec<String> {
        self.inner
            .schema
            .columns
            .iter()
            .map(|c| c.name.clone())
            .collect()
    }

    /// Get rows as list of dicts.
    #[getter]
    fn rows(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let json_str = serde_json::to_string(&self.inner.rows)
                .map_err(|e| PyIOError::new_err(e.to_string()))?;

            let json_module = py.import("json")?;
            let result = json_module.call_method1("loads", (json_str,))?;
            Ok(result.into())
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "CsvDocument(columns={}, rows={})",
            self.inner.schema.columns.len(),
            self.inner.len()
        )
    }
}

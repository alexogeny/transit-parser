//! JSON Python bindings.

use json_parser::JsonDocument;
use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;

/// Python wrapper for JSON document.
#[pyclass(name = "JsonDocument")]
pub struct PyJsonDocument {
    inner: JsonDocument,
}

#[pymethods]
impl PyJsonDocument {
    /// Parse a JSON file from path.
    #[staticmethod]
    fn from_path(path: &str) -> PyResult<Self> {
        JsonDocument::from_path(path)
            .map(|inner| Self { inner })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Parse JSON from bytes.
    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        JsonDocument::from_bytes(data)
            .map(|inner| Self { inner })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Parse JSON from string.
    #[staticmethod]
    fn from_string(json: &str) -> PyResult<Self> {
        JsonDocument::from_str(json)
            .map(|inner| Self { inner })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Write JSON to path.
    fn to_path(&self, path: &str) -> PyResult<()> {
        self.inner
            .to_path(path)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Write JSON to string.
    fn to_string(&self) -> PyResult<String> {
        self.inner
            .to_string()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Write JSON to pretty-printed string.
    fn to_string_pretty(&self) -> PyResult<String> {
        self.inner
            .to_string_pretty()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Check if root is an object.
    fn is_object(&self) -> bool {
        self.inner.is_object()
    }

    /// Check if root is an array.
    fn is_array(&self) -> bool {
        self.inner.is_array()
    }

    /// Get the root value as Python object.
    #[getter]
    fn root(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let json_str = serde_json::to_string(&self.inner.root)
                .map_err(|e| PyIOError::new_err(e.to_string()))?;

            let json_module = py.import("json")?;
            let result = json_module.call_method1("loads", (json_str,))?;
            Ok(result.into())
        })
    }

    /// Get a value by JSON pointer.
    fn pointer(&self, path: &str) -> PyResult<Option<PyObject>> {
        Python::with_gil(|py| {
            match self.inner.pointer(path) {
                Some(value) => {
                    let json_str = serde_json::to_string(value)
                        .map_err(|e| PyIOError::new_err(e.to_string()))?;

                    let json_module = py.import("json")?;
                    let result = json_module.call_method1("loads", (json_str,))?;
                    Ok(Some(result.into()))
                }
                None => Ok(None),
            }
        })
    }

    fn __repr__(&self) -> String {
        let type_str = if self.inner.is_object() {
            "object"
        } else if self.inner.is_array() {
            "array"
        } else {
            "value"
        };
        format!("JsonDocument(type={})", type_str)
    }
}

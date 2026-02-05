//! Schedule Python bindings.

use crate::gtfs::PyGtfsFeed;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use schedule_parser::{
    ColumnMapping, CsvExporter, DeadheadInferrer, ExportConfig, ExportPreset,
    GtfsComplianceLevel, ReadOptions, Schedule, ScheduleReader, ScheduleRow, ValidationConfig,
    ValidationResult, Validator,
};

/// Python wrapper for ScheduleRow.
#[pyclass(name = "ScheduleRow")]
#[derive(Clone)]
pub struct PyScheduleRow {
    inner: ScheduleRow,
}

#[pymethods]
impl PyScheduleRow {
    #[new]
    fn new() -> Self {
        Self {
            inner: ScheduleRow::default(),
        }
    }

    #[getter]
    fn run_number(&self) -> Option<String> {
        self.inner.run_number.clone()
    }

    #[setter]
    fn set_run_number(&mut self, value: Option<String>) {
        self.inner.run_number = value;
    }

    #[getter]
    fn block(&self) -> Option<String> {
        self.inner.block.clone()
    }

    #[setter]
    fn set_block(&mut self, value: Option<String>) {
        self.inner.block = value;
    }

    #[getter]
    fn start_place(&self) -> Option<String> {
        self.inner.start_place.clone()
    }

    #[setter]
    fn set_start_place(&mut self, value: Option<String>) {
        self.inner.start_place = value;
    }

    #[getter]
    fn end_place(&self) -> Option<String> {
        self.inner.end_place.clone()
    }

    #[setter]
    fn set_end_place(&mut self, value: Option<String>) {
        self.inner.end_place = value;
    }

    #[getter]
    fn start_time(&self) -> Option<String> {
        self.inner.start_time.clone()
    }

    #[setter]
    fn set_start_time(&mut self, value: Option<String>) {
        self.inner.start_time = value;
    }

    #[getter]
    fn end_time(&self) -> Option<String> {
        self.inner.end_time.clone()
    }

    #[setter]
    fn set_end_time(&mut self, value: Option<String>) {
        self.inner.end_time = value;
    }

    #[getter]
    fn trip_id(&self) -> Option<String> {
        self.inner.trip_id.clone()
    }

    #[setter]
    fn set_trip_id(&mut self, value: Option<String>) {
        self.inner.trip_id = value;
    }

    #[getter]
    fn depot(&self) -> Option<String> {
        self.inner.depot.clone()
    }

    #[setter]
    fn set_depot(&mut self, value: Option<String>) {
        self.inner.depot = value;
    }

    #[getter]
    fn vehicle_class(&self) -> Option<String> {
        self.inner.vehicle_class.clone()
    }

    #[getter]
    fn vehicle_type(&self) -> Option<String> {
        self.inner.vehicle_type.clone()
    }

    #[getter]
    fn start_lat(&self) -> Option<f64> {
        self.inner.start_lat
    }

    #[getter]
    fn start_lon(&self) -> Option<f64> {
        self.inner.start_lon
    }

    #[getter]
    fn end_lat(&self) -> Option<f64> {
        self.inner.end_lat
    }

    #[getter]
    fn end_lon(&self) -> Option<f64> {
        self.inner.end_lon
    }

    #[getter]
    fn route_shape_id(&self) -> Option<String> {
        self.inner.route_shape_id.clone()
    }

    #[getter]
    fn row_type(&self) -> String {
        format!("{:?}", self.inner.row_type).to_lowercase()
    }

    #[getter]
    fn duty_id(&self) -> Option<String> {
        self.inner.duty_id.clone()
    }

    #[getter]
    fn shift_id(&self) -> Option<String> {
        self.inner.shift_id.clone()
    }

    /// Check if this is a revenue (passenger-carrying) trip.
    fn is_revenue(&self) -> bool {
        self.inner.is_revenue()
    }

    /// Check if this is a deadhead movement.
    fn is_deadhead(&self) -> bool {
        self.inner.is_deadhead()
    }

    /// Get duration in seconds.
    fn duration_seconds(&self) -> Option<u32> {
        self.inner.duration_seconds()
    }

    fn __repr__(&self) -> String {
        format!(
            "ScheduleRow(block={:?}, trip_id={:?}, start_time={:?})",
            self.inner.block, self.inner.trip_id, self.inner.start_time
        )
    }
}

impl From<ScheduleRow> for PyScheduleRow {
    fn from(inner: ScheduleRow) -> Self {
        Self { inner }
    }
}

/// Python wrapper for Schedule.
#[pyclass(name = "Schedule")]
pub struct PySchedule {
    inner: Schedule,
}

#[pymethods]
impl PySchedule {
    #[new]
    fn new() -> Self {
        Self {
            inner: Schedule::new(),
        }
    }

    /// Load a schedule from a CSV file.
    #[staticmethod]
    fn from_csv(path: &str) -> PyResult<Self> {
        ScheduleReader::read_path(path, ReadOptions::new())
            .map(|s| Self { inner: s })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Load a schedule from a CSV string.
    #[staticmethod]
    fn from_csv_string(csv_str: &str) -> PyResult<Self> {
        ScheduleReader::read_str(csv_str, ReadOptions::new())
            .map(|s| Self { inner: s })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Load a schedule with custom column mapping.
    #[staticmethod]
    #[pyo3(signature = (path, column_mapping=None))]
    fn from_csv_with_mapping(path: &str, column_mapping: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let mut options = ReadOptions::new();

        if let Some(mapping) = column_mapping {
            let mut cm = ColumnMapping::new();
            for (key, value) in mapping.iter() {
                let field: String = key.extract()?;
                let column: String = value.extract()?;
                cm.add(field, column);
            }
            options = options.with_mapping(cm);
        }

        ScheduleReader::read_path(path, options)
            .map(|s| Self { inner: s })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Number of rows in the schedule.
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Get all rows.
    #[getter]
    fn rows(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let list = PyList::new(
            py,
            self.inner.rows.iter().cloned().map(PyScheduleRow::from),
        )?;
        Ok(list.into())
    }

    /// Get revenue trips count.
    #[getter]
    fn revenue_trip_count(&self) -> usize {
        self.inner.revenue_trips().count()
    }

    /// Get unique block IDs.
    fn block_ids(&self) -> Vec<String> {
        self.inner.block_ids()
    }

    /// Get unique run numbers.
    fn run_numbers(&self) -> Vec<String> {
        self.inner.run_numbers()
    }

    /// Get unique depot codes.
    fn depots(&self) -> Vec<String> {
        self.inner.depots()
    }

    /// Get unique trip IDs.
    fn trip_ids(&self) -> Vec<String> {
        self.inner.trip_ids()
    }

    /// Get summary statistics.
    fn summary(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let summary = self.inner.summary();
        let dict = PyDict::new(py);
        dict.set_item("total_rows", summary.total_rows)?;
        dict.set_item("revenue_trips", summary.revenue_trips)?;
        dict.set_item("deadheads", summary.deadheads)?;
        dict.set_item("breaks_and_reliefs", summary.breaks_and_reliefs)?;
        dict.set_item("unique_blocks", summary.unique_blocks)?;
        dict.set_item("unique_runs", summary.unique_runs)?;
        dict.set_item("unique_depots", summary.unique_depots)?;
        Ok(dict.into())
    }

    /// Validate the schedule against GTFS data.
    #[pyo3(signature = (gtfs, config=None))]
    fn validate(&mut self, gtfs: &PyGtfsFeed, config: Option<&PyValidationConfig>) -> PyResult<PyValidationResult> {
        let cfg = config
            .map(|c| c.inner.clone())
            .unwrap_or_else(ValidationConfig::new);

        let validator = Validator::new(cfg);
        let result = validator.validate(&mut self.inner, &gtfs.inner);

        Ok(PyValidationResult { inner: result })
    }

    /// Validate schedule structure (without GTFS).
    #[pyo3(signature = (config=None))]
    fn validate_structure(&mut self, config: Option<&PyValidationConfig>) -> PyResult<PyValidationResult> {
        let cfg = config
            .map(|c| c.inner.clone())
            .unwrap_or_else(ValidationConfig::new);

        let validator = Validator::new(cfg);
        let result = validator.validate_structure(&mut self.inner);

        Ok(PyValidationResult { inner: result })
    }

    /// Infer missing deadheads.
    #[pyo3(signature = (gtfs=None, default_depot=None))]
    fn infer_deadheads(
        &mut self,
        gtfs: Option<&PyGtfsFeed>,
        default_depot: Option<String>,
    ) -> PyResult<PyDeadheadInferenceResult> {
        use schedule_parser::deadhead::inferrer::InferenceConfig;

        let mut config = InferenceConfig::new();
        if let Some(depot) = default_depot {
            config = config.with_default_depot(depot);
        }

        let inferrer = match gtfs {
            Some(g) => DeadheadInferrer::with_gtfs(config, &g.inner),
            None => DeadheadInferrer::new(config),
        };

        let result = inferrer.infer(&mut self.inner);
        Ok(PyDeadheadInferenceResult { inner: result })
    }

    /// Export to CSV file.
    #[pyo3(signature = (path, columns=None, preset=None))]
    fn to_csv(
        &self,
        path: &str,
        columns: Option<Vec<String>>,
        preset: Option<&str>,
    ) -> PyResult<()> {
        let config = Self::build_export_config(columns, preset)?;
        let exporter = CsvExporter::new(config);
        exporter
            .export_to_path(&self.inner, path)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Export to CSV string.
    #[pyo3(signature = (columns=None, preset=None))]
    fn to_csv_string(&self, columns: Option<Vec<String>>, preset: Option<&str>) -> PyResult<String> {
        let config = Self::build_export_config(columns, preset)?;
        let exporter = CsvExporter::new(config);
        exporter
            .export_to_string(&self.inner)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        let summary = self.inner.summary();
        format!(
            "Schedule(rows={}, blocks={}, runs={})",
            summary.total_rows, summary.unique_blocks, summary.unique_runs
        )
    }
}

impl PySchedule {
    fn build_export_config(
        columns: Option<Vec<String>>,
        preset: Option<&str>,
    ) -> PyResult<ExportConfig> {
        if let Some(preset_name) = preset {
            let preset = match preset_name.to_lowercase().as_str() {
                "default" => ExportPreset::Default,
                "minimal" => ExportPreset::Minimal,
                "extended" => ExportPreset::Extended,
                "optibus" | "optibus_like" => ExportPreset::OptibusLike,
                "hastus" | "hastus_like" => ExportPreset::HastusLike,
                "gtfs_block" => ExportPreset::GtfsBlock,
                _ => return Err(PyValueError::new_err(format!("Unknown preset: {}", preset_name))),
            };
            return Ok(preset.to_config());
        }

        if let Some(cols) = columns {
            let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
            return Ok(ExportConfig::with_columns(col_refs));
        }

        Ok(ExportConfig::default())
    }
}

/// Python wrapper for ValidationConfig.
#[pyclass(name = "ValidationConfig")]
#[derive(Clone)]
pub struct PyValidationConfig {
    inner: ValidationConfig,
}

#[pymethods]
impl PyValidationConfig {
    #[new]
    #[pyo3(signature = (
        gtfs_compliance=None,
        min_layover_seconds=None,
        max_trip_duration_seconds=None,
        max_duty_length_seconds=None,
        max_continuous_driving_seconds=None,
        min_break_duration_seconds=None,
        time_tolerance_seconds=None,
        validate_block_continuity=None,
        validate_duty_constraints=None,
        generate_warnings=None
    ))]
    fn new(
        gtfs_compliance: Option<&str>,
        min_layover_seconds: Option<u32>,
        max_trip_duration_seconds: Option<u32>,
        max_duty_length_seconds: Option<u32>,
        max_continuous_driving_seconds: Option<u32>,
        min_break_duration_seconds: Option<u32>,
        time_tolerance_seconds: Option<u32>,
        validate_block_continuity: Option<bool>,
        validate_duty_constraints: Option<bool>,
        generate_warnings: Option<bool>,
    ) -> PyResult<Self> {
        let mut config = ValidationConfig::new();

        if let Some(level) = gtfs_compliance {
            config.gtfs_compliance = match level.to_lowercase().as_str() {
                "strict" => GtfsComplianceLevel::Strict,
                "standard" => GtfsComplianceLevel::Standard,
                "lenient" => GtfsComplianceLevel::Lenient,
                _ => return Err(PyValueError::new_err(format!("Unknown compliance level: {}", level))),
            };
        }

        if let Some(v) = min_layover_seconds {
            config.business_rules.min_layover_seconds = v;
        }
        if let Some(v) = max_trip_duration_seconds {
            config.business_rules.max_trip_duration_seconds = v;
        }
        if let Some(v) = max_duty_length_seconds {
            config.business_rules.max_duty_length_seconds = v;
        }
        if let Some(v) = max_continuous_driving_seconds {
            config.business_rules.max_continuous_driving_seconds = v;
        }
        if let Some(v) = min_break_duration_seconds {
            config.business_rules.min_break_duration_seconds = v;
        }
        if let Some(v) = time_tolerance_seconds {
            config.business_rules.time_tolerance_seconds = v;
        }
        if let Some(v) = validate_block_continuity {
            config.validate_block_continuity = v;
        }
        if let Some(v) = validate_duty_constraints {
            config.validate_duty_constraints = v;
        }
        if let Some(v) = generate_warnings {
            config.generate_warnings = v;
        }

        Ok(Self { inner: config })
    }

    /// Create a strict validation config.
    #[staticmethod]
    fn strict() -> Self {
        Self {
            inner: ValidationConfig::strict(),
        }
    }

    /// Create a lenient validation config.
    #[staticmethod]
    fn lenient() -> Self {
        Self {
            inner: ValidationConfig::lenient(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ValidationConfig(gtfs_compliance={:?})",
            self.inner.gtfs_compliance
        )
    }
}

/// Python wrapper for ValidationResult.
#[pyclass(name = "ValidationResult")]
pub struct PyValidationResult {
    inner: ValidationResult,
}

#[pymethods]
impl PyValidationResult {
    /// Check if validation passed.
    #[getter]
    fn is_valid(&self) -> bool {
        self.inner.is_valid()
    }

    /// Get error count.
    #[getter]
    fn error_count(&self) -> usize {
        self.inner.error_count()
    }

    /// Get warning count.
    #[getter]
    fn warning_count(&self) -> usize {
        self.inner.warning_count()
    }

    /// Get all errors.
    #[getter]
    fn errors(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let errors: Vec<_> = self
            .inner
            .errors
            .iter()
            .map(|e| {
                let dict = PyDict::new(py);
                dict.set_item("code", &e.code).unwrap();
                dict.set_item("category", format!("{:?}", e.category)).unwrap();
                dict.set_item("message", &e.message).unwrap();
                dict.set_item("context", &e.context).unwrap();
                dict.to_object(py)
            })
            .collect();
        Ok(PyList::new(py, errors)?.into())
    }

    /// Get all warnings.
    #[getter]
    fn warnings(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let warnings: Vec<_> = self
            .inner
            .warnings
            .iter()
            .map(|w| {
                let dict = PyDict::new(py);
                dict.set_item("code", &w.code).unwrap();
                dict.set_item("category", format!("{:?}", w.category)).unwrap();
                dict.set_item("message", &w.message).unwrap();
                dict.set_item("context", &w.context).unwrap();
                dict.to_object(py)
            })
            .collect();
        Ok(PyList::new(py, warnings)?.into())
    }

    /// Get number of rows validated.
    #[getter]
    fn rows_validated(&self) -> usize {
        self.inner.rows_validated
    }

    /// Get number of blocks validated.
    #[getter]
    fn blocks_validated(&self) -> usize {
        self.inner.blocks_validated
    }

    fn __repr__(&self) -> String {
        format!(
            "ValidationResult(valid={}, errors={}, warnings={})",
            self.inner.is_valid(),
            self.inner.error_count(),
            self.inner.warning_count()
        )
    }
}

/// Python wrapper for DeadheadInferenceResult.
#[pyclass(name = "DeadheadInferenceResult")]
pub struct PyDeadheadInferenceResult {
    inner: schedule_parser::DeadheadInferenceResult,
}

#[pymethods]
impl PyDeadheadInferenceResult {
    /// Number of inferred pull-outs.
    #[getter]
    fn pull_out_count(&self) -> usize {
        self.inner.pull_outs.len()
    }

    /// Number of inferred pull-ins.
    #[getter]
    fn pull_in_count(&self) -> usize {
        self.inner.pull_ins.len()
    }

    /// Number of inferred interlinings.
    #[getter]
    fn interlining_count(&self) -> usize {
        self.inner.interlinings.len()
    }

    /// Total count of inferred deadheads.
    #[getter]
    fn total_count(&self) -> usize {
        self.inner.total_count()
    }

    /// Blocks that couldn't have deadheads inferred.
    #[getter]
    fn incomplete_blocks(&self) -> Vec<String> {
        self.inner.incomplete_blocks.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "DeadheadInferenceResult(pull_outs={}, pull_ins={}, interlinings={})",
            self.inner.pull_outs.len(),
            self.inner.pull_ins.len(),
            self.inner.interlinings.len()
        )
    }
}

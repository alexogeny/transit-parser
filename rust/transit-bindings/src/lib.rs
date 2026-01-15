//! PyO3 Python bindings for transit-parser.

use pyo3::prelude::*;

mod adapters;
mod csv;
mod gtfs;
mod json;
mod lazy_gtfs;
mod models;
mod txc;

/// Transit data parser Python module.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register submodules
    m.add_class::<models::PyAgency>()?;
    m.add_class::<models::PyStop>()?;
    m.add_class::<models::PyRoute>()?;
    m.add_class::<models::PyTrip>()?;
    m.add_class::<models::PyStopTime>()?;
    m.add_class::<models::PyCalendar>()?;
    m.add_class::<models::PyCalendarDate>()?;
    m.add_class::<models::PyShape>()?;

    // GTFS functions
    m.add_class::<gtfs::PyGtfsFeed>()?;
    m.add_class::<lazy_gtfs::PyLazyGtfsFeed>()?;

    // TXC functions
    m.add_class::<txc::PyTxcDocument>()?;

    // CSV functions
    m.add_class::<csv::PyCsvDocument>()?;

    // JSON functions
    m.add_class::<json::PyJsonDocument>()?;

    // Adapter functions
    m.add_class::<adapters::PyTxcToGtfsConverter>()?;
    m.add_class::<adapters::PyConversionOptions>()?;
    m.add_class::<adapters::PyConversionResult>()?;
    m.add_class::<adapters::PyConversionStats>()?;

    Ok(())
}

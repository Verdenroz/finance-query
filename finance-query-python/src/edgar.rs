//! SEC EDGAR initialization helpers.
//!
//! EDGAR requires a User-Agent containing a contact email per SEC's
//! [Fair Access](https://www.sec.gov/os/accessing-edgar-data) policy.
//! Call `edgar_init(email)` once at process start before any
//! `Ticker.edgar_submissions()` or related call.

use crate::error::to_py_err;
use pyo3::prelude::*;
use std::time::Duration;

#[pyfunction]
pub fn edgar_init(email: String) -> PyResult<()> {
    finance_query::edgar::init(email).map_err(to_py_err)
}

#[pyfunction]
#[pyo3(signature = (email, app_name, timeout_seconds = 60))]
pub fn edgar_init_with_config(email: String, app_name: String, timeout_seconds: u64) -> PyResult<()> {
    finance_query::edgar::init_with_config(email, app_name, Duration::from_secs(timeout_seconds))
        .map_err(to_py_err)
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(pyo3::wrap_pyfunction!(edgar_init, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(edgar_init_with_config, m)?)?;
    Ok(())
}

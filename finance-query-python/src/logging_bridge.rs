//! Bridge from Rust `tracing` events to user-visible logs.
//!
//! Phase 1 installs a stderr fmt subscriber. Phase 3 may replace this with a
//! proper bridge to Python's `logging` module.

use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (level = "INFO"))]
pub fn enable_logging(level: &str) -> PyResult<()> {
    let level_filter = match level.to_uppercase().as_str() {
        "TRACE" => tracing::Level::TRACE,
        "DEBUG" => tracing::Level::DEBUG,
        "INFO" => tracing::Level::INFO,
        "WARN" => tracing::Level::WARN,
        "ERROR" => tracing::Level::ERROR,
        other => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "invalid log level: {} (expected TRACE/DEBUG/INFO/WARN/ERROR)",
                other
            )));
        }
    };
    // try_init() returns Err if a subscriber is already set — that's fine; ignore.
    let _ = tracing_subscriber::fmt()
        .with_max_level(level_filter)
        .try_init();
    Ok(())
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(pyo3::wrap_pyfunction!(enable_logging, m)?)?;
    Ok(())
}

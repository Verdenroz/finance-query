//! Shared tokio runtime singleton, initialised once on module init.

use pyo3::prelude::*;

pub fn init_runtime() -> PyResult<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("finance-query-py")
        .build()
        .map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("tokio init failed: {}", e))
        })?;
    let rt: &'static tokio::runtime::Runtime = Box::leak(Box::new(rt));
    pyo3_async_runtimes::tokio::init_with_runtime(rt)
        .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("runtime already initialised"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_initialises_once() {
        Python::initialize();
        assert!(init_runtime().is_ok());
        // Second call should error (already initialised)
        assert!(init_runtime().is_err());
    }
}

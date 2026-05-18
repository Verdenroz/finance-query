//! Python wrapper for `finance_query::Tickers` (batch ticker operations).

use crate::error::to_py_err;
use crate::models::{PyChart, PyQuote};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;

#[pyclass(frozen, name = "Tickers")]
pub struct PyTickers {
    inner: Arc<finance_query::Tickers>,
}

#[pyclass(frozen, name = "BatchResult")]
pub struct PyBatchResult {
    #[pyo3(get)]
    data: PyObject,
    #[pyo3(get)]
    errors: PyObject,
}

#[pymethods]
impl PyTickers {
    #[staticmethod]
    fn new<'py>(py: Python<'py>, symbols: Vec<String>) -> PyResult<Bound<'py, PyAny>> {
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let t = finance_query::Tickers::new(symbols)
                .await
                .map_err(to_py_err)?;
            Ok(PyTickers {
                inner: Arc::new(t),
            })
        })
    }

    fn symbols(&self) -> Vec<String> {
        self.inner
            .symbols()
            .into_iter()
            .map(String::from)
            .collect()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn quotes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner.quotes().await.map_err(to_py_err)?;
            Python::with_gil(|py| {
                let data = PyDict::new(py);
                for (sym, q) in r.quotes {
                    data.set_item(sym, PyQuote::from(q))?;
                }
                let errors = PyDict::new(py);
                for (sym, e) in r.errors {
                    errors.set_item(sym, e)?;
                }
                Ok(PyBatchResult {
                    data: data.into_any().unbind(),
                    errors: errors.into_any().unbind(),
                })
            })
        })
    }

    fn charts<'py>(
        &self,
        py: Python<'py>,
        interval: finance_query::PyInterval,
        range: finance_query::PyTimeRange,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner
                .charts(interval.into(), range.into())
                .await
                .map_err(to_py_err)?;
            Python::with_gil(|py| {
                let data = PyDict::new(py);
                for (sym, c) in r.charts {
                    data.set_item(sym, PyChart::from(c))?;
                }
                let errors = PyDict::new(py);
                for (sym, e) in r.errors {
                    errors.set_item(sym, e)?;
                }
                Ok(PyBatchResult {
                    data: data.into_any().unbind(),
                    errors: errors.into_any().unbind(),
                })
            })
        })
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTickers>()?;
    m.add_class::<PyBatchResult>()?;
    Ok(())
}

//! Python wrapper for `finance_query::Tickers` (batch ticker operations).

use crate::backtest::PyPortfolioResult;
use crate::error::to_py_err;
use crate::models::{
    PyCapitalGain, PyChart, PyDividend, PyFinancialStatement, PyIndicatorsSummary, PyNews,
    PyOptions, PyQuote, PyRecommendation, PySpark, PySplit,
};
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
            Ok(PyTickers { inner: Arc::new(t) })
        })
    }

    fn symbols(&self) -> Vec<String> {
        self.inner.symbols().into_iter().map(String::from).collect()
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

    fn chart<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        interval: finance_query::PyInterval,
        range: finance_query::PyTimeRange,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner
                .chart(&symbol, interval.into(), range.into())
                .await
                .map_err(to_py_err)?;
            Ok(PyChart::from(r))
        })
    }

    fn charts_range<'py>(
        &self,
        py: Python<'py>,
        interval: finance_query::PyInterval,
        start: i64,
        end: i64,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner
                .charts_range(interval.into(), start, end)
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

    fn dividends<'py>(
        &self,
        py: Python<'py>,
        range: finance_query::PyTimeRange,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner.dividends(range.into()).await.map_err(to_py_err)?;
            Python::with_gil(|py| {
                let data = PyDict::new(py);
                for (sym, items) in r.dividends {
                    let list: Vec<PyDividend> = items.into_iter().map(PyDividend::from).collect();
                    data.set_item(sym, list)?;
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

    fn splits<'py>(
        &self,
        py: Python<'py>,
        range: finance_query::PyTimeRange,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner.splits(range.into()).await.map_err(to_py_err)?;
            Python::with_gil(|py| {
                let data = PyDict::new(py);
                for (sym, items) in r.splits {
                    let list: Vec<PySplit> = items.into_iter().map(PySplit::from).collect();
                    data.set_item(sym, list)?;
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

    fn capital_gains<'py>(
        &self,
        py: Python<'py>,
        range: finance_query::PyTimeRange,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner.capital_gains(range.into()).await.map_err(to_py_err)?;
            Python::with_gil(|py| {
                let data = PyDict::new(py);
                for (sym, items) in r.capital_gains {
                    let list: Vec<PyCapitalGain> =
                        items.into_iter().map(PyCapitalGain::from).collect();
                    data.set_item(sym, list)?;
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

    fn recommendations<'py>(&self, py: Python<'py>, limit: u32) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner.recommendations(limit).await.map_err(to_py_err)?;
            Python::with_gil(|py| {
                let data = PyDict::new(py);
                for (sym, v) in r.recommendations {
                    data.set_item(sym, PyRecommendation::from(v))?;
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

    fn financials<'py>(
        &self,
        py: Python<'py>,
        statement: finance_query::PyStatementType,
        frequency: finance_query::PyFrequency,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner
                .financials(statement.into(), frequency.into())
                .await
                .map_err(to_py_err)?;
            Python::with_gil(|py| {
                let data = PyDict::new(py);
                for (sym, v) in r.financials {
                    data.set_item(sym, PyFinancialStatement::from(v))?;
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

    fn spark<'py>(
        &self,
        py: Python<'py>,
        interval: finance_query::PyInterval,
        range: finance_query::PyTimeRange,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner
                .spark(interval.into(), range.into())
                .await
                .map_err(to_py_err)?;
            Python::with_gil(|py| {
                let data = PyDict::new(py);
                for (sym, v) in r.sparks {
                    data.set_item(sym, PySpark::from(v))?;
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

    #[pyo3(signature = (date=None))]
    fn options<'py>(&self, py: Python<'py>, date: Option<i64>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner.options(date).await.map_err(to_py_err)?;
            Python::with_gil(|py| {
                let data = PyDict::new(py);
                for (sym, v) in r.options {
                    data.set_item(sym, PyOptions::from(v))?;
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

    fn indicators<'py>(
        &self,
        py: Python<'py>,
        interval: finance_query::PyInterval,
        range: finance_query::PyTimeRange,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner
                .indicators(interval.into(), range.into())
                .await
                .map_err(to_py_err)?;
            Python::with_gil(|py| {
                let data = PyDict::new(py);
                for (sym, v) in r.indicators {
                    data.set_item(sym, PyIndicatorsSummary::from(v))?;
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

    /// Run a portfolio backtest across all symbols with the given strategy.
    ///
    /// `strategy` must be one of the prebuilt strategy classes.
    fn backtest<'py>(
        &self,
        py: Python<'py>,
        strategy: Py<PyAny>,
        interval: finance_query::PyInterval,
        range: finance_query::PyTimeRange,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        // Extract the concrete strategy on the calling thread (needs GIL).
        let kind = crate::strategy::extract_strategy(py, &strategy)?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = crate::strategy::run_backtest_portfolio(
                &inner,
                kind,
                interval.into(),
                range.into(),
                None,
            )
            .await
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
            Ok(PyPortfolioResult::from(r))
        })
    }

    fn news<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner.news().await.map_err(to_py_err)?;
            Python::with_gil(|py| {
                let data = PyDict::new(py);
                for (sym, items) in r.news {
                    let list: Vec<PyNews> = items.into_iter().map(PyNews::from).collect();
                    data.set_item(sym, list)?;
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

    fn quote<'py>(&self, py: Python<'py>, symbol: String) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let q = inner.quote(&symbol).await.map_err(to_py_err)?;
            Ok(PyQuote::from(q))
        })
    }

    fn clear_cache<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.clear_cache().await;
            Ok(())
        })
    }

    fn clear_quote_cache<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.clear_quote_cache().await;
            Ok(())
        })
    }

    fn clear_chart_cache<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.clear_chart_cache().await;
            Ok(())
        })
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTickers>()?;
    m.add_class::<PyBatchResult>()?;
    Ok(())
}

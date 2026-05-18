//! Python wrapper for `finance_query::Ticker`.

use crate::error::to_py_err;
use crate::models::{
    PyCapitalGain, PyChart, PyDividend, PyEdgarSubmissions, PyFinancialStatement, PyNews, PyQuote,
    PyRecommendation, PySplit,
};
use pyo3::prelude::*;
use std::sync::Arc;
use std::time::Duration;

#[pyclass(frozen, name = "Ticker")]
pub struct PyTicker {
    inner: Arc<finance_query::Ticker>,
}

#[pymethods]
impl PyTicker {
    #[staticmethod]
    fn new<'py>(py: Python<'py>, symbol: String) -> PyResult<Bound<'py, PyAny>> {
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let t = finance_query::Ticker::new(symbol).await.map_err(to_py_err)?;
            Ok(PyTicker { inner: Arc::new(t) })
        })
    }

    #[staticmethod]
    fn builder(symbol: String) -> PyTickerBuilder {
        PyTickerBuilder {
            inner: Some(finance_query::Ticker::builder(symbol)),
        }
    }

    #[getter]
    fn symbol(&self) -> String {
        self.inner.symbol().to_string()
    }

    fn quote<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let q = inner.quote().await.map_err(to_py_err)?;
            Ok(PyQuote::from(q))
        })
    }

    fn chart<'py>(
        &self,
        py: Python<'py>,
        interval: finance_query::PyInterval,
        range: finance_query::PyTimeRange,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let c = inner
                .chart(interval.into(), range.into())
                .await
                .map_err(to_py_err)?;
            Ok(PyChart::from(c))
        })
    }

    fn chart_range<'py>(
        &self,
        py: Python<'py>,
        interval: finance_query::PyInterval,
        start: i64,
        end: i64,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let c = inner
                .chart_range(interval.into(), start, end)
                .await
                .map_err(to_py_err)?;
            Ok(PyChart::from(c))
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
            let py_vec: Vec<PyDividend> = r.into_iter().map(Into::into).collect();
            Ok(py_vec)
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
            let py_vec: Vec<PySplit> = r.into_iter().map(Into::into).collect();
            Ok(py_vec)
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
            let py_vec: Vec<PyCapitalGain> = r.into_iter().map(Into::into).collect();
            Ok(py_vec)
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
            Ok(PyFinancialStatement::from(r))
        })
    }

    fn news<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner.news().await.map_err(to_py_err)?;
            let py_vec: Vec<PyNews> = r.into_iter().map(Into::into).collect();
            Ok(py_vec)
        })
    }

    fn recommendations<'py>(&self, py: Python<'py>, limit: u32) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner.recommendations(limit).await.map_err(to_py_err)?;
            Ok(PyRecommendation::from(r))
        })
    }

    fn edgar_submissions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let r = inner.edgar_submissions().await.map_err(to_py_err)?;
            Ok(PyEdgarSubmissions::from(r))
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

#[pyclass(name = "TickerBuilder")]
pub struct PyTickerBuilder {
    inner: Option<finance_query::TickerBuilder>,
}

#[pymethods]
impl PyTickerBuilder {
    fn lang(mut slf: PyRefMut<'_, Self>, lang: String) -> PyRefMut<'_, Self> {
        if let Some(b) = slf.inner.take() {
            slf.inner = Some(b.lang(lang));
        }
        slf
    }

    fn region_code(mut slf: PyRefMut<'_, Self>, region: String) -> PyRefMut<'_, Self> {
        if let Some(b) = slf.inner.take() {
            slf.inner = Some(b.region_code(region));
        }
        slf
    }

    fn region(
        mut slf: PyRefMut<'_, Self>,
        region: finance_query::PyRegion,
    ) -> PyRefMut<'_, Self> {
        if let Some(b) = slf.inner.take() {
            slf.inner = Some(b.region(region.into()));
        }
        slf
    }

    fn timeout(mut slf: PyRefMut<'_, Self>, seconds: u64) -> PyRefMut<'_, Self> {
        if let Some(b) = slf.inner.take() {
            slf.inner = Some(b.timeout(Duration::from_secs(seconds)));
        }
        slf
    }

    fn proxy(mut slf: PyRefMut<'_, Self>, proxy: String) -> PyRefMut<'_, Self> {
        if let Some(b) = slf.inner.take() {
            slf.inner = Some(b.proxy(proxy));
        }
        slf
    }

    fn cache(mut slf: PyRefMut<'_, Self>, ttl_seconds: u64) -> PyRefMut<'_, Self> {
        if let Some(b) = slf.inner.take() {
            slf.inner = Some(b.cache(Duration::from_secs(ttl_seconds)));
        }
        slf
    }

    fn logo(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        if let Some(b) = slf.inner.take() {
            slf.inner = Some(b.logo());
        }
        slf
    }

    fn build<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let b = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("builder already consumed")
        })?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let t = b.build().await.map_err(to_py_err)?;
            Ok(PyTicker { inner: Arc::new(t) })
        })
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTicker>()?;
    m.add_class::<PyTickerBuilder>()?;
    Ok(())
}

//! Top-level finance functions exposed on the `finance_query.finance` submodule.
//!
//! Mirrors the free functions in [`finance_query::finance`]: `search`,
//! `screener`, `trending`, `fear_and_greed`, `lookup`, `market_summary`,
//! `hours`, `sector`, `currencies`, `news`, `industry`, and `exchanges`.

use crate::error::to_py_err;
use crate::models::{
    PyFearAndGreed, PyNews, PyScreenerQuote, PyScreenerResults, PySearchQuote, PyTrendingQuote,
};
use finance_query::{
    PyCurrency, PyExchange, PyIndustryData, PyLookupResults, PyMarketHours, PyMarketSummaryQuote,
    PyRegion, PyScreener, PySector, PySectorData,
};
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

/// Search Yahoo Finance for symbols, news, and research matching `query`.
///
/// Currently uses `SearchOptions::default()`; richer options will be exposed
/// once `SearchOptions` itself has a Python wrapper. Returns just the quote
/// list — news/research-reports/total-time are deferred to a later task.
#[pyfunction]
fn search<'py>(py: Python<'py>, query: String) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let opts = finance_query::SearchOptions::default();
        let r = finance_query::finance::search(&query, &opts)
            .await
            .map_err(to_py_err)?;
        let py_vec: Vec<PySearchQuote> = r.quotes.0.into_iter().map(Into::into).collect();
        Ok(py_vec)
    })
}

/// Run a Yahoo Finance predefined screener (e.g. `Screener.DayGainers`).
///
/// `count` defaults to 25 to match Yahoo's typical screener page size.
#[pyfunction]
#[pyo3(signature = (screener, count=25))]
fn screener<'py>(py: Python<'py>, screener: PyScreener, count: u32) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let r = finance_query::finance::screener(screener.into(), count)
            .await
            .map_err(to_py_err)?;
        Ok(PyScreenerResults::from(r))
    })
}

/// Fetch the trending tickers for `region` (defaults to US when `None`).
#[pyfunction]
#[pyo3(signature = (region=None))]
fn trending<'py>(py: Python<'py>, region: Option<PyRegion>) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let r = finance_query::finance::trending(region.map(Into::into))
            .await
            .map_err(to_py_err)?;
        let py_vec: Vec<PyTrendingQuote> = r.into_iter().map(Into::into).collect();
        Ok(py_vec)
    })
}

/// Fetch the current CNN Fear & Greed Index from Alternative.me.
#[pyfunction]
fn fear_and_greed<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let r = finance_query::finance::fear_and_greed()
            .await
            .map_err(to_py_err)?;
        Ok(PyFearAndGreed::from(r))
    })
}

/// Look up Yahoo Finance symbols by name/ticker prefix (`query`).
///
/// Uses `LookupOptions::default()`; richer filtering will be exposed once
/// `LookupOptions` itself has a Python wrapper.
#[pyfunction]
fn lookup<'py>(py: Python<'py>, query: String) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let opts = finance_query::LookupOptions::default();
        let r = finance_query::finance::lookup(&query, &opts)
            .await
            .map_err(to_py_err)?;
        Ok(PyLookupResults::from(r))
    })
}

/// Fetch Yahoo Finance's market summary quotes for `region` (defaults to US).
#[pyfunction]
#[pyo3(signature = (region=None))]
fn market_summary<'py>(py: Python<'py>, region: Option<PyRegion>) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let r = finance_query::finance::market_summary(region.map(Into::into))
            .await
            .map_err(to_py_err)?;
        let py_vec: Vec<PyMarketSummaryQuote> = r.into_iter().map(Into::into).collect();
        Ok(py_vec)
    })
}

/// Fetch market hours for `region` (string code like "us"/"uk"; None = all).
#[pyfunction]
#[pyo3(signature = (region=None))]
fn hours<'py>(py: Python<'py>, region: Option<String>) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let r = finance_query::finance::hours(region.as_deref())
            .await
            .map_err(to_py_err)?;
        Ok(PyMarketHours::from(r))
    })
}

/// Fetch sector data for a given `Sector` enum value (e.g. `Sector.Technology`).
#[pyfunction]
fn sector<'py>(py: Python<'py>, sector_type: PySector) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let r = finance_query::finance::sector(sector_type.into())
            .await
            .map_err(to_py_err)?;
        Ok(PySectorData::from(r))
    })
}

/// Fetch Yahoo Finance's list of supported currencies.
#[pyfunction]
fn currencies<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let r = finance_query::finance::currencies()
            .await
            .map_err(to_py_err)?;
        let py_vec: Vec<PyCurrency> = r.into_iter().map(Into::into).collect();
        Ok(py_vec)
    })
}

/// Fetch the latest scraped Yahoo Finance news headlines.
#[pyfunction]
fn news<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let r = finance_query::finance::news().await.map_err(to_py_err)?;
        let py_vec: Vec<PyNews> = r.into_iter().map(Into::into).collect();
        Ok(py_vec)
    })
}

/// Fetch industry data for the given industry key (e.g. `"semiconductors"`).
#[pyfunction]
fn industry<'py>(py: Python<'py>, industry_key: String) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let r = finance_query::finance::industry(&industry_key)
            .await
            .map_err(to_py_err)?;
        Ok(PyIndustryData::from(r))
    })
}

/// Fetch Yahoo Finance's list of supported exchanges.
#[pyfunction]
fn exchanges<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let r = finance_query::finance::exchanges()
            .await
            .map_err(to_py_err)?;
        let py_vec: Vec<PyExchange> = r.into_iter().map(Into::into).collect();
        Ok(py_vec)
    })
}

/// Register the `finance` submodule and its function members on `parent`.
pub fn register(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent.py();
    let m = PyModule::new(py, "finance")?;
    m.add_function(wrap_pyfunction!(search, &m)?)?;
    m.add_function(wrap_pyfunction!(screener, &m)?)?;
    m.add_function(wrap_pyfunction!(trending, &m)?)?;
    m.add_function(wrap_pyfunction!(fear_and_greed, &m)?)?;
    m.add_function(wrap_pyfunction!(lookup, &m)?)?;
    m.add_function(wrap_pyfunction!(market_summary, &m)?)?;
    m.add_function(wrap_pyfunction!(hours, &m)?)?;
    m.add_function(wrap_pyfunction!(sector, &m)?)?;
    m.add_function(wrap_pyfunction!(currencies, &m)?)?;
    m.add_function(wrap_pyfunction!(news, &m)?)?;
    m.add_function(wrap_pyfunction!(industry, &m)?)?;
    m.add_function(wrap_pyfunction!(exchanges, &m)?)?;

    // Expose the response types on the submodule so users can introspect
    // them via e.g. `finance_query.finance.ScreenerResults`.
    m.add_class::<PyScreenerResults>()?;
    m.add_class::<PyScreenerQuote>()?;
    m.add_class::<PySearchQuote>()?;
    m.add_class::<PyTrendingQuote>()?;
    m.add_class::<PyFearAndGreed>()?;
    m.add_class::<PyLookupResults>()?;
    m.add_class::<PyMarketSummaryQuote>()?;
    m.add_class::<PyMarketHours>()?;
    m.add_class::<PySectorData>()?;
    m.add_class::<PyCurrency>()?;
    m.add_class::<PyNews>()?;
    m.add_class::<PyIndustryData>()?;
    m.add_class::<PyExchange>()?;

    parent.add_submodule(&m)?;
    Ok(())
}

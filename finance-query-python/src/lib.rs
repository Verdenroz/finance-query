//! Python bindings for the finance-query Rust library.
//!
//! See the design spec at `docs/superpowers/specs/2026-05-12-python-bindings-design.md`.

use pyo3::prelude::*;

mod backtest;
mod edgar;
mod enums;
mod error;
mod finance;
mod indicators;
mod logging_bridge;
mod models;
mod runtime;
mod strategy;
mod ticker;
mod tickers;

#[pymodule]
fn _finance_query(m: &Bound<'_, PyModule>) -> PyResult<()> {
    runtime::init_runtime()?;
    error::register(m)?;
    enums::register(m)?;
    indicators::register(m)?;
    strategy::register(m)?;
    ticker::register(m)?;
    tickers::register(m)?;
    finance::register(m)?;
    logging_bridge::register(m)?;
    edgar::register(m)?;
    m.add_class::<finance_query::PyFearGreedLabel>()?;
    m.add_class::<finance_query::PySentimentLabel>()?;
    m.add_class::<finance_query::PyProvider>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

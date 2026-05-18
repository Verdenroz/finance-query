//! Re-exports of the Python enum wrappers defined in finance-query.
//!
//! The enum mirror types (`PyInterval`, `PyTimeRange`, etc.) live in the core
//! `finance_query` crate (gated on `python` feature) so the `PyModel` derive
//! macro can resolve them from inside model files via `use crate::{...}`.
//! This module simply re-exports them and registers the classes with the
//! `_finance_query` Python module.

use ::pyo3::prelude::*;

pub use finance_query::{
    PyExchangeCode, PyFrequency, PyIndustry, PyInterval, PyRegion, PyScreener, PySector,
    PyStatementType, PyTimeRange, PyValueFormat,
};

pub fn register(m: &::pyo3::Bound<'_, ::pyo3::types::PyModule>) -> ::pyo3::PyResult<()> {
    m.add_class::<PyInterval>()?;
    m.add_class::<PyTimeRange>()?;
    m.add_class::<PyFrequency>()?;
    m.add_class::<PyStatementType>()?;
    m.add_class::<PyRegion>()?;
    m.add_class::<PyValueFormat>()?;
    m.add_class::<PySector>()?;
    m.add_class::<PyScreener>()?;
    m.add_class::<PyExchangeCode>()?;
    m.add_class::<PyIndustry>()?;
    Ok(())
}

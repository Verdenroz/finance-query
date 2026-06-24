//! Python wrappers for backtest result types.
//!
//! `BacktestResult` and `PortfolioResult` have deeply nested Rust types with enums,
//! newtypes, and skipped fields that resist the `PyModel` derive macro. These are
//! instead wrapped by serialising to a Python dict via `serde_json` + PyO3.

use finance_query::backtesting::{BacktestResult, portfolio::PortfolioResult};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

// ── PyBacktestResult ─────────────────────────────────────────────────────────

/// Python wrapper for `finance_query::backtesting::BacktestResult`.
///
/// Exposes the most-commonly-used scalar fields as getters; `to_dict()` gives
/// full access to every field (including nested structs / arrays) by
/// serialising the inner value through `serde_json`.
#[pyclass(frozen, name = "BacktestResult")]
#[derive(Debug)]
pub struct PyBacktestResult {
    inner: std::sync::Arc<BacktestResult>,
}

impl From<BacktestResult> for PyBacktestResult {
    fn from(r: BacktestResult) -> Self {
        Self {
            inner: std::sync::Arc::new(r),
        }
    }
}

#[pymethods]
impl PyBacktestResult {
    /// Symbol that was backtested.
    #[getter]
    fn symbol(&self) -> &str {
        &self.inner.symbol
    }

    /// Strategy name.
    #[getter]
    fn strategy_name(&self) -> &str {
        &self.inner.strategy_name
    }

    /// Start timestamp (Unix seconds).
    #[getter]
    fn start_timestamp(&self) -> i64 {
        self.inner.start_timestamp
    }

    /// End timestamp (Unix seconds).
    #[getter]
    fn end_timestamp(&self) -> i64 {
        self.inner.end_timestamp
    }

    /// Initial capital.
    #[getter]
    fn initial_capital(&self) -> f64 {
        self.inner.initial_capital
    }

    /// Final equity.
    #[getter]
    fn final_equity(&self) -> f64 {
        self.inner.final_equity
    }

    // ── Top-level metrics shortcuts ──────────────────────────────────────────

    #[getter]
    fn total_return_pct(&self) -> f64 {
        self.inner.metrics.total_return_pct
    }

    #[getter]
    fn annualized_return_pct(&self) -> f64 {
        self.inner.metrics.annualized_return_pct
    }

    #[getter]
    fn sharpe_ratio(&self) -> f64 {
        self.inner.metrics.sharpe_ratio
    }

    #[getter]
    fn sortino_ratio(&self) -> f64 {
        self.inner.metrics.sortino_ratio
    }

    #[getter]
    fn max_drawdown_pct(&self) -> f64 {
        self.inner.metrics.max_drawdown_pct
    }

    #[getter]
    fn win_rate(&self) -> f64 {
        self.inner.metrics.win_rate
    }

    #[getter]
    fn profit_factor(&self) -> f64 {
        self.inner.metrics.profit_factor
    }

    #[getter]
    fn total_trades(&self) -> usize {
        self.inner.metrics.total_trades
    }

    #[getter]
    fn winning_trades(&self) -> usize {
        self.inner.metrics.winning_trades
    }

    #[getter]
    fn losing_trades(&self) -> usize {
        self.inner.metrics.losing_trades
    }

    #[getter]
    fn calmar_ratio(&self) -> f64 {
        self.inner.metrics.calmar_ratio
    }

    #[getter]
    fn sqn(&self) -> f64 {
        self.inner.metrics.sqn
    }

    #[getter]
    fn expectancy(&self) -> f64 {
        self.inner.metrics.expectancy
    }

    #[getter]
    fn diagnostics(&self) -> Vec<String> {
        self.inner.diagnostics.clone()
    }

    // ── Full serialised access ───────────────────────────────────────────────

    /// Return the entire result as a Python dict (deep-serialised via JSON).
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let value = serde_json::to_value(&*self.inner)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        json_value_to_dict(py, &value)
    }

    fn __repr__(&self) -> String {
        format!(
            "BacktestResult(symbol={:?}, strategy={:?}, trades={}, return={:.2}%)",
            self.inner.symbol,
            self.inner.strategy_name,
            self.inner.metrics.total_trades,
            self.inner.metrics.total_return_pct,
        )
    }
}

// ── PyPortfolioResult ─────────────────────────────────────────────────────────

/// Python wrapper for `finance_query::backtesting::portfolio::PortfolioResult`.
#[pyclass(frozen, name = "PortfolioResult")]
#[derive(Debug)]
pub struct PyPortfolioResult {
    inner: std::sync::Arc<PortfolioResult>,
}

impl From<PortfolioResult> for PyPortfolioResult {
    fn from(r: PortfolioResult) -> Self {
        Self {
            inner: std::sync::Arc::new(r),
        }
    }
}

#[pymethods]
impl PyPortfolioResult {
    #[getter]
    fn initial_capital(&self) -> f64 {
        self.inner.initial_capital
    }

    #[getter]
    fn final_equity(&self) -> f64 {
        self.inner.final_equity
    }

    #[getter]
    fn total_return_pct(&self) -> f64 {
        self.inner.portfolio_metrics.total_return_pct
    }

    #[getter]
    fn sharpe_ratio(&self) -> f64 {
        self.inner.portfolio_metrics.sharpe_ratio
    }

    #[getter]
    fn max_drawdown_pct(&self) -> f64 {
        self.inner.portfolio_metrics.max_drawdown_pct
    }

    #[getter]
    fn total_trades(&self) -> usize {
        self.inner.portfolio_metrics.total_trades
    }

    /// Symbols included in this portfolio result.
    #[getter]
    fn symbols(&self) -> Vec<String> {
        self.inner.symbols.keys().cloned().collect()
    }

    /// Per-symbol results as a dict[str, BacktestResult].
    fn symbol_results<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        for (sym, r) in &self.inner.symbols {
            d.set_item(sym, PyBacktestResult::from(r.clone()))?;
        }
        Ok(d)
    }

    /// Return the entire result as a Python dict (deep-serialised via JSON).
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let value = serde_json::to_value(&*self.inner)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        json_value_to_dict(py, &value)
    }

    fn __repr__(&self) -> String {
        format!(
            "PortfolioResult(symbols={:?}, trades={}, return={:.2}%)",
            self.inner.symbols.keys().collect::<Vec<_>>(),
            self.inner.portfolio_metrics.total_trades,
            self.inner.portfolio_metrics.total_return_pct,
        )
    }
}

// ── JSON → Python dict helper ─────────────────────────────────────────────────

/// Recursively convert a `serde_json::Value` into a Python `dict` / `list` / scalar.
fn json_value_to_pyobject<'py>(
    py: Python<'py>,
    value: &serde_json::Value,
) -> PyResult<Bound<'py, PyAny>> {
    use serde_json::Value;
    match value {
        Value::Null => Ok(py.None().into_bound(py)),
        Value::Bool(b) => Ok((*b).into_pyobject(py)?.to_owned().into_any()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.into_any())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.into_any())
            } else {
                Ok(n.to_string().into_pyobject(py)?.into_any())
            }
        }
        Value::String(s) => Ok(s.as_str().into_pyobject(py)?.into_any()),
        Value::Array(arr) => {
            let list = PyList::empty(py);
            for item in arr {
                list.append(json_value_to_pyobject(py, item)?)?;
            }
            Ok(list.into_any())
        }
        Value::Object(map) => {
            let d = PyDict::new(py);
            for (k, v) in map {
                d.set_item(k, json_value_to_pyobject(py, v)?)?;
            }
            Ok(d.into_any())
        }
    }
}

fn json_value_to_dict<'py>(
    py: Python<'py>,
    value: &serde_json::Value,
) -> PyResult<Bound<'py, PyDict>> {
    let obj = json_value_to_pyobject(py, value)?;
    obj.downcast::<PyDict>().map(|d| d.clone()).map_err(|_| {
        pyo3::exceptions::PyTypeError::new_err("expected dict from JSON serialization")
    })
}

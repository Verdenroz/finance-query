//! Python bindings for `finance_query::indicators::Indicator` and `IndicatorResult`.
//!
//! Because `Indicator` is a data-carrying enum (variants like `Sma(usize)`, `Macd { fast, slow,
//! signal }`, …), it cannot be a plain `eq_int` PyO3 enum. Instead we expose a `PyIndicator`
//! frozen pyclass with one `#[staticmethod]` constructor per variant.
//!
//! `IndicatorResult` is also an enum, so `PyModel` cannot be derived on it. We hand-write
//! `PyIndicatorResult` as a pyclass whose getters expose a tagged-union view: a `kind` string
//! discriminant plus variant-specific list/dict accessors.

use finance_query::indicators::{Indicator, IndicatorResult};
use pyo3::prelude::*;
use pyo3::types::PyDict;

// ============================================================================
// PyIndicator — one staticmethod per variant
// ============================================================================

/// Python-facing wrapper for the Rust `Indicator` enum.
///
/// Construct an indicator value with the appropriate static factory method,
/// then pass it to `Ticker.indicator()`.
#[pyclass(frozen, name = "Indicator")]
#[derive(Clone)]
pub struct PyIndicator(pub Indicator);

#[pymethods]
impl PyIndicator {
    // ---- Moving Averages ----
    #[staticmethod]
    fn sma(period: usize) -> Self {
        Self(Indicator::Sma(period))
    }

    #[staticmethod]
    fn ema(period: usize) -> Self {
        Self(Indicator::Ema(period))
    }

    #[staticmethod]
    fn wma(period: usize) -> Self {
        Self(Indicator::Wma(period))
    }

    #[staticmethod]
    fn dema(period: usize) -> Self {
        Self(Indicator::Dema(period))
    }

    #[staticmethod]
    fn tema(period: usize) -> Self {
        Self(Indicator::Tema(period))
    }

    #[staticmethod]
    fn hma(period: usize) -> Self {
        Self(Indicator::Hma(period))
    }

    #[staticmethod]
    fn vwma(period: usize) -> Self {
        Self(Indicator::Vwma(period))
    }

    /// Arnaud Legoux Moving Average.
    ///
    /// * `period` — window size
    /// * `offset` — Gaussian offset (typically 0.85)
    /// * `sigma`  — Gaussian sigma (typically 6.0)
    #[staticmethod]
    fn alma(period: usize, offset: f64, sigma: f64) -> Self {
        Self(Indicator::Alma {
            period,
            offset,
            sigma,
        })
    }

    #[staticmethod]
    fn mcginley_dynamic(period: usize) -> Self {
        Self(Indicator::McginleyDynamic(period))
    }

    // ---- Momentum Oscillators ----
    #[staticmethod]
    fn rsi(period: usize) -> Self {
        Self(Indicator::Rsi(period))
    }

    #[staticmethod]
    fn cci(period: usize) -> Self {
        Self(Indicator::Cci(period))
    }

    #[staticmethod]
    fn williams_r(period: usize) -> Self {
        Self(Indicator::WilliamsR(period))
    }

    #[staticmethod]
    fn roc(period: usize) -> Self {
        Self(Indicator::Roc(period))
    }

    #[staticmethod]
    fn momentum(period: usize) -> Self {
        Self(Indicator::Momentum(period))
    }

    #[staticmethod]
    fn cmo(period: usize) -> Self {
        Self(Indicator::Cmo(period))
    }

    /// Stochastic Oscillator.
    ///
    /// * `k_period` — raw %K look-back
    /// * `k_slow`   — %K slowing
    /// * `d_period` — %D SMA period
    #[staticmethod]
    fn stochastic(k_period: usize, k_slow: usize, d_period: usize) -> Self {
        Self(Indicator::Stochastic {
            k_period,
            k_slow,
            d_period,
        })
    }

    /// Stochastic RSI.
    #[staticmethod]
    fn stochastic_rsi(
        rsi_period: usize,
        stoch_period: usize,
        k_period: usize,
        d_period: usize,
    ) -> Self {
        Self(Indicator::StochasticRsi {
            rsi_period,
            stoch_period,
            k_period,
            d_period,
        })
    }

    /// Awesome Oscillator.
    #[staticmethod]
    fn awesome_oscillator(fast: usize, slow: usize) -> Self {
        Self(Indicator::AwesomeOscillator { fast, slow })
    }

    /// Coppock Curve.
    #[staticmethod]
    fn coppock_curve(wma_period: usize, long_roc: usize, short_roc: usize) -> Self {
        Self(Indicator::CoppockCurve {
            wma_period,
            long_roc,
            short_roc,
        })
    }

    // ---- Trend Indicators ----
    #[staticmethod]
    fn macd(fast: usize, slow: usize, signal: usize) -> Self {
        Self(Indicator::Macd { fast, slow, signal })
    }

    #[staticmethod]
    fn adx(period: usize) -> Self {
        Self(Indicator::Adx(period))
    }

    #[staticmethod]
    fn aroon(period: usize) -> Self {
        Self(Indicator::Aroon(period))
    }

    /// SuperTrend.
    ///
    /// * `period`     — ATR period
    /// * `multiplier` — ATR multiplier
    #[staticmethod]
    fn supertrend(period: usize, multiplier: f64) -> Self {
        Self(Indicator::Supertrend { period, multiplier })
    }

    /// Ichimoku Cloud.
    #[staticmethod]
    fn ichimoku(conversion: usize, base: usize, lagging: usize, displacement: usize) -> Self {
        Self(Indicator::Ichimoku {
            conversion,
            base,
            lagging,
            displacement,
        })
    }

    /// Parabolic SAR.
    #[staticmethod]
    fn parabolic_sar(step: f64, max: f64) -> Self {
        Self(Indicator::ParabolicSar { step, max })
    }

    // ---- Volatility Indicators ----
    #[staticmethod]
    fn bollinger(period: usize, std_dev: f64) -> Self {
        Self(Indicator::Bollinger { period, std_dev })
    }

    #[staticmethod]
    fn atr(period: usize) -> Self {
        Self(Indicator::Atr(period))
    }

    /// Keltner Channels.
    #[staticmethod]
    fn keltner_channels(period: usize, multiplier: f64, atr_period: usize) -> Self {
        Self(Indicator::KeltnerChannels {
            period,
            multiplier,
            atr_period,
        })
    }

    #[staticmethod]
    fn donchian_channels(period: usize) -> Self {
        Self(Indicator::DonchianChannels(period))
    }

    #[staticmethod]
    fn true_range() -> Self {
        Self(Indicator::TrueRange)
    }

    #[staticmethod]
    fn choppiness_index(period: usize) -> Self {
        Self(Indicator::ChoppinessIndex(period))
    }

    // ---- Volume Indicators ----
    #[staticmethod]
    fn obv() -> Self {
        Self(Indicator::Obv)
    }

    #[staticmethod]
    fn vwap() -> Self {
        Self(Indicator::Vwap)
    }

    #[staticmethod]
    fn mfi(period: usize) -> Self {
        Self(Indicator::Mfi(period))
    }

    #[staticmethod]
    fn cmf(period: usize) -> Self {
        Self(Indicator::Cmf(period))
    }

    #[staticmethod]
    fn chaikin_oscillator() -> Self {
        Self(Indicator::ChaikinOscillator)
    }

    #[staticmethod]
    fn accumulation_distribution() -> Self {
        Self(Indicator::AccumulationDistribution)
    }

    // ---- Power Indicators ----
    #[staticmethod]
    fn bull_bear_power(period: usize) -> Self {
        Self(Indicator::BullBearPower(period))
    }

    #[staticmethod]
    fn elder_ray(period: usize) -> Self {
        Self(Indicator::ElderRay(period))
    }

    /// Balance of Power.
    ///
    /// Pass `None` (omit argument in Python) for unsmoothed, or a period for SMA smoothing.
    #[staticmethod]
    #[pyo3(signature = (period=None))]
    fn balance_of_power(period: Option<usize>) -> Self {
        Self(Indicator::BalanceOfPower(period))
    }

    fn __repr__(&self) -> String {
        format!("Indicator({:?})", self.0)
    }
}

// ============================================================================
// PyIndicatorResult — hand-written wrapper for the IndicatorResult enum
// ============================================================================

/// Python-facing wrapper for the Rust `IndicatorResult` enum.
///
/// Access `.kind` to determine which variant is present, then use the
/// appropriate accessor:
///
/// * `kind == "Series"` → `.series` → `list[float | None]`
/// * `kind == "Macd"` → `.macd_line`, `.signal_line`, `.histogram`
/// * `kind == "Bollinger"` → `.upper`, `.middle`, `.lower`
/// * `kind == "Stochastic"` → `.k`, `.d`
/// * `kind == "Aroon"` → `.aroon_up`, `.aroon_down`
/// * `kind == "SuperTrend"` → `.value`, `.is_uptrend`
/// * `kind == "Ichimoku"` → `.conversion_line`, `.base_line`, `.leading_span_a`,
///                           `.leading_span_b`, `.lagging_span`
/// * `kind == "BullBearPower"` / `"ElderRay"` → `.bull_power`, `.bear_power`
/// * `kind == "Keltner"` / `"Donchian"` → `.upper`, `.middle`, `.lower`
#[pyclass(frozen, name = "IndicatorResult")]
#[derive(Clone)]
pub struct PyIndicatorResult {
    inner: IndicatorResult,
}

impl From<IndicatorResult> for PyIndicatorResult {
    fn from(r: IndicatorResult) -> Self {
        Self { inner: r }
    }
}

#[pymethods]
impl PyIndicatorResult {
    /// Discriminant string identifying which variant this result holds.
    #[getter]
    fn kind(&self) -> &'static str {
        match &self.inner {
            IndicatorResult::Series(_) => "Series",
            IndicatorResult::Macd(_) => "Macd",
            IndicatorResult::Bollinger(_) => "Bollinger",
            IndicatorResult::Stochastic(_) => "Stochastic",
            IndicatorResult::Aroon(_) => "Aroon",
            IndicatorResult::SuperTrend(_) => "SuperTrend",
            IndicatorResult::Ichimoku(_) => "Ichimoku",
            IndicatorResult::BullBearPower(_) => "BullBearPower",
            IndicatorResult::ElderRay(_) => "ElderRay",
            IndicatorResult::Keltner(_) => "Keltner",
            IndicatorResult::Donchian(_) => "Donchian",
            // IndicatorResult is #[non_exhaustive]; handle future variants gracefully.
            _ => "Unknown",
        }
    }

    // ---- Series ----
    #[getter]
    fn series(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Series(v) => Some(v.clone()),
            _ => None,
        }
    }

    // ---- Macd ----
    #[getter]
    fn macd_line(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Macd(m) => Some(m.macd_line.clone()),
            _ => None,
        }
    }

    #[getter]
    fn signal_line(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Macd(m) => Some(m.signal_line.clone()),
            _ => None,
        }
    }

    #[getter]
    fn histogram(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Macd(m) => Some(m.histogram.clone()),
            _ => None,
        }
    }

    // ---- Bollinger / Keltner / Donchian (share upper/middle/lower) ----
    #[getter]
    fn upper(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Bollinger(b) => Some(b.upper.clone()),
            IndicatorResult::Keltner(k) => Some(k.upper.clone()),
            IndicatorResult::Donchian(d) => Some(d.upper.clone()),
            _ => None,
        }
    }

    #[getter]
    fn middle(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Bollinger(b) => Some(b.middle.clone()),
            IndicatorResult::Keltner(k) => Some(k.middle.clone()),
            IndicatorResult::Donchian(d) => Some(d.middle.clone()),
            _ => None,
        }
    }

    #[getter]
    fn lower(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Bollinger(b) => Some(b.lower.clone()),
            IndicatorResult::Keltner(k) => Some(k.lower.clone()),
            IndicatorResult::Donchian(d) => Some(d.lower.clone()),
            _ => None,
        }
    }

    // ---- Stochastic ----
    #[getter]
    fn k(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Stochastic(s) => Some(s.k.clone()),
            _ => None,
        }
    }

    #[getter]
    fn d(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Stochastic(s) => Some(s.d.clone()),
            _ => None,
        }
    }

    // ---- Aroon ----
    #[getter]
    fn aroon_up(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Aroon(a) => Some(a.aroon_up.clone()),
            _ => None,
        }
    }

    #[getter]
    fn aroon_down(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Aroon(a) => Some(a.aroon_down.clone()),
            _ => None,
        }
    }

    // ---- SuperTrend ----
    #[getter]
    fn value(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::SuperTrend(s) => Some(s.value.clone()),
            _ => None,
        }
    }

    #[getter]
    fn is_uptrend(&self) -> Option<Vec<Option<bool>>> {
        match &self.inner {
            IndicatorResult::SuperTrend(s) => Some(s.is_uptrend.clone()),
            _ => None,
        }
    }

    // ---- Ichimoku ----
    #[getter]
    fn conversion_line(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Ichimoku(i) => Some(i.conversion_line.clone()),
            _ => None,
        }
    }

    #[getter]
    fn base_line(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Ichimoku(i) => Some(i.base_line.clone()),
            _ => None,
        }
    }

    #[getter]
    fn leading_span_a(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Ichimoku(i) => Some(i.leading_span_a.clone()),
            _ => None,
        }
    }

    #[getter]
    fn leading_span_b(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Ichimoku(i) => Some(i.leading_span_b.clone()),
            _ => None,
        }
    }

    #[getter]
    fn lagging_span(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::Ichimoku(i) => Some(i.lagging_span.clone()),
            _ => None,
        }
    }

    // ---- BullBearPower / ElderRay (share bull_power / bear_power) ----
    #[getter]
    fn bull_power(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::BullBearPower(b) => Some(b.bull_power.clone()),
            IndicatorResult::ElderRay(e) => Some(e.bull_power.clone()),
            _ => None,
        }
    }

    #[getter]
    fn bear_power(&self) -> Option<Vec<Option<f64>>> {
        match &self.inner {
            IndicatorResult::BullBearPower(b) => Some(b.bear_power.clone()),
            IndicatorResult::ElderRay(e) => Some(e.bear_power.clone()),
            _ => None,
        }
    }

    /// Serialize the result to a plain Python dict.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        d.set_item("kind", self.kind())?;
        match &self.inner {
            IndicatorResult::Series(v) => {
                d.set_item("series", v.clone())?;
            }
            IndicatorResult::Macd(m) => {
                d.set_item("macd_line", m.macd_line.clone())?;
                d.set_item("signal_line", m.signal_line.clone())?;
                d.set_item("histogram", m.histogram.clone())?;
            }
            IndicatorResult::Bollinger(b) => {
                d.set_item("upper", b.upper.clone())?;
                d.set_item("middle", b.middle.clone())?;
                d.set_item("lower", b.lower.clone())?;
            }
            IndicatorResult::Stochastic(s) => {
                d.set_item("k", s.k.clone())?;
                d.set_item("d", s.d.clone())?;
            }
            IndicatorResult::Aroon(a) => {
                d.set_item("aroon_up", a.aroon_up.clone())?;
                d.set_item("aroon_down", a.aroon_down.clone())?;
            }
            IndicatorResult::SuperTrend(s) => {
                d.set_item("value", s.value.clone())?;
                d.set_item("is_uptrend", s.is_uptrend.clone())?;
            }
            IndicatorResult::Ichimoku(i) => {
                d.set_item("conversion_line", i.conversion_line.clone())?;
                d.set_item("base_line", i.base_line.clone())?;
                d.set_item("leading_span_a", i.leading_span_a.clone())?;
                d.set_item("leading_span_b", i.leading_span_b.clone())?;
                d.set_item("lagging_span", i.lagging_span.clone())?;
            }
            IndicatorResult::BullBearPower(b) => {
                d.set_item("bull_power", b.bull_power.clone())?;
                d.set_item("bear_power", b.bear_power.clone())?;
            }
            IndicatorResult::ElderRay(e) => {
                d.set_item("bull_power", e.bull_power.clone())?;
                d.set_item("bear_power", e.bear_power.clone())?;
            }
            IndicatorResult::Keltner(k) => {
                d.set_item("upper", k.upper.clone())?;
                d.set_item("middle", k.middle.clone())?;
                d.set_item("lower", k.lower.clone())?;
            }
            IndicatorResult::Donchian(dc) => {
                d.set_item("upper", dc.upper.clone())?;
                d.set_item("middle", dc.middle.clone())?;
                d.set_item("lower", dc.lower.clone())?;
            }
            // IndicatorResult is #[non_exhaustive]; future variants produce an empty dict with kind.
            _ => {}
        }
        Ok(d)
    }

    fn __repr__(&self) -> String {
        format!("IndicatorResult(kind={:?})", self.kind())
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIndicator>()?;
    m.add_class::<PyIndicatorResult>()?;
    Ok(())
}

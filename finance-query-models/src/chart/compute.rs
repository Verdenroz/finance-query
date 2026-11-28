use super::Chart;
use finance_query_types::indicators::{self, Indicator, IndicatorResult, Result};

/// Compute a single [`Indicator`] over a [`Chart`].
///
/// Shared dispatch used by `Ticker::indicator` and the domain handles
/// (`ForexPair`, `CryptoCoin`, `Index`, `FuturesContract`, `Commodity`) so the
/// indicator-selection logic lives in exactly one place.
#[doc(hidden)]
pub fn compute_indicator(indicator: Indicator, chart: &Chart) -> Result<IndicatorResult> {
    let o = chart.open_prices();
    let h = chart.high_prices();
    let l = chart.low_prices();
    let c = chart.close_prices();
    let v = chart.volumes();
    Ok(match indicator {
        Indicator::Sma(p) => IndicatorResult::Series(chart.sma(p)),
        Indicator::Ema(p) => IndicatorResult::Series(chart.ema(p)),
        Indicator::Rsi(p) => IndicatorResult::Series(chart.rsi(p)?),
        Indicator::Macd { fast, slow, signal } => {
            IndicatorResult::Macd(chart.macd(fast, slow, signal)?)
        }
        Indicator::Bollinger { period, std_dev } => {
            IndicatorResult::Bollinger(chart.bollinger_bands(period, std_dev)?)
        }
        Indicator::Atr(p) => IndicatorResult::Series(chart.atr(p)?),
        Indicator::Vwap => IndicatorResult::Series(indicators::vwap(&h, &l, &c, &v)?),
        Indicator::Wma(p) => IndicatorResult::Series(indicators::wma(&c, p)?),
        Indicator::Obv => IndicatorResult::Series(indicators::obv(&c, &v)?),
        Indicator::Dema(p) => IndicatorResult::Series(indicators::dema(&c, p)?),
        Indicator::Tema(p) => IndicatorResult::Series(indicators::tema(&c, p)?),
        Indicator::Hma(p) => IndicatorResult::Series(indicators::hma(&c, p)?),
        Indicator::Vwma(p) => IndicatorResult::Series(indicators::vwma(&c, &v, p)?),
        Indicator::Alma {
            period,
            offset,
            sigma,
        } => IndicatorResult::Series(indicators::alma(&c, period, offset, sigma)?),
        Indicator::McginleyDynamic(p) => {
            IndicatorResult::Series(indicators::mcginley_dynamic(&c, p)?)
        }
        Indicator::Stochastic {
            k_period,
            k_slow,
            d_period,
        } => IndicatorResult::Stochastic(indicators::stochastic(
            &h, &l, &c, k_period, k_slow, d_period,
        )?),
        Indicator::StochasticRsi {
            rsi_period,
            stoch_period,
            k_period,
            d_period,
        } => IndicatorResult::Stochastic(indicators::stochastic_rsi(
            &c,
            rsi_period,
            stoch_period,
            k_period,
            d_period,
        )?),
        Indicator::Cci(p) => IndicatorResult::Series(indicators::cci(&h, &l, &c, p)?),
        Indicator::WilliamsR(p) => IndicatorResult::Series(indicators::williams_r(&h, &l, &c, p)?),
        Indicator::Roc(p) => IndicatorResult::Series(indicators::roc(&c, p)?),
        Indicator::Momentum(p) => IndicatorResult::Series(indicators::momentum(&c, p)?),
        Indicator::Cmo(p) => IndicatorResult::Series(indicators::cmo(&c, p)?),
        Indicator::AwesomeOscillator { fast, slow } => {
            IndicatorResult::Series(indicators::awesome_oscillator(&h, &l, fast, slow)?)
        }
        Indicator::CoppockCurve {
            long_roc,
            short_roc,
            wma_period,
        } => IndicatorResult::Series(indicators::coppock_curve(
            &c, long_roc, short_roc, wma_period,
        )?),
        Indicator::Adx(p) => IndicatorResult::Series(indicators::adx(&h, &l, &c, p)?),
        Indicator::Aroon(p) => IndicatorResult::Aroon(indicators::aroon(&h, &l, p)?),
        Indicator::Supertrend { period, multiplier } => {
            IndicatorResult::SuperTrend(indicators::supertrend(&h, &l, &c, period, multiplier)?)
        }
        Indicator::Ichimoku {
            conversion,
            base,
            lagging,
            displacement,
        } => IndicatorResult::Ichimoku(indicators::ichimoku(
            &h,
            &l,
            &c,
            conversion,
            base,
            lagging,
            displacement,
        )?),
        Indicator::ParabolicSar { step, max } => {
            IndicatorResult::Series(indicators::parabolic_sar(&h, &l, &c, step, max)?)
        }
        Indicator::BullBearPower(p) => {
            IndicatorResult::BullBearPower(indicators::bull_bear_power(&h, &l, &c, p)?)
        }
        Indicator::ElderRay(p) => IndicatorResult::ElderRay(indicators::elder_ray(&h, &l, &c, p)?),
        Indicator::KeltnerChannels {
            period,
            multiplier,
            atr_period,
        } => IndicatorResult::Keltner(indicators::keltner_channels(
            &h, &l, &c, period, atr_period, multiplier,
        )?),
        Indicator::DonchianChannels(p) => {
            IndicatorResult::Donchian(indicators::donchian_channels(&h, &l, p)?)
        }
        Indicator::TrueRange => IndicatorResult::Series(indicators::true_range(&h, &l, &c)?),
        Indicator::ChoppinessIndex(p) => {
            IndicatorResult::Series(indicators::choppiness_index(&h, &l, &c, p)?)
        }
        Indicator::Mfi(p) => IndicatorResult::Series(indicators::mfi(&h, &l, &c, &v, p)?),
        Indicator::Cmf(p) => IndicatorResult::Series(indicators::cmf(&h, &l, &c, &v, p)?),
        Indicator::ChaikinOscillator => {
            IndicatorResult::Series(indicators::chaikin_oscillator(&h, &l, &c, &v)?)
        }
        Indicator::AccumulationDistribution => {
            IndicatorResult::Series(indicators::accumulation_distribution(&h, &l, &c, &v)?)
        }
        Indicator::BalanceOfPower(p) => {
            IndicatorResult::Series(indicators::balance_of_power(&o, &h, &l, &c, p)?)
        }
        Indicator::PivotPointsStandard => {
            IndicatorResult::PivotPoints(indicators::pivot_points(&h, &l, &c)?)
        }
        Indicator::PivotPointsFibonacci => {
            IndicatorResult::PivotPoints(indicators::fibonacci_pivot_points(&h, &l, &c)?)
        }
        Indicator::HeikinAshi => {
            IndicatorResult::HeikinAshi(indicators::heikin_ashi(&chart.candles)?)
        }
        Indicator::ZigZag(deviation_pct) => {
            IndicatorResult::ZigZag(indicators::zigzag(&h, &l, deviation_pct)?)
        }
        Indicator::FibonacciRetracement(period) => IndicatorResult::FibonacciRetracement(
            indicators::fibonacci_retracement(&h, &l, period)?,
        ),
        other => {
            return Err(indicators::IndicatorError::InvalidPeriod(format!(
                "unsupported indicator {other:?}"
            )));
        }
    })
}

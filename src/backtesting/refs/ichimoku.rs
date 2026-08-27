use crate::backtesting::strategy::StrategyContext;
use crate::indicators::Indicator;

use super::IndicatorRef;

/// Ichimoku Cloud configuration.
#[derive(Debug, Clone, Copy)]
pub struct IchimokuConfig {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
}

impl IchimokuConfig {
    /// Get the Tenkan-sen (Conversion Line) reference.
    pub fn conversion_line(&self) -> IchimokuConversionRef {
        IchimokuConversionRef::new(self.conversion, self.base, self.lagging, self.displacement)
    }

    /// Get the Kijun-sen (Base Line) reference.
    pub fn base_line(&self) -> IchimokuBaseRef {
        IchimokuBaseRef::new(self.conversion, self.base, self.lagging, self.displacement)
    }

    /// Get the Senkou Span A (Leading Span A) reference.
    pub fn leading_span_a(&self) -> IchimokuLeadingARef {
        IchimokuLeadingARef::new(self.conversion, self.base, self.lagging, self.displacement)
    }

    /// Get the Senkou Span B (Leading Span B) reference.
    pub fn leading_span_b(&self) -> IchimokuLeadingBRef {
        IchimokuLeadingBRef::new(self.conversion, self.base, self.lagging, self.displacement)
    }

    /// Get the Chikou Span (Lagging Span) reference.
    pub fn lagging_span(&self) -> IchimokuLaggingRef {
        IchimokuLaggingRef::new(self.conversion, self.base, self.lagging, self.displacement)
    }
}

/// Create an Ichimoku Cloud configuration with default periods (9, 26, 26, 26).
///
/// Senkou Span B is not independently configurable here; it is always `2 * base`.
#[inline]
pub fn ichimoku() -> IchimokuConfig {
    IchimokuConfig {
        conversion: 9,
        base: 26,
        lagging: 26,
        displacement: 26,
    }
}

/// Create an Ichimoku Cloud configuration with custom periods.
///
/// `lagging` is the Chikou Span back-displacement, not a Senkou Span B
/// period — Senkou Span B is always `2 * base` and is not independently
/// configurable here.
#[inline]
pub fn ichimoku_custom(
    conversion: usize,
    base: usize,
    lagging: usize,
    displacement: usize,
) -> IchimokuConfig {
    IchimokuConfig {
        conversion,
        base,
        lagging,
        displacement,
    }
}

/// Ichimoku Conversion Line (Tenkan-sen) reference.
#[derive(Debug, Clone)]
pub struct IchimokuConversionRef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
    key: String,
}

impl IchimokuConversionRef {
    fn new(conversion: usize, base: usize, lagging: usize, displacement: usize) -> Self {
        Self {
            conversion,
            base,
            lagging,
            displacement,
            key: format!("ichimoku_conversion_{conversion}_{base}_{lagging}_{displacement}"),
        }
    }
}

impl IndicatorRef for IchimokuConversionRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Ichimoku Base Line (Kijun-sen) reference.
#[derive(Debug, Clone)]
pub struct IchimokuBaseRef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
    key: String,
}

impl IchimokuBaseRef {
    fn new(conversion: usize, base: usize, lagging: usize, displacement: usize) -> Self {
        Self {
            conversion,
            base,
            lagging,
            displacement,
            key: format!("ichimoku_base_{conversion}_{base}_{lagging}_{displacement}"),
        }
    }
}

impl IndicatorRef for IchimokuBaseRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Ichimoku Leading Span A (Senkou Span A) reference.
#[derive(Debug, Clone)]
pub struct IchimokuLeadingARef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
    key: String,
}

impl IchimokuLeadingARef {
    fn new(conversion: usize, base: usize, lagging: usize, displacement: usize) -> Self {
        Self {
            conversion,
            base,
            lagging,
            displacement,
            key: format!("ichimoku_leading_a_{conversion}_{base}_{lagging}_{displacement}"),
        }
    }
}

impl IndicatorRef for IchimokuLeadingARef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Ichimoku Leading Span B (Senkou Span B) reference.
#[derive(Debug, Clone)]
pub struct IchimokuLeadingBRef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
    key: String,
}

impl IchimokuLeadingBRef {
    fn new(conversion: usize, base: usize, lagging: usize, displacement: usize) -> Self {
        Self {
            conversion,
            base,
            lagging,
            displacement,
            key: format!("ichimoku_leading_b_{conversion}_{base}_{lagging}_{displacement}"),
        }
    }
}

impl IndicatorRef for IchimokuLeadingBRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Chikou Span reference: the close from `lagging` bars ago.
///
/// This is the price level the chikou span is compared against at decision
/// time, not the plot-aligned span itself — that array is stored shifted
/// forward for charting (slot `j` holds `close[j + lagging]`) and would leak
/// future closes if read directly at the current bar.
///
/// Reads base-timeframe candles directly, so inside an `htf()` wrapper it
/// still lags by base bars, not HTF bars.
#[derive(Debug, Clone)]
pub struct IchimokuLaggingRef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
    key: String,
}

impl IchimokuLaggingRef {
    fn new(conversion: usize, base: usize, lagging: usize, displacement: usize) -> Self {
        Self {
            conversion,
            base,
            lagging,
            displacement,
            key: format!("ichimoku_lagging_{conversion}_{base}_{lagging}_{displacement}"),
        }
    }
}

impl IndicatorRef for IchimokuLaggingRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![] // computed directly from candle history, not the plot-aligned array
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        let idx = ctx.index.checked_sub(self.lagging)?;
        Some(ctx.candles[idx].close)
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        let idx = ctx.index.checked_sub(self.lagging + 1)?;
        Some(ctx.candles[idx].close)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::config::BacktestConfig;
    use crate::backtesting::engine::BacktestEngine;
    use crate::backtesting::refs::{IndicatorRefExt, price};
    use crate::backtesting::strategy::StrategyBuilder;
    use crate::models::chart::Candle;
    use std::collections::HashMap;

    #[test]
    fn test_ichimoku_keys() {
        let ich = ichimoku();
        assert_eq!(
            ich.conversion_line().key(),
            "ichimoku_conversion_9_26_26_26"
        );
        assert_eq!(ich.base_line().key(), "ichimoku_base_9_26_26_26");
        assert_eq!(ich.leading_span_a().key(), "ichimoku_leading_a_9_26_26_26");
        assert_eq!(ich.leading_span_b().key(), "ichimoku_leading_b_9_26_26_26");
        assert_eq!(ich.lagging_span().key(), "ichimoku_lagging_9_26_26_26");
    }

    fn flat_candles(closes: &[f64]) -> Vec<Candle> {
        closes
            .iter()
            .enumerate()
            .map(|(i, &c)| Candle {
                timestamp: i as i64 * 86_400,
                open: c,
                high: c,
                low: c,
                close: c,
                volume: 1_000,
                adj_close: Some(c),
                provider_id: None,
            })
            .collect()
    }

    #[test]
    fn test_lagging_ref_reads_close_from_lagging_bars_ago() {
        let closes: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let candles = flat_candles(&closes);
        let indicators = HashMap::new();
        let lagging_ref = ichimoku_custom(9, 26, 5, 26).lagging_span();

        let ctx_at = |index: usize| StrategyContext {
            candles: &candles,
            index,
            position: None,
            equity: 0.0,
            indicators: &indicators,
            extremes: None,
            indicator_index: None,
        };

        assert_eq!(lagging_ref.value(&ctx_at(4)), None);
        assert_eq!(lagging_ref.value(&ctx_at(5)), Some(closes[0]));
        assert_eq!(lagging_ref.value(&ctx_at(10)), Some(closes[5]));
        assert_eq!(lagging_ref.prev_value(&ctx_at(5)), None);
        assert_eq!(lagging_ref.prev_value(&ctx_at(10)), Some(closes[4]));
    }

    #[test]
    fn test_lagging_span_condition_signals_in_final_lagging_bars() {
        // Crossing lands in the last `lagging` bars, which the old
        // plot-aligned array always forced to None.
        let lagging = 5;
        let n = 40;
        let mut closes = vec![100.0; n];
        for (offset, close) in closes.iter_mut().enumerate().skip(36) {
            *close = 100.0 + (offset - 35) as f64 * 5.0;
        }
        let candles = flat_candles(&closes);

        let ich = ichimoku_custom(9, 26, lagging, 26);
        let strategy = StrategyBuilder::new("Chikou Cross")
            .entry(price().above_ref(ich.lagging_span()))
            .exit(price().below_ref(ich.lagging_span()))
            .warmup(lagging + 1)
            .build();

        let result = BacktestEngine::new(BacktestConfig::zero_cost())
            .run("TEST", &candles, strategy)
            .unwrap();

        let last_lagging_bars_start = (n - lagging) as i64 * 86_400;
        assert!(
            result
                .trades
                .iter()
                .any(|t| t.entry_timestamp >= last_lagging_bars_start)
        );
    }
}

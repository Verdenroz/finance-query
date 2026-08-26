use crate::backtesting::config::{PositionSizing, SizingContext};
use crate::backtesting::position::Trade;
use crate::indicators;
use crate::models::chart::Candle;

use super::BacktestEngine;

/// Series the active sizing scheme reads, computed once per run. Both fields
/// stay `None` for schemes that need no bar history.
#[derive(Default)]
pub(super) struct SizingSeries {
    atr: Option<Vec<Option<f64>>>,
    vol: Option<Vec<Option<f64>>>,
}

/// Sample standard deviation of simple returns over a trailing window.
///
/// Entries before `lookback` returns exist are `None`, as is any window
/// touching a non-positive close.
fn rolling_volatility(closes: &[f64], lookback: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if lookback < 2 || n < 2 {
        return out;
    }

    let mut returns = vec![f64::NAN; n];
    for i in 1..n {
        if closes[i - 1] > 0.0 {
            returns[i] = closes[i] / closes[i - 1] - 1.0;
        }
    }

    for i in lookback..n {
        let window = &returns[i - lookback + 1..=i];
        if window.iter().any(|r| r.is_nan()) {
            continue;
        }
        let mean = window.iter().sum::<f64>() / lookback as f64;
        let variance =
            window.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (lookback as f64 - 1.0);
        out[i] = Some(variance.sqrt());
    }

    out
}

/// Win rate and payoff ratio over the trailing fully-closed trades.
///
/// Partial closes are skipped so one entry that scales out several times
/// contributes one observation rather than several. Returns `(None, None)`
/// unless the window holds at least one win and one loss.
///
/// Break-even trades sit outside the win rate, unlike
/// [`PerformanceMetrics::win_rate`], so both Kelly inputs rest on the same set
/// of trades.
///
/// [`PerformanceMetrics::win_rate`]: crate::backtesting::PerformanceMetrics::win_rate
fn trailing_kelly_inputs(trades: &[Trade], lookback: usize) -> (Option<f64>, Option<f64>) {
    let closed: Vec<&Trade> = trades.iter().filter(|t| !t.is_partial).collect();
    let start = closed.len().saturating_sub(lookback.max(1));
    let window = &closed[start..];
    if window.len() < 2 {
        return (None, None);
    }

    let wins: Vec<f64> = window
        .iter()
        .filter(|t| t.pnl > 0.0)
        .map(|t| t.return_pct)
        .collect();
    let losses: Vec<f64> = window
        .iter()
        .filter(|t| t.pnl < 0.0)
        .map(|t| t.return_pct.abs())
        .collect();
    if wins.is_empty() || losses.is_empty() {
        return (None, None);
    }

    let win_rate = wins.len() as f64 / (wins.len() + losses.len()) as f64;
    let avg_win = wins.iter().sum::<f64>() / wins.len() as f64;
    let avg_loss = losses.iter().sum::<f64>() / losses.len() as f64;
    if avg_loss <= 0.0 {
        return (None, None);
    }

    (Some(win_rate), Some(avg_win / avg_loss))
}

impl BacktestEngine {
    pub(super) fn compute_sizing_series(&self, candles: &[Candle]) -> SizingSeries {
        match self.config.position_sizing {
            PositionSizing::Atr { atr_period, .. } => {
                let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
                let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();
                let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
                SizingSeries {
                    atr: indicators::atr(&highs, &lows, &closes, atr_period).ok(),
                    vol: None,
                }
            }
            PositionSizing::VolatilityTarget { lookback, .. } => {
                let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
                SizingSeries {
                    atr: None,
                    vol: Some(rolling_volatility(&closes, lookback)),
                }
            }
            PositionSizing::FixedFraction | PositionSizing::FractionalKelly { .. } => {
                SizingSeries::default()
            }
        }
    }

    pub(super) fn build_sizing_context(
        &self,
        index: usize,
        series: &SizingSeries,
        trades: &[Trade],
    ) -> SizingContext {
        let (win_rate, payoff_ratio) = match self.config.position_sizing {
            PositionSizing::FractionalKelly {
                lookback_trades, ..
            } => trailing_kelly_inputs(trades, lookback_trades),
            _ => (None, None),
        };

        SizingContext {
            atr: series
                .atr
                .as_ref()
                .and_then(|s| s.get(index).copied().flatten()),
            recent_volatility: series
                .vol
                .as_ref()
                .and_then(|s| s.get(index).copied().flatten()),
            win_rate,
            payoff_ratio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::config::BacktestConfig;
    use crate::backtesting::engine::fixtures::{
        EnterLongAt, make_alternating_candles, make_candles_with_range,
    };
    use crate::backtesting::position::{PositionSide, Trade};
    use crate::backtesting::signal::Signal;

    fn make_trade(pnl: f64, return_pct: f64, is_partial: bool) -> Trade {
        Trade {
            side: PositionSide::Long,
            entry_timestamp: 0,
            exit_timestamp: 1,
            entry_price: 100.0,
            exit_price: 100.0 + pnl,
            quantity: 1.0,
            entry_quantity: 1.0,
            commission: 0.0,
            transaction_tax: 0.0,
            pnl,
            return_pct,
            dividend_income: 0.0,
            unreinvested_dividends: 0.0,
            financing_cost: 0.0,
            entry_signal: Signal::long(0, 100.0),
            exit_signal: Signal::exit(1, 100.0),
            tags: vec![],
            is_partial,
            scale_sequence: 0,
        }
    }

    fn run_with(sizing: PositionSizing, candles: &[Candle], entry_at: usize) -> f64 {
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .position_sizing(sizing)
            .build()
            .unwrap();
        let result = BacktestEngine::new(config)
            .run("TEST", candles, EnterLongAt(entry_at))
            .unwrap();
        result.trades[0].entry_quantity
    }

    #[test]
    fn test_rolling_volatility_is_none_before_lookback() {
        let closes: Vec<f64> = (0..10).map(|i| 100.0 + i as f64).collect();
        let vol = rolling_volatility(&closes, 4);
        assert!(vol[..4].iter().all(|v| v.is_none()));
        assert!(vol[4].is_some());
    }

    #[test]
    fn test_rolling_volatility_grows_with_swing_size() {
        let calm: Vec<f64> = (0..40u32)
            .map(|i| if i.is_multiple_of(2) { 100.0 } else { 100.1 })
            .collect();
        let wild: Vec<f64> = (0..40u32)
            .map(|i| if i.is_multiple_of(2) { 100.0 } else { 110.0 })
            .collect();
        let calm_vol = rolling_volatility(&calm, 20)[39].unwrap();
        let wild_vol = rolling_volatility(&wild, 20)[39].unwrap();
        assert!(wild_vol > calm_vol);
    }

    #[test]
    fn test_rolling_volatility_skips_windows_with_nonpositive_close() {
        let closes = vec![100.0, 0.0, 101.0, 102.0, 103.0, 104.0];
        let vol = rolling_volatility(&closes, 3);
        assert!(vol[3].is_none());
        assert!(vol[5].is_some());
    }

    #[test]
    fn test_trailing_kelly_inputs_from_wins_and_losses() {
        let trades = vec![
            make_trade(10.0, 0.10, false),
            make_trade(10.0, 0.10, false),
            make_trade(10.0, 0.10, false),
            make_trade(-5.0, -0.05, false),
        ];
        let (win_rate, payoff) = trailing_kelly_inputs(&trades, 10);
        assert!((win_rate.unwrap() - 0.75).abs() < 1e-12);
        assert!((payoff.unwrap() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn test_trailing_kelly_inputs_exclude_partial_closes() {
        let trades = vec![
            make_trade(10.0, 0.10, false),
            make_trade(10.0, 0.10, true),
            make_trade(10.0, 0.10, true),
            make_trade(10.0, 0.10, true),
            make_trade(-5.0, -0.05, false),
        ];
        let (win_rate, _) = trailing_kelly_inputs(&trades, 10);
        assert!((win_rate.unwrap() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_trailing_kelly_inputs_ignore_break_even_trades() {
        let trades = vec![
            make_trade(10.0, 0.10, false),
            make_trade(0.0, 0.0, false),
            make_trade(0.0, 0.0, false),
            make_trade(-5.0, -0.05, false),
        ];
        let (win_rate, payoff) = trailing_kelly_inputs(&trades, 10);
        assert!((win_rate.unwrap() - 0.5).abs() < 1e-12);
        assert!((payoff.unwrap() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn test_trailing_kelly_inputs_need_both_outcomes() {
        let all_wins = vec![make_trade(10.0, 0.10, false), make_trade(8.0, 0.08, false)];
        assert_eq!(trailing_kelly_inputs(&all_wins, 10), (None, None));

        let all_losses = vec![
            make_trade(-10.0, -0.10, false),
            make_trade(-8.0, -0.08, false),
        ];
        assert_eq!(trailing_kelly_inputs(&all_losses, 10), (None, None));

        let single = vec![make_trade(10.0, 0.10, false)];
        assert_eq!(trailing_kelly_inputs(&single, 10), (None, None));
    }

    #[test]
    fn test_sizing_warmup_covers_the_first_usable_series_value() {
        let candles = make_candles_with_range(&[100.0; 60], 0.02);
        let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
        let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

        let atr_period = 14;
        let atr = indicators::atr(&highs, &lows, &closes, atr_period).unwrap();
        assert!(atr[atr_period].is_some());

        let lookback = 20;
        let vol = rolling_volatility(&closes, lookback);
        assert!(vol[lookback].is_some());
    }

    #[test]
    fn test_atr_sizing_shrinks_position_in_wider_ranges() {
        let sizing = PositionSizing::Atr {
            risk_pct: 0.02,
            atr_period: 14,
            atr_multiple: 2.0,
        };
        let narrow = make_candles_with_range(&[100.0; 60], 0.002);
        let wide = make_candles_with_range(&[100.0; 60], 0.05);
        assert!(run_with(sizing, &wide, 20) < run_with(sizing, &narrow, 20));
    }

    #[test]
    fn test_volatility_target_sizing_shrinks_in_higher_volatility() {
        let sizing = PositionSizing::VolatilityTarget {
            target_vol_pct: 0.005,
            lookback: 20,
        };
        let calm = make_alternating_candles(100.0, 0.001, 60);
        let wild = make_alternating_candles(100.0, 0.10, 60);
        assert!(run_with(sizing, &wild, 30) < run_with(sizing, &calm, 30));
    }

    #[test]
    fn test_explicit_fixed_fraction_matches_an_untouched_config() {
        let candles = make_candles_with_range(&[100.0; 60], 0.02);
        let baseline = BacktestEngine::new(BacktestConfig::default())
            .run("TEST", &candles, EnterLongAt(0))
            .unwrap();
        let explicit = BacktestEngine::new(
            BacktestConfig::builder()
                .position_sizing(PositionSizing::FixedFraction)
                .build()
                .unwrap(),
        )
        .run("TEST", &candles, EnterLongAt(0))
        .unwrap();

        assert_eq!(baseline.trades.len(), explicit.trades.len());
        assert!((baseline.final_equity - explicit.final_equity).abs() < 1e-12);
        assert!(
            (baseline.metrics.total_return_pct - explicit.metrics.total_return_pct).abs() < 1e-12
        );
    }

    #[test]
    fn test_sizing_warmup_defers_the_first_entry() {
        let config = BacktestConfig::builder()
            .position_sizing(PositionSizing::VolatilityTarget {
                target_vol_pct: 0.01,
                lookback: 20,
            })
            .build()
            .unwrap();
        let candles = make_alternating_candles(100.0, 0.01, 10);
        let err = BacktestEngine::new(config).run("TEST", &candles, EnterLongAt(0));
        assert!(err.is_err());
    }
}

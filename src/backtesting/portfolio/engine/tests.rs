use super::*;
use crate::backtesting::portfolio::config::{PortfolioConfig, RebalanceMode};
use crate::backtesting::signal::SignalDirection;
use crate::backtesting::strategy::{Strategy, StrategyContext};
use crate::backtesting::{BacktestConfig, SmaCrossover};
use crate::indicators::Indicator;

#[derive(Clone)]
struct EnterShortHold;

impl Strategy for EnterShortHold {
    fn name(&self) -> &str {
        "Enter Short Hold"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 && !ctx.has_position() {
            Signal::short(ctx.timestamp(), ctx.close())
        } else {
            Signal::hold()
        }
    }
}

#[derive(Clone)]
struct TimedLongStrategy {
    entry_idx: usize,
    exit_idx: usize,
}

impl Strategy for TimedLongStrategy {
    fn name(&self) -> &str {
        "Timed Long"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == self.entry_idx && !ctx.has_position() {
            Signal::long(ctx.timestamp(), ctx.close())
        } else if ctx.index == self.exit_idx && ctx.has_position() {
            Signal::exit(ctx.timestamp(), ctx.close())
        } else {
            Signal::hold()
        }
    }
}

#[test]
fn test_sym_max_leverage_stays_unlevered_when_an_entry_draws_on_grown_cash() {
    let winner = make_candles(&[100.0, 100.0, 200.0, 200.0, 200.0, 200.0, 200.0, 200.0]);
    let flat = make_candles(&[100.0; 8]);
    let data = vec![
        SymbolData::new("WIN", winner),
        SymbolData::new("FLAT", flat),
    ];

    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap(),
    )
    .rebalance(RebalanceMode::AvailableCapital);

    let result = PortfolioEngine::new(config)
        .run(&data, |sym| {
            if sym == "WIN" {
                TimedLongStrategy {
                    entry_idx: 0,
                    exit_idx: 3,
                }
            } else {
                TimedLongStrategy {
                    entry_idx: 4,
                    exit_idx: 7,
                }
            }
        })
        .unwrap();

    assert!(result.final_equity > result.initial_capital);
    for sym in result.symbols.values() {
        assert!(
            sym.max_leverage_used <= 1.0 + 1e-9,
            "{} reported {:.3}x in an unlevered portfolio",
            sym.symbol,
            sym.max_leverage_used,
        );
    }
}

#[derive(Clone)]
pub(super) struct FirstBarDirectionalEntry {
    pub(super) direction: SignalDirection,
}

impl Strategy for FirstBarDirectionalEntry {
    fn name(&self) -> &str {
        "First Bar Directional Entry"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 && !ctx.has_position() {
            match self.direction {
                SignalDirection::Short => Signal::short(ctx.timestamp(), ctx.close()),
                _ => Signal::long(ctx.timestamp(), ctx.close()),
            }
        } else {
            Signal::hold()
        }
    }
}

#[test]
fn test_short_proceeds_do_not_fund_a_same_bar_long_beyond_equity() {
    let prices = vec![100.0, 100.0, 100.0];
    let data = vec![
        SymbolData::new("A", make_candles(&prices)),
        SymbolData::new("B", make_candles(&prices)),
    ];

    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(10_000.0)
            .allow_short(true)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap(),
    );

    let result = PortfolioEngine::new(config)
        .run(&data, |sym| FirstBarDirectionalEntry {
            direction: if sym == "A" {
                SignalDirection::Short
            } else {
                SignalDirection::Long
            },
        })
        .unwrap();

    for sym in result.symbols.values() {
        assert!(
            sym.max_leverage_used <= 1.0 + 1e-9,
            "{} reported {:.3}x in an unlevered portfolio",
            sym.symbol,
            sym.max_leverage_used,
        );
    }
    assert!(
        result.symbols["B"].trades.is_empty(),
        "A's full-size short leaves no buying power for B's long"
    );
}

#[test]
fn test_trailing_stop_does_not_arm_and_fire_on_the_same_bar() {
    let candles = vec![
        Candle {
            timestamp: 0,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: 1_000,
            adj_close: Some(100.0),
            provider_id: None,
        },
        Candle {
            timestamp: 86_400,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: 1_000,
            adj_close: Some(100.0),
            provider_id: None,
        },
        Candle {
            timestamp: 172_800,
            open: 100.0,
            high: 110.0,
            low: 99.0,
            close: 109.0,
            volume: 1_000,
            adj_close: Some(109.0),
            provider_id: None,
        },
    ];
    let data = vec![SymbolData::new("A", candles)];

    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .trailing_stop_pct(0.05)
            .close_at_end(false)
            .build()
            .unwrap(),
    );

    let result = PortfolioEngine::new(config)
        .run(&data, |_| FirstBarLongElseHold { enabled: true })
        .unwrap();

    assert!(
        result.symbols["A"].trades.is_empty(),
        "trailing stop should not fire on the bar that set its own high"
    );
}

pub(super) fn make_candles(prices: &[f64]) -> Vec<Candle> {
    prices
        .iter()
        .enumerate()
        .map(|(i, &p)| Candle {
            timestamp: i as i64 * 86400,
            open: p,
            high: p * 1.005,
            low: p * 0.995,
            close: p,
            volume: 1_000,
            adj_close: Some(p),
            provider_id: None,
        })
        .collect()
}

fn make_candles_with_timestamps(prices: &[f64], timestamps: &[i64]) -> Vec<Candle> {
    prices
        .iter()
        .zip(timestamps.iter())
        .map(|(&p, &ts)| Candle {
            timestamp: ts,
            open: p,
            high: p * 1.005,
            low: p * 0.995,
            close: p,
            volume: 1_000,
            adj_close: Some(p),
            provider_id: None,
        })
        .collect()
}

#[derive(Clone)]
pub(super) struct FirstBarLongElseHold {
    pub(super) enabled: bool,
}

impl Strategy for FirstBarLongElseHold {
    fn name(&self) -> &str {
        "First Bar Long"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if self.enabled && ctx.index == 0 && !ctx.has_position() {
            Signal::long(ctx.timestamp(), ctx.close())
        } else {
            Signal::hold()
        }
    }
}

fn trending_prices(n: usize, start: f64, rate: f64) -> Vec<f64> {
    (0..n).map(|i| start + i as f64 * rate).collect()
}

#[test]
fn test_two_symbol_basic() {
    let prices_a = trending_prices(100, 100.0, 0.5);
    let prices_b = trending_prices(100, 50.0, 0.25);

    let symbol_data = vec![
        SymbolData::new("AAPL", make_candles(&prices_a)),
        SymbolData::new("MSFT", make_candles(&prices_b)),
    ];

    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(20_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap(),
    )
    .max_total_positions(2);

    let engine = PortfolioEngine::new(config);
    let result = engine
        .run(&symbol_data, |_| SmaCrossover::new(5, 20))
        .unwrap();

    assert!(result.symbols.contains_key("AAPL"));
    assert!(result.symbols.contains_key("MSFT"));
    assert!(result.final_equity > 0.0);
    assert!(!result.portfolio_equity_curve.is_empty());
}

#[test]
fn test_max_total_positions_respected() {
    // Two strongly trending symbols; with max_positions=1 only one should trade
    let prices = trending_prices(100, 100.0, 1.0);
    let symbol_data = vec![
        SymbolData::new("A", make_candles(&prices)),
        SymbolData::new("B", make_candles(&prices)),
    ];

    let config = PortfolioConfig::new(BacktestConfig::default()).max_total_positions(1);

    let engine = PortfolioEngine::new(config);
    let result = engine
        .run(&symbol_data, |_| SmaCrossover::new(5, 20))
        .unwrap();

    // At any time only one symbol should be open — total concurrent positions ≤ 1
    for snapshot in &result.allocation_history {
        assert!(
            snapshot.positions.len() <= 1,
            "more than 1 position open at timestamp {}",
            snapshot.timestamp
        );
    }
}

#[test]
fn test_equal_weight_allocation() {
    let prices = trending_prices(100, 100.0, 0.5);
    let symbol_data = vec![
        SymbolData::new("X", make_candles(&prices)),
        SymbolData::new("Y", make_candles(&prices)),
    ];

    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap(),
    )
    .rebalance(RebalanceMode::EqualWeight)
    .max_total_positions(2);

    let engine = PortfolioEngine::new(config);
    let result = engine
        .run(&symbol_data, |_| SmaCrossover::new(5, 20))
        .unwrap();

    // Portfolio should have run without error
    assert!(result.final_equity > 0.0);
}

#[test]
fn test_dividend_credited() {
    let prices = trending_prices(50, 100.0, 0.2);
    let dividends = vec![
        Dividend {
            timestamp: 20 * 86400,
            amount: 1.0,
            provider_id: None,
        },
        Dividend {
            timestamp: 40 * 86400,
            amount: 1.0,
            provider_id: None,
        },
    ];
    let symbol_data = vec![SymbolData::new("DIV", make_candles(&prices)).with_dividends(dividends)];

    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap(),
    );

    let engine = PortfolioEngine::new(config);
    let result = engine
        .run(&symbol_data, |_| SmaCrossover::new(5, 20))
        .unwrap();

    // Total dividend income across all trades should be non-negative
    let total_div: f64 = result.symbols["DIV"]
        .trades
        .iter()
        .map(|t| t.dividend_income)
        .sum();
    assert!(total_div >= 0.0);
}

#[test]
fn test_empty_symbol_data_fails() {
    let config = PortfolioConfig::default();
    let engine = PortfolioEngine::new(config);
    assert!(
        engine
            .run::<SmaCrossover, _>(&[], |_| SmaCrossover::new(5, 20))
            .is_err()
    );
}

#[test]
fn test_short_dividend_is_liability() {
    let prices = vec![100.0, 100.0, 100.0];
    let candles = make_candles(&prices);
    let dividends = vec![Dividend {
        timestamp: candles[1].timestamp,
        amount: 1.0,
        provider_id: None,
    }];
    let symbol_data = vec![SymbolData::new("DIVS", candles).with_dividends(dividends)];

    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(10_000.0)
            .allow_short(true)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap(),
    );

    let engine = PortfolioEngine::new(config);
    let result = engine.run(&symbol_data, |_| EnterShortHold).unwrap();

    let trades = &result.symbols["DIVS"].trades;
    assert_eq!(trades.len(), 1);
    assert!(trades[0].dividend_income < 0.0);
    assert!(result.final_equity < 10_000.0);
}

#[test]
fn test_portfolio_time_in_market_uses_union_exposure() {
    let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0];

    let symbol_data = vec![
        SymbolData::new("A", make_candles(&prices)),
        SymbolData::new("B", make_candles(&prices)),
    ];

    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(10_000.0)
            .position_size_pct(0.5)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .close_at_end(false)
            .build()
            .unwrap(),
    )
    .max_total_positions(2);

    let engine = PortfolioEngine::new(config);
    let result = engine
        .run(&symbol_data, |sym| {
            if sym == "A" {
                TimedLongStrategy {
                    entry_idx: 0,
                    exit_idx: 2,
                }
            } else {
                TimedLongStrategy {
                    entry_idx: 1,
                    exit_idx: 3,
                }
            }
        })
        .unwrap();

    // Union exposure spans [t0, t3] over total [t0, t4] => 3/4 = 0.75.
    // A per-trade sum approach would overstate this to 1.0 (clipped).
    let actual = result.portfolio_metrics.time_in_market_pct;
    assert!(
        (actual - 0.75).abs() < 1e-9,
        "expected 0.75 union exposure, got {actual}"
    );
}

#[test]
fn test_portfolio_marks_open_positions_on_sparse_timestamps() {
    let symbol_data = vec![
        SymbolData::new("A", make_candles_with_timestamps(&[100.0, 110.0], &[0, 2])),
        SymbolData::new(
            "B",
            make_candles_with_timestamps(&[50.0, 50.0, 50.0], &[0, 1, 2]),
        ),
    ];

    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(10_000.0)
            .position_size_pct(1.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .close_at_end(false)
            .build()
            .unwrap(),
    )
    .max_total_positions(2);

    let engine = PortfolioEngine::new(config);
    let result = engine
        .run(&symbol_data, |sym| FirstBarLongElseHold {
            enabled: sym == "A",
        })
        .unwrap();

    let snapshot_t1 = result
        .allocation_history
        .iter()
        .find(|s| s.timestamp == 1)
        .expect("snapshot at timestamp 1");
    assert!(
        snapshot_t1.positions.contains_key("A"),
        "open A position should be valued at t=1"
    );
    // Entry fills at next-bar open (110), valued at close_at_or_before(t=1)=100.
    // Equity ≈ 9091 (not ~10000): the key property is the position is included,
    // not that there's no slippage from fill-bar price differences.
    assert!(
        snapshot_t1.total_equity() > 8_000.0,
        "equity should include carried-forward A valuation, got {}",
        snapshot_t1.total_equity()
    );
}

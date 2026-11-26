//! Margin, financing, and sizing parity tests.

use crate::backtesting::signal::{Signal, SignalDirection};
use crate::backtesting::strategy::{Strategy, StrategyContext};
use crate::backtesting::{BacktestConfig, PositionSizing};
use crate::indicators::Indicator;

use super::super::config::PortfolioConfig;
use super::tests::{FirstBarDirectionalEntry, FirstBarLongElseHold, make_candles};
use super::{PortfolioEngine, SymbolData};

#[derive(Clone)]
struct ShortThenScaleIn;

impl Strategy for ShortThenScaleIn {
    fn name(&self) -> &str {
        "Short Then Scale In"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 {
            Signal::short(ctx.timestamp(), ctx.close())
        } else if ctx.has_position() {
            Signal::scale_in(1.0, ctx.timestamp(), ctx.close())
        } else {
            Signal::hold()
        }
    }
}

#[test]
fn test_short_scale_in_cannot_exceed_portfolio_equity() {
    let prices = vec![100.0; 10];
    let data = vec![SymbolData::new("A", make_candles(&prices))];

    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(10_000.0)
            .allow_short(true)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .close_at_end(false)
            .build()
            .unwrap(),
    );

    let result = PortfolioEngine::new(config)
        .run(&data, |_| ShortThenScaleIn)
        .unwrap();

    assert!(
        result.symbols["A"].max_leverage_used <= 1.0 + 1e-9,
        "repeated short scale-ins reached {:.3}x in an unlevered portfolio",
        result.symbols["A"].max_leverage_used,
    );
}

#[derive(Clone)]
struct LongAtBar {
    bar: usize,
}

impl Strategy for LongAtBar {
    fn name(&self) -> &str {
        "Long At Bar"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == self.bar && !ctx.has_position() {
            Signal::long(ctx.timestamp(), ctx.close())
        } else {
            Signal::hold()
        }
    }
}

#[derive(Clone)]
struct FirstBarLongWithBracketStop;

impl Strategy for FirstBarLongWithBracketStop {
    fn name(&self) -> &str {
        "First Bar Long With Bracket Stop"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 && !ctx.has_position() {
            Signal::long(ctx.timestamp(), ctx.close()).stop_loss(0.05)
        } else {
            Signal::hold()
        }
    }
}

#[test]
fn test_per_trade_bracket_stop_fires_in_portfolio() {
    let data = vec![SymbolData::new(
        "A",
        make_candles(&[100.0, 100.0, 100.0, 95.2]),
    )];
    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .close_at_end(false)
            .build()
            .unwrap(),
    );

    let result = PortfolioEngine::new(config)
        .run(&data, |_| FirstBarLongWithBracketStop)
        .unwrap();

    let sym = &result.symbols["A"];
    assert_eq!(
        sym.trades.len(),
        1,
        "the signal-level stop_loss bracket must produce a stop exit"
    );
    assert!((sym.trades[0].exit_price - 95.0).abs() < 1e-9);
}

#[test]
fn test_levered_entry_doubles_notional() {
    let data = vec![SymbolData::new("A", make_candles(&[100.0; 5]))];
    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(10_000.0)
            .max_leverage(2.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap(),
    );

    let result = PortfolioEngine::new(config)
        .run(&data, |_| FirstBarLongElseHold { enabled: true })
        .unwrap();

    let sym = &result.symbols["A"];
    assert_eq!(sym.trades.len(), 1);
    assert!(
        (sym.trades[0].entry_quantity - 200.0).abs() < 1.0,
        "expected a 2x notional fill, got {} shares",
        sym.trades[0].entry_quantity
    );
    assert!(
        (sym.max_leverage_used - 2.0).abs() < 0.05,
        "got {:.3}x",
        sym.max_leverage_used
    );
    assert!((result.final_equity - 10_000.0).abs() < 1e-6);
}

#[test]
fn test_portfolio_margin_call_liquidates_largest_first() {
    let data = vec![
        SymbolData::new("A", make_candles(&[100.0, 100.0, 40.0])),
        SymbolData::new("B", make_candles(&[100.0, 100.0, 40.0])),
    ];
    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(10_000.0)
            .max_leverage(2.0)
            .position_size_pct(0.5)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .close_at_end(false)
            .build()
            .unwrap(),
    );

    let result = PortfolioEngine::new(config)
        .run(&data, |_| FirstBarLongElseHold { enabled: true })
        .unwrap();

    let a = &result.symbols["A"];
    let b = &result.symbols["B"];
    assert_eq!(a.trades.len(), 1, "largest position should be liquidated");
    assert!(
        a.trades[0]
            .exit_signal
            .reason
            .as_deref()
            .unwrap_or_default()
            .contains("Margin call"),
    );
    assert!(a.open_position.is_none());
    assert!(
        b.open_position.is_some() && b.trades.is_empty(),
        "liquidation should stop once the requirement is met"
    );
}

#[test]
fn test_short_borrow_accrues_financing_in_portfolio() {
    let data = vec![SymbolData::new("A", make_candles(&[100.0; 5]))];
    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(10_000.0)
            .allow_short(true)
            .short_borrow_rate(0.252)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap(),
    );

    let result = PortfolioEngine::new(config)
        .run(&data, |_| FirstBarDirectionalEntry {
            direction: SignalDirection::Short,
        })
        .unwrap();

    // 100 shares shorted at 100: 0.252 / 252 = 0.1% per bar on 10,000
    // notional, accrued on bars 1..=4.
    let trade = &result.symbols["A"].trades[0];
    assert!(
        (trade.financing_cost - 40.0).abs() < 1e-6,
        "got {}",
        trade.financing_cost
    );
    let total_pnl: f64 = result
        .symbols
        .values()
        .flat_map(|r| r.trades.iter())
        .map(|t| t.pnl)
        .sum();
    assert!(
        (result.final_equity - (10_000.0 + total_pnl)).abs() < 1e-6,
        "final {} vs initial + pnl {}",
        result.final_equity,
        10_000.0 + total_pnl
    );
}

#[test]
fn test_volatility_sizing_shrinks_portfolio_entry() {
    let wild: Vec<f64> = (0..12)
        .map(|i| if i % 2 == 0 { 100.0 } else { 106.0 })
        .collect();
    let run = |sizing: PositionSizing| {
        let data = vec![SymbolData::new("A", make_candles(&wild))];
        let config = PortfolioConfig::new(
            BacktestConfig::builder()
                .initial_capital(10_000.0)
                .commission_pct(0.0)
                .slippage_pct(0.0)
                .position_sizing(sizing)
                .build()
                .unwrap(),
        );
        PortfolioEngine::new(config)
            .run(&data, |_| LongAtBar { bar: 8 })
            .unwrap()
            .symbols["A"]
            .trades[0]
            .entry_quantity
    };

    let fixed = run(PositionSizing::FixedFraction);
    let vol_sized = run(PositionSizing::VolatilityTarget {
        target_vol_pct: 0.005,
        lookback: 5,
    });
    assert!(
        vol_sized < fixed * 0.5,
        "vol-sized {vol_sized} vs fixed {fixed}"
    );
}

use crate::backtesting::signal::Signal;
use crate::backtesting::strategy::{Strategy, StrategyContext};
use crate::indicators::Indicator;
use crate::models::chart::Candle;

#[derive(Clone)]
pub(super) struct EnterLongHold;

impl Strategy for EnterLongHold {
    fn name(&self) -> &str {
        "Enter Long Hold"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 && !ctx.has_position() {
            Signal::long(ctx.timestamp(), ctx.close())
        } else {
            Signal::hold()
        }
    }
}

#[derive(Clone)]
pub(super) struct EnterShortHold;

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
pub(super) struct EnterLongAt(pub usize);

impl Strategy for EnterLongAt {
    fn name(&self) -> &str {
        "Enter Long At"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == self.0 && !ctx.has_position() {
            Signal::long(ctx.timestamp(), ctx.close())
        } else {
            Signal::hold()
        }
    }
}

pub(super) fn make_candles(prices: &[f64]) -> Vec<Candle> {
    prices
        .iter()
        .enumerate()
        .map(|(i, &p)| Candle {
            timestamp: i as i64,
            open: p,
            high: p * 1.01,
            low: p * 0.99,
            close: p,
            volume: 1000,
            adj_close: Some(p),
            provider_id: None,
        })
        .collect()
}

pub(super) fn make_candles_with_timestamps(prices: &[f64], timestamps: &[i64]) -> Vec<Candle> {
    prices
        .iter()
        .zip(timestamps.iter())
        .map(|(&p, &ts)| Candle {
            timestamp: ts,
            open: p,
            high: p * 1.01,
            low: p * 0.99,
            close: p,
            volume: 1000,
            adj_close: Some(p),
            provider_id: None,
        })
        .collect()
}

/// Build a candle with explicit OHLC values (not derived from a single price).
pub(super) fn make_candle_ohlc(ts: i64, open: f64, high: f64, low: f64, close: f64) -> Candle {
    Candle {
        timestamp: ts,
        open,
        high,
        low,
        close,
        volume: 1000,
        adj_close: Some(close),
        provider_id: None,
    }
}

/// Flat-close candles whose intrabar range is `range_pct` of price, giving a
/// controllable ATR without moving the close series.
pub(super) fn make_candles_with_range(prices: &[f64], range_pct: f64) -> Vec<Candle> {
    prices
        .iter()
        .enumerate()
        .map(|(i, &p)| Candle {
            timestamp: i as i64,
            open: p,
            high: p * (1.0 + range_pct),
            low: p * (1.0 - range_pct),
            close: p,
            volume: 1000,
            adj_close: Some(p),
            provider_id: None,
        })
        .collect()
}

/// Candles whose closes alternate between `base` and `base * (1 + swing_pct)`,
/// giving a controllable close-to-close return volatility.
pub(super) fn make_alternating_candles(base: f64, swing_pct: f64, n: usize) -> Vec<Candle> {
    let prices: Vec<f64> = (0..n)
        .map(|i| {
            if i.is_multiple_of(2) {
                base
            } else {
                base * (1.0 + swing_pct)
            }
        })
        .collect();
    make_candles(&prices)
}

/// A strategy that opens a long on the first bar and holds forever.
pub(super) struct EnterLongBar0;
impl Strategy for EnterLongBar0 {
    fn name(&self) -> &str {
        "Enter Long Bar 0"
    }
    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }
    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 && !ctx.has_position() {
            Signal::long(ctx.timestamp(), ctx.close())
        } else {
            Signal::hold()
        }
    }
}

/// Strategy: enter long on bar 0, scale in on bar 1, exit on bar 2.
#[derive(Clone)]
pub(super) struct EnterScaleInExit;

impl Strategy for EnterScaleInExit {
    fn name(&self) -> &str {
        "EnterScaleInExit"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        match ctx.index {
            0 => Signal::long(ctx.timestamp(), ctx.close()),
            1 if ctx.has_position() => Signal::scale_in(0.5, ctx.timestamp(), ctx.close()),
            2 if ctx.has_position() => Signal::exit(ctx.timestamp(), ctx.close()),
            _ => Signal::hold(),
        }
    }
}

/// Strategy: enter short on bar 0, scale in on bar 1, hold.
#[derive(Clone)]
pub(super) struct EnterShortScaleIn;

impl Strategy for EnterShortScaleIn {
    fn name(&self) -> &str {
        "EnterShortScaleIn"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        match ctx.index {
            0 => Signal::short(ctx.timestamp(), ctx.close()),
            1 if ctx.has_position() => Signal::scale_in(0.5, ctx.timestamp(), ctx.close()),
            _ => Signal::hold(),
        }
    }
}

/// Strategy: enter long on bar 0, scale out 50% on bar 1, exit remainder on bar 2.
#[derive(Clone)]
pub(super) struct EnterScaleOutExit;

impl Strategy for EnterScaleOutExit {
    fn name(&self) -> &str {
        "EnterScaleOutExit"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        match ctx.index {
            0 => Signal::long(ctx.timestamp(), ctx.close()),
            1 if ctx.has_position() => Signal::scale_out(0.5, ctx.timestamp(), ctx.close()),
            2 if ctx.has_position() => Signal::exit(ctx.timestamp(), ctx.close()),
            _ => Signal::hold(),
        }
    }
}

/// Enters a long position on bar 0 with a per-trade stop-loss.
#[derive(Clone)]
pub(super) struct BracketLongStopLossStrategy {
    pub(super) stop_pct: f64,
}
impl Strategy for BracketLongStopLossStrategy {
    fn name(&self) -> &str {
        "BracketLongStopLoss"
    }
    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }
    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 && !ctx.has_position() {
            Signal::long(ctx.timestamp(), ctx.close()).stop_loss(self.stop_pct)
        } else {
            Signal::hold()
        }
    }
}

/// Enters a short position on bar 0 with a per-trade stop-loss.
#[derive(Clone)]
pub(super) struct BracketShortStopLossStrategy {
    pub(super) stop_pct: f64,
}
impl Strategy for BracketShortStopLossStrategy {
    fn name(&self) -> &str {
        "BracketShortStopLoss"
    }
    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }
    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 && !ctx.has_position() {
            Signal::short(ctx.timestamp(), ctx.close()).stop_loss(self.stop_pct)
        } else {
            Signal::hold()
        }
    }
}

/// Enters a long position on bar 0 with a per-trade take-profit.
#[derive(Clone)]
pub(super) struct BracketLongTakeProfitStrategy {
    pub(super) tp_pct: f64,
}
impl Strategy for BracketLongTakeProfitStrategy {
    fn name(&self) -> &str {
        "BracketLongTakeProfit"
    }
    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }
    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 && !ctx.has_position() {
            Signal::long(ctx.timestamp(), ctx.close()).take_profit(self.tp_pct)
        } else {
            Signal::hold()
        }
    }
}

/// Enters a short position on bar 0 with a per-trade take-profit.
#[derive(Clone)]
pub(super) struct BracketShortTakeProfitStrategy {
    pub(super) tp_pct: f64,
}
impl Strategy for BracketShortTakeProfitStrategy {
    fn name(&self) -> &str {
        "BracketShortTakeProfit"
    }
    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }
    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 && !ctx.has_position() {
            Signal::short(ctx.timestamp(), ctx.close()).take_profit(self.tp_pct)
        } else {
            Signal::hold()
        }
    }
}

/// Enters a long position on bar 0 with a per-trade trailing stop.
#[derive(Clone)]
pub(super) struct BracketLongTrailingStopStrategy {
    pub(super) trail_pct: f64,
}
impl Strategy for BracketLongTrailingStopStrategy {
    fn name(&self) -> &str {
        "BracketLongTrailingStop"
    }
    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }
    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 && !ctx.has_position() {
            Signal::long(ctx.timestamp(), ctx.close()).trailing_stop(self.trail_pct)
        } else {
            Signal::hold()
        }
    }
}

/// Enters a short position on bar 0 with a per-trade trailing stop.
#[derive(Clone)]
pub(super) struct BracketShortTrailingStopStrategy {
    pub(super) trail_pct: f64,
}
impl Strategy for BracketShortTrailingStopStrategy {
    fn name(&self) -> &str {
        "BracketShortTrailingStop"
    }
    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }
    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 && !ctx.has_position() {
            Signal::short(ctx.timestamp(), ctx.close()).trailing_stop(self.trail_pct)
        } else {
            Signal::hold()
        }
    }
}

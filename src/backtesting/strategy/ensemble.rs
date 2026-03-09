//! Ensemble strategy that combines multiple strategies with configurable voting modes.
//!
//! An ensemble aggregates signals from multiple sub-strategies and resolves them
//! using one of four voting modes.
//!
//! # Example
//!
//! ```no_run
//! use finance_query::backtesting::{EnsembleStrategy, EnsembleMode, SmaCrossover, RsiReversal, MacdSignal};
//!
//! let strategy = EnsembleStrategy::new("Multi-Signal")
//!     .add(SmaCrossover::new(10, 50), 1.0)
//!     .add(RsiReversal::default(), 0.5)
//!     .add(MacdSignal::default(), 1.0)
//!     .mode(EnsembleMode::WeightedMajority)
//!     .build();
//! ```

use crate::indicators::Indicator;

use super::{Signal, Strategy, StrategyContext};
use crate::backtesting::signal::{SignalDirection, SignalStrength};

/// Voting mode that determines how sub-strategy signals are combined.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EnsembleMode {
    /// All active sub-strategies must agree on the same direction ("no-dissent" semantics).
    ///
    /// Strategies that return `Hold` abstain from the vote and do not block a consensus.
    /// If any two *active* strategies disagree, the ensemble returns Hold.
    /// The resulting signal strength is the average of all active strengths.
    ///
    /// **Note**: if you require *all* strategies (including abstainers) to explicitly
    /// vote for the same direction, check `active_count == strategies.len()` yourself
    /// and wrap this in a custom [`Strategy`] impl.
    Unanimous,

    /// Conviction-weighted vote: each vote is `weight × signal_strength`.
    ///
    /// All five directions (`Long`, `Short`, `Exit`, `ScaleIn`, `ScaleOut`) are
    /// tallied independently; the highest score wins.
    ///
    /// **Strength denominator**: the output `signal_strength` is
    /// `winner_score / Σ(all_weights)`, not `winner_score / Σ(active_scores)`.
    /// Dividing by total potential prevents a lone weak voter from being
    /// artificially amplified when the majority of sub-strategies abstain.
    ///
    /// **Scale fraction**: when `ScaleIn` or `ScaleOut` wins, the emitted
    /// `scale_fraction` is the conviction-weighted average of all same-direction
    /// voters' fractions, not the single highest-score contributor's fraction.
    ///
    /// **Position guard**: `Exit`, `ScaleIn`, and `ScaleOut` are only tallied
    /// when a position is currently open. While flat they are discarded so they
    /// cannot suppress entry signal strength. Their weights still count toward
    /// the total-potential denominator.
    ///
    /// **Note on vote splitting**: `Exit` and `Short` are counted as independent
    /// factions — if their combined intent would dominate but each falls below
    /// `Long` individually, the ensemble may maintain a long position despite the
    /// majority wanting to exit. Document your ensemble weights accordingly.
    #[default]
    WeightedMajority,

    /// First non-Hold signal wins (strategies are evaluated in insertion order).
    ///
    /// **Note**: this gives a permanent priority advantage to strategies added
    /// first via [`add`](EnsembleStrategy::add). Use [`StrongestSignal`](Self::StrongestSignal)
    /// if you want insertion-order independence.
    AnySignal,

    /// The non-Hold signal with the highest `signal_strength` value wins.
    StrongestSignal,
}

/// A strategy that aggregates signals from multiple sub-strategies.
///
/// Build with the fluent builder methods [`add`](Self::add), [`mode`](Self::mode),
/// then finalise with [`build`](Self::build).
///
/// All six [`SignalDirection`](crate::backtesting::SignalDirection) variants are
/// fully supported. In [`EnsembleMode::WeightedMajority`], `ScaleIn` and `ScaleOut`
/// participate in the vote with the same position guard as `Exit` — they are only
/// tallied when a position is open. In `Unanimous`, `AnySignal`, and
/// `StrongestSignal` modes they are treated like any other non-Hold direction.
pub struct EnsembleStrategy {
    name: String,
    strategies: Vec<(Box<dyn Strategy>, f64)>,
    mode: EnsembleMode,
}

impl EnsembleStrategy {
    /// Create a new ensemble with the given name.
    ///
    /// The default voting mode is [`EnsembleMode::WeightedMajority`].
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            strategies: Vec::new(),
            mode: EnsembleMode::default(),
        }
    }

    /// Add a sub-strategy with the given weight.
    ///
    /// Weight is only meaningful for [`EnsembleMode::WeightedMajority`]; other
    /// modes ignore it. Negative weights are treated as zero.
    pub fn add<S: Strategy + 'static>(mut self, strategy: S, weight: f64) -> Self {
        self.strategies.push((Box::new(strategy), weight.max(0.0)));
        self
    }

    /// Set the voting mode.
    pub fn mode(mut self, mode: EnsembleMode) -> Self {
        self.mode = mode;
        self
    }

    /// Finalise the ensemble. Returns `self` (all configuration happens in the
    /// builder methods).
    pub fn build(self) -> Self {
        self
    }

    // ── voting helpers ────────────────────────────────────────────────────────

    fn any_signal(&self, ctx: &StrategyContext) -> Signal {
        for (strategy, _) in &self.strategies {
            let signal = strategy.on_candle(ctx);
            if !signal.is_hold() {
                return signal;
            }
        }
        Signal::hold()
    }

    fn unanimous(&self, ctx: &StrategyContext) -> Signal {
        // Evaluated iteratively — no allocations in the hot path.
        // As soon as any two active sub-strategies disagree we bail out early.
        let mut first_dir: Option<SignalDirection> = None;
        let mut first_signal: Option<Signal> = None;
        let mut total_strength = 0.0_f64;
        let mut active_count = 0_usize;

        for (strategy, _) in &self.strategies {
            let signal = strategy.on_candle(ctx);
            if signal.is_hold() {
                continue;
            }
            match first_dir {
                None => {
                    first_dir = Some(signal.direction);
                    total_strength = signal.strength.value();
                    first_signal = Some(signal);
                    active_count = 1;
                }
                Some(dir) if dir == signal.direction => {
                    total_strength += signal.strength.value();
                    active_count += 1;
                }
                _ => return Signal::hold(), // disagreement — short-circuit
            }
        }

        let Some(mut sig) = first_signal else {
            return Signal::hold();
        };

        let dir = first_dir.unwrap();
        let avg_strength = total_strength / active_count as f64;
        let original_reason = sig.reason.take();
        sig.strength = SignalStrength::clamped(avg_strength);
        sig.reason = Some(format!(
            "Unanimous ({} of {} agree): {}",
            active_count,
            self.strategies.len(),
            original_reason.as_deref().unwrap_or(&dir.to_string())
        ));
        sig
    }

    fn weighted_majority(&self, ctx: &StrategyContext) -> Signal {
        // Denominator = sum of ALL strategy weights (total potential conviction).
        // Prevents a lone weak voter from being artificially amplified when
        // the majority of sub-strategies abstain.
        let total_potential: f64 = self.strategies.iter().map(|(_, w)| *w).sum();
        if total_potential < f64::EPSILON {
            return Signal::hold();
        }

        let mut long_weight = 0.0_f64;
        let mut short_weight = 0.0_f64;
        let mut exit_weight = 0.0_f64;
        let mut scale_in_weight = 0.0_f64;
        let mut scale_out_weight = 0.0_f64;

        // Σ(scale_fraction × score) for conviction-weighted average fraction.
        let mut scale_in_frac_score = 0.0_f64;
        let mut scale_out_frac_score = 0.0_f64;

        // Track (signal, score) so we inherit metadata from the highest-conviction
        // contributor, not merely the first one encountered.
        let mut best_long: Option<(Signal, f64)> = None;
        let mut best_short: Option<(Signal, f64)> = None;
        let mut best_exit: Option<(Signal, f64)> = None;
        let mut best_scale_in: Option<(Signal, f64)> = None;
        let mut best_scale_out: Option<(Signal, f64)> = None;

        let has_position = ctx.has_position();

        for (strategy, weight) in &self.strategies {
            let signal = strategy.on_candle(ctx);
            // Vote score = static weight × dynamic conviction
            let score = weight * signal.strength.value();
            match signal.direction {
                SignalDirection::Long => {
                    long_weight += score;
                    if best_long.as_ref().is_none_or(|&(_, s)| score > s) {
                        best_long = Some((signal, score));
                    }
                }
                SignalDirection::Short => {
                    short_weight += score;
                    if best_short.as_ref().is_none_or(|&(_, s)| score > s) {
                        best_short = Some((signal, score));
                    }
                }
                // Exit, ScaleIn, ScaleOut require an open position — while flat
                // they are discarded so they cannot suppress entry signal strength.
                // Their weights still count toward total_potential (denominator).
                SignalDirection::Exit if has_position => {
                    exit_weight += score;
                    if best_exit.as_ref().is_none_or(|&(_, s)| score > s) {
                        best_exit = Some((signal, score));
                    }
                }
                SignalDirection::ScaleIn if has_position => {
                    let frac = signal.scale_fraction.unwrap_or(0.0);
                    scale_in_weight += score;
                    scale_in_frac_score += frac * score;
                    if best_scale_in.as_ref().is_none_or(|&(_, s)| score > s) {
                        best_scale_in = Some((signal, score));
                    }
                }
                SignalDirection::ScaleOut if has_position => {
                    let frac = signal.scale_fraction.unwrap_or(0.0);
                    scale_out_weight += score;
                    scale_out_frac_score += frac * score;
                    if best_scale_out.as_ref().is_none_or(|&(_, s)| score > s) {
                        best_scale_out = Some((signal, score));
                    }
                }
                _ => {}
            }
        }

        // At least one strategy must have cast a non-Hold vote.
        let total_active =
            long_weight + short_weight + exit_weight + scale_in_weight + scale_out_weight;
        if total_active < f64::EPSILON {
            return Signal::hold();
        }

        // Determine winner (strict majority across all five directions; ties → Hold)
        let (winner, winner_score) = if long_weight > short_weight
            && long_weight > exit_weight
            && long_weight > scale_in_weight
            && long_weight > scale_out_weight
        {
            (best_long, long_weight)
        } else if short_weight > long_weight
            && short_weight > exit_weight
            && short_weight > scale_in_weight
            && short_weight > scale_out_weight
        {
            (best_short, short_weight)
        } else if exit_weight > long_weight
            && exit_weight > short_weight
            && exit_weight > scale_in_weight
            && exit_weight > scale_out_weight
        {
            (best_exit, exit_weight)
        } else if scale_in_weight > long_weight
            && scale_in_weight > short_weight
            && scale_in_weight > exit_weight
            && scale_in_weight > scale_out_weight
        {
            (best_scale_in, scale_in_weight)
        } else if scale_out_weight > long_weight
            && scale_out_weight > short_weight
            && scale_out_weight > exit_weight
            && scale_out_weight > scale_in_weight
        {
            (best_scale_out, scale_out_weight)
        } else {
            return Signal::hold();
        };

        let Some((mut sig, _)) = winner else {
            return Signal::hold();
        };

        // Strength = winner score / total potential (not just active votes).
        sig.strength = SignalStrength::clamped(winner_score / total_potential);

        // Replace inherited scale_fraction with the conviction-weighted average
        // of all same-direction voters to avoid single-contributor size bias.
        if sig.direction == SignalDirection::ScaleIn && scale_in_weight > f64::EPSILON {
            sig.scale_fraction = Some((scale_in_frac_score / scale_in_weight).clamp(0.0, 1.0));
        } else if sig.direction == SignalDirection::ScaleOut && scale_out_weight > f64::EPSILON {
            sig.scale_fraction = Some((scale_out_frac_score / scale_out_weight).clamp(0.0, 1.0));
        }

        sig.reason = Some(format!(
            "WeightedMajority: long={long_weight:.2} short={short_weight:.2} \
             exit={exit_weight:.2} scale_in={scale_in_weight:.2} scale_out={scale_out_weight:.2}"
        ));
        sig
    }

    fn strongest_signal(&self, ctx: &StrategyContext) -> Signal {
        self.strategies
            .iter()
            .map(|(s, _)| s.on_candle(ctx))
            .filter(|s| !s.is_hold())
            .max_by(|a, b| a.strength.value().total_cmp(&b.strength.value()))
            .unwrap_or_else(Signal::hold)
    }
}

impl std::fmt::Debug for EnsembleStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnsembleStrategy")
            .field("name", &self.name)
            .field("strategies_count", &self.strategies.len())
            .field("mode", &self.mode)
            .finish()
    }
}

impl Strategy for EnsembleStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        let mut indicators: Vec<(String, Indicator)> = self
            .strategies
            .iter()
            .flat_map(|(s, _)| s.required_indicators())
            .collect();
        indicators.sort_by(|a, b| a.0.cmp(&b.0));
        indicators.dedup_by(|a, b| a.0 == b.0);
        indicators
    }

    fn warmup_period(&self) -> usize {
        self.strategies
            .iter()
            .map(|(s, _)| s.warmup_period())
            .max()
            .unwrap_or(1)
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if self.strategies.is_empty() {
            return Signal::hold();
        }
        match self.mode {
            EnsembleMode::AnySignal => self.any_signal(ctx),
            EnsembleMode::Unanimous => self.unanimous(ctx),
            EnsembleMode::WeightedMajority => self.weighted_majority(ctx),
            EnsembleMode::StrongestSignal => self.strongest_signal(ctx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::signal::SignalDirection;
    use crate::backtesting::strategy::Strategy;
    use crate::indicators::Indicator;
    use crate::models::chart::Candle;
    use std::collections::HashMap;

    fn make_candle(ts: i64, price: f64) -> Candle {
        Candle {
            timestamp: ts,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 1000,
            adj_close: None,
        }
    }

    fn make_ctx<'a>(
        candles: &'a [Candle],
        indicators: &'a HashMap<String, Vec<Option<f64>>>,
    ) -> StrategyContext<'a> {
        StrategyContext {
            candles,
            index: candles.len() - 1,
            position: None,
            equity: 10_000.0,
            indicators,
        }
    }

    // A strategy that always emits the given direction
    struct FixedStrategy {
        direction: SignalDirection,
        strength: f64,
    }

    impl Strategy for FixedStrategy {
        fn name(&self) -> &str {
            "Fixed"
        }
        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }
        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            match self.direction {
                SignalDirection::Long => {
                    let mut s = Signal::long(ctx.timestamp(), ctx.close());
                    s.strength = SignalStrength::clamped(self.strength);
                    s
                }
                SignalDirection::Short => {
                    let mut s = Signal::short(ctx.timestamp(), ctx.close());
                    s.strength = SignalStrength::clamped(self.strength);
                    s
                }
                SignalDirection::Exit => {
                    let mut s = Signal::exit(ctx.timestamp(), ctx.close());
                    s.strength = SignalStrength::clamped(self.strength);
                    s
                }
                SignalDirection::ScaleIn => {
                    let mut s = Signal::scale_in(0.1, ctx.timestamp(), ctx.close());
                    s.strength = SignalStrength::clamped(self.strength);
                    s
                }
                SignalDirection::ScaleOut => {
                    let mut s = Signal::scale_out(0.5, ctx.timestamp(), ctx.close());
                    s.strength = SignalStrength::clamped(self.strength);
                    s
                }
                _ => Signal::hold(),
            }
        }
    }

    fn make_ctx_with_position<'a>(
        candles: &'a [Candle],
        indicators: &'a HashMap<String, Vec<Option<f64>>>,
        position: &'a crate::backtesting::Position,
    ) -> StrategyContext<'a> {
        StrategyContext {
            candles,
            index: candles.len() - 1,
            position: Some(position),
            equity: 10_000.0,
            indicators,
        }
    }

    fn candles() -> Vec<Candle> {
        vec![make_candle(1, 100.0), make_candle(2, 101.0)]
    }

    fn empty_indicators() -> HashMap<String, Vec<Option<f64>>> {
        HashMap::new()
    }

    #[test]
    fn test_any_signal_returns_first_non_hold() {
        let c = candles();
        let ind = empty_indicators();
        let ctx = make_ctx(&c, &ind);

        let ensemble = EnsembleStrategy::new("test")
            .add(
                FixedStrategy {
                    direction: SignalDirection::Hold,
                    strength: 1.0,
                },
                1.0,
            )
            .add(
                FixedStrategy {
                    direction: SignalDirection::Long,
                    strength: 0.8,
                },
                1.0,
            )
            .add(
                FixedStrategy {
                    direction: SignalDirection::Short,
                    strength: 1.0,
                },
                1.0,
            )
            .mode(EnsembleMode::AnySignal)
            .build();

        let signal = ensemble.on_candle(&ctx);
        assert_eq!(signal.direction, SignalDirection::Long);
    }

    #[test]
    fn test_unanimous_all_agree() {
        let c = candles();
        let ind = empty_indicators();
        let ctx = make_ctx(&c, &ind);

        let ensemble = EnsembleStrategy::new("test")
            .add(
                FixedStrategy {
                    direction: SignalDirection::Long,
                    strength: 1.0,
                },
                1.0,
            )
            .add(
                FixedStrategy {
                    direction: SignalDirection::Long,
                    strength: 0.6,
                },
                1.0,
            )
            .mode(EnsembleMode::Unanimous)
            .build();

        let signal = ensemble.on_candle(&ctx);
        assert_eq!(signal.direction, SignalDirection::Long);
        assert!((signal.strength.value() - 0.8).abs() < 1e-9); // avg of 1.0 and 0.6
    }

    #[test]
    fn test_unanimous_disagreement_returns_hold() {
        let c = candles();
        let ind = empty_indicators();
        let ctx = make_ctx(&c, &ind);

        let ensemble = EnsembleStrategy::new("test")
            .add(
                FixedStrategy {
                    direction: SignalDirection::Long,
                    strength: 1.0,
                },
                1.0,
            )
            .add(
                FixedStrategy {
                    direction: SignalDirection::Short,
                    strength: 1.0,
                },
                1.0,
            )
            .mode(EnsembleMode::Unanimous)
            .build();

        let signal = ensemble.on_candle(&ctx);
        assert!(signal.is_hold());
    }

    #[test]
    fn test_weighted_majority_long_wins() {
        let c = candles();
        let ind = empty_indicators();
        let ctx = make_ctx(&c, &ind);

        let ensemble = EnsembleStrategy::new("test")
            .add(
                FixedStrategy {
                    direction: SignalDirection::Long,
                    strength: 1.0,
                },
                2.0,
            )
            .add(
                FixedStrategy {
                    direction: SignalDirection::Short,
                    strength: 1.0,
                },
                1.0,
            )
            .mode(EnsembleMode::WeightedMajority)
            .build();

        let signal = ensemble.on_candle(&ctx);
        assert_eq!(signal.direction, SignalDirection::Long);
        // strength = 2.0 / 3.0
        assert!((signal.strength.value() - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_weighted_majority_tie_returns_hold() {
        let c = candles();
        let ind = empty_indicators();
        let ctx = make_ctx(&c, &ind);

        let ensemble = EnsembleStrategy::new("test")
            .add(
                FixedStrategy {
                    direction: SignalDirection::Long,
                    strength: 1.0,
                },
                1.0,
            )
            .add(
                FixedStrategy {
                    direction: SignalDirection::Short,
                    strength: 1.0,
                },
                1.0,
            )
            .mode(EnsembleMode::WeightedMajority)
            .build();

        let signal = ensemble.on_candle(&ctx);
        assert!(signal.is_hold());
    }

    #[test]
    fn test_strongest_signal() {
        let c = candles();
        let ind = empty_indicators();
        let ctx = make_ctx(&c, &ind);

        let ensemble = EnsembleStrategy::new("test")
            .add(
                FixedStrategy {
                    direction: SignalDirection::Long,
                    strength: 0.4,
                },
                1.0,
            )
            .add(
                FixedStrategy {
                    direction: SignalDirection::Short,
                    strength: 0.9,
                },
                1.0,
            )
            .mode(EnsembleMode::StrongestSignal)
            .build();

        let signal = ensemble.on_candle(&ctx);
        assert_eq!(signal.direction, SignalDirection::Short);
        assert!((signal.strength.value() - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_empty_ensemble_returns_hold() {
        let c = candles();
        let ind = empty_indicators();
        let ctx = make_ctx(&c, &ind);

        let ensemble = EnsembleStrategy::new("empty").build();
        assert!(ensemble.on_candle(&ctx).is_hold());
    }

    #[test]
    fn test_warmup_is_max_of_sub_strategies() {
        struct WarmupStrategy(usize);
        impl Strategy for WarmupStrategy {
            fn name(&self) -> &str {
                "Warmup"
            }
            fn required_indicators(&self) -> Vec<(String, Indicator)> {
                vec![]
            }
            fn on_candle(&self, _ctx: &StrategyContext) -> Signal {
                Signal::hold()
            }
            fn warmup_period(&self) -> usize {
                self.0
            }
        }

        let ensemble = EnsembleStrategy::new("test")
            .add(WarmupStrategy(10), 1.0)
            .add(WarmupStrategy(25), 1.0)
            .add(WarmupStrategy(5), 1.0)
            .build();

        assert_eq!(ensemble.warmup_period(), 25);
    }

    #[test]
    fn test_weighted_majority_exit_ignored_when_flat() {
        // Exit votes should not suppress Long conviction when there is no position.
        let c = candles();
        let ind = empty_indicators();
        let ctx = make_ctx(&c, &ind); // position = None

        // Exit weight would dominate if counted (3.0 vs Long 2.0), but while flat
        // it must be discarded and Long should win.
        let ensemble = EnsembleStrategy::new("test")
            .add(
                FixedStrategy {
                    direction: SignalDirection::Long,
                    strength: 1.0,
                },
                2.0,
            )
            .add(
                FixedStrategy {
                    direction: SignalDirection::Exit,
                    strength: 1.0,
                },
                3.0,
            )
            .mode(EnsembleMode::WeightedMajority)
            .build();

        let signal = ensemble.on_candle(&ctx);
        assert_eq!(signal.direction, SignalDirection::Long);
        // strength = 2.0 / 5.0 = 0.4 (exit weight counts in denominator even when
        // its vote is discarded, correctly suppressing overconfidence)
        assert!((signal.strength.value() - 2.0 / 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_weighted_majority_scale_in_wins_when_position_open() {
        use crate::backtesting::{Position, PositionSide};

        let c = candles();
        let ind = empty_indicators();
        let pos = Position::new(
            PositionSide::Long,
            1,
            100.0,
            10.0,
            0.0,
            Signal::long(1, 100.0),
        );
        let ctx = make_ctx_with_position(&c, &ind, &pos);

        // ScaleIn has the highest conviction score (3.0) while Long has 2.0.
        let ensemble = EnsembleStrategy::new("test")
            .add(
                FixedStrategy {
                    direction: SignalDirection::Long,
                    strength: 1.0,
                },
                2.0,
            )
            .add(
                FixedStrategy {
                    direction: SignalDirection::ScaleIn,
                    strength: 1.0,
                },
                3.0,
            )
            .mode(EnsembleMode::WeightedMajority)
            .build();

        let signal = ensemble.on_candle(&ctx);
        assert_eq!(signal.direction, SignalDirection::ScaleIn);
        // strength = 3.0 / 5.0
        assert!((signal.strength.value() - 0.6).abs() < 1e-9);
        // scale_fraction is the conviction-weighted average of all ScaleIn voters
        let frac = signal.scale_fraction.expect("scale_fraction must be set");
        assert!((frac - 0.1).abs() < 1e-9, "expected 0.10, got {frac}");
    }

    #[test]
    fn test_weighted_majority_scale_fraction_is_conviction_weighted_average() {
        use crate::backtesting::{Position, PositionSide};

        let c = candles();
        let ind = empty_indicators();
        let pos = Position::new(
            PositionSide::Long,
            1,
            100.0,
            10.0,
            0.0,
            Signal::long(1, 100.0),
        );
        let ctx = make_ctx_with_position(&c, &ind, &pos);

        // Strategy A: ScaleOut 10% with score 1.0 × 1.0 = 1.0
        // Strategy B: ScaleOut 50% with score 1.0 × 1.0 = 1.0
        // Weighted-average fraction = (0.10 × 1.0 + 0.50 × 1.0) / 2.0 = 0.30
        struct ScaleOutStrategy {
            fraction: f64,
        }
        impl Strategy for ScaleOutStrategy {
            fn name(&self) -> &str {
                "ScaleOut"
            }
            fn required_indicators(&self) -> Vec<(String, Indicator)> {
                vec![]
            }
            fn on_candle(&self, ctx: &StrategyContext) -> Signal {
                Signal::scale_out(self.fraction, ctx.timestamp(), ctx.close())
            }
        }

        let ensemble = EnsembleStrategy::new("test")
            .add(ScaleOutStrategy { fraction: 0.10 }, 1.0)
            .add(ScaleOutStrategy { fraction: 0.50 }, 1.0)
            .mode(EnsembleMode::WeightedMajority)
            .build();

        let signal = ensemble.on_candle(&ctx);
        assert_eq!(signal.direction, SignalDirection::ScaleOut);
        let frac = signal.scale_fraction.expect("scale_fraction must be set");
        assert!((frac - 0.30).abs() < 1e-9, "expected 0.30, got {frac}");
    }

    #[test]
    fn test_weighted_majority_scale_in_ignored_when_flat() {
        let c = candles();
        let ind = empty_indicators();
        let ctx = make_ctx(&c, &ind); // position = None

        // ScaleIn would dominate (3.0 vs Long 2.0) but must be ignored while flat.
        let ensemble = EnsembleStrategy::new("test")
            .add(
                FixedStrategy {
                    direction: SignalDirection::Long,
                    strength: 1.0,
                },
                2.0,
            )
            .add(
                FixedStrategy {
                    direction: SignalDirection::ScaleIn,
                    strength: 1.0,
                },
                3.0,
            )
            .mode(EnsembleMode::WeightedMajority)
            .build();

        let signal = ensemble.on_candle(&ctx);
        assert_eq!(signal.direction, SignalDirection::Long);
        // strength = 2.0 / 5.0 = 0.4 (scale_in weight counts in denominator even
        // when its vote is discarded while flat)
        assert!((signal.strength.value() - 2.0 / 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_required_indicators_deduplication() {
        struct IndStrategy(Vec<(String, Indicator)>);
        impl Strategy for IndStrategy {
            fn name(&self) -> &str {
                "Ind"
            }
            fn required_indicators(&self) -> Vec<(String, Indicator)> {
                self.0.clone()
            }
            fn on_candle(&self, _ctx: &StrategyContext) -> Signal {
                Signal::hold()
            }
        }

        let ensemble = EnsembleStrategy::new("test")
            .add(
                IndStrategy(vec![
                    ("sma_10".to_string(), Indicator::Sma(10)),
                    ("sma_20".to_string(), Indicator::Sma(20)),
                ]),
                1.0,
            )
            .add(
                IndStrategy(vec![
                    ("sma_20".to_string(), Indicator::Sma(20)), // duplicate
                    ("rsi_14".to_string(), Indicator::Rsi(14)),
                ]),
                1.0,
            )
            .build();

        let indicators = ensemble.required_indicators();
        assert_eq!(indicators.len(), 3);
        assert!(indicators.iter().any(|(k, _)| k == "sma_10"));
        assert!(indicators.iter().any(|(k, _)| k == "sma_20"));
        assert!(indicators.iter().any(|(k, _)| k == "rsi_14"));
    }
}

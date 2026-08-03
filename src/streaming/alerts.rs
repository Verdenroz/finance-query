//! Threshold-triggered alerting over a price subscription.
//!
//! Consumers that only care about specific crossings would otherwise receive
//! every tick and filter client-side. [`AlertEvaluator`] moves that predicate
//! next to the stream; [`AlertStream`] wraps it as a `Stream` adapter so it
//! composes with any `Stream<Item = PriceUpdate>` (including a server-side
//! shared hub stream) rather than being welded to one transport.

use std::collections::{HashMap, VecDeque};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::Stream;
use serde::{Deserialize, Serialize};

use super::pricing::PriceUpdate;

/// A predicate over incoming price ticks.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum AlertCondition {
    /// Price moved from at-or-below the threshold to above it.
    CrossesAbove(f64),
    /// Price moved from at-or-above the threshold to below it.
    CrossesBelow(f64),
    /// Price is above the threshold (fires on the first matching tick).
    PriceAbove(f64),
    /// Price is below the threshold (fires on the first matching tick).
    PriceBelow(f64),
    /// Percent change from the previous close is at or above the threshold.
    PercentChangeAbove(f64),
    /// Percent change from the previous close is at or below the threshold.
    PercentChangeBelow(f64),
    /// Day volume is at or above the threshold.
    VolumeAbove(i64),
}

impl AlertCondition {
    /// Whether the condition holds for this tick.
    ///
    /// `previous` is the last price seen for the symbol; crossing conditions
    /// need it and never fire on the first tick of a symbol.
    fn holds(&self, update: &PriceUpdate, previous: Option<f32>) -> bool {
        let price = update.price as f64;
        match *self {
            Self::CrossesAbove(t) => previous.is_some_and(|p| (p as f64) <= t) && price > t,
            Self::CrossesBelow(t) => previous.is_some_and(|p| (p as f64) >= t) && price < t,
            Self::PriceAbove(t) => price > t,
            Self::PriceBelow(t) => price < t,
            Self::PercentChangeAbove(t) => update.change_percent as f64 >= t,
            Self::PercentChangeBelow(t) => update.change_percent as f64 <= t,
            Self::VolumeAbove(t) => update.day_volume >= t,
        }
    }
}

/// A symbol paired with the condition that should trigger an alert.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AlertRule {
    /// Symbol this rule watches (matched against `PriceUpdate::id`).
    pub symbol: String,
    /// Predicate to evaluate.
    pub condition: AlertCondition,
    /// Fire repeatedly (re-arming whenever the condition stops holding)
    /// instead of once.
    pub repeat: bool,
}

impl AlertRule {
    /// A one-shot rule: fires the first time the condition holds.
    pub fn new(symbol: impl Into<String>, condition: AlertCondition) -> Self {
        Self {
            symbol: symbol.into(),
            condition,
            repeat: false,
        }
    }

    /// Fire every time the condition becomes true again.
    pub fn repeating(mut self) -> Self {
        self.repeat = true;
        self
    }
}

/// A fired alert, carrying the tick that triggered it.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AlertEvent {
    /// Symbol that triggered.
    pub symbol: String,
    /// Condition that fired.
    pub condition: AlertCondition,
    /// Price on the triggering tick.
    pub price: f32,
    /// Last price seen before this tick, if any.
    pub previous_price: Option<f32>,
    /// Percent change carried by the triggering tick.
    pub change_percent: f32,
    /// Tick timestamp (milliseconds).
    pub time: i64,
    /// The full triggering tick.
    pub update: PriceUpdate,
}

/// Per-rule firing state.
struct RuleState {
    rule: AlertRule,
    /// `false` after firing; a repeating rule re-arms when the condition
    /// stops holding, a one-shot rule never does.
    armed: bool,
}

/// Evaluates [`AlertRule`]s against a price feed, tracking the per-symbol
/// history that crossing conditions need.
///
/// Exposed so consumers that already own a price stream (the server's shared
/// hub, for instance) can apply alerts without re-wrapping the stream.
pub struct AlertEvaluator {
    rules: Vec<RuleState>,
    last_price: HashMap<String, f32>,
}

impl AlertEvaluator {
    /// Build an evaluator from a rule set.
    pub fn new(rules: impl IntoIterator<Item = AlertRule>) -> Self {
        Self {
            rules: rules
                .into_iter()
                .map(|rule| RuleState { rule, armed: true })
                .collect(),
            last_price: HashMap::new(),
        }
    }

    /// Distinct symbols referenced by the rule set, in first-seen order.
    pub fn symbols(&self) -> Vec<String> {
        let mut symbols: Vec<String> = Vec::new();
        for state in &self.rules {
            if !symbols.contains(&state.rule.symbol) {
                symbols.push(state.rule.symbol.clone());
            }
        }
        symbols
    }

    /// `true` once every rule has fired and none can fire again.
    pub fn is_exhausted(&self) -> bool {
        self.rules
            .iter()
            .all(|state| !state.armed && !state.rule.repeat)
    }

    /// Feed one tick, returning the alerts it triggered.
    pub fn evaluate(&mut self, update: &PriceUpdate) -> Vec<AlertEvent> {
        let previous = self.last_price.get(&update.id).copied();
        let mut fired = Vec::new();

        for state in self.rules.iter_mut() {
            if state.rule.symbol != update.id {
                continue;
            }
            let holds = state.rule.condition.holds(update, previous);
            if holds && state.armed {
                state.armed = false;
                fired.push(AlertEvent {
                    symbol: update.id.clone(),
                    condition: state.rule.condition,
                    price: update.price,
                    previous_price: previous,
                    change_percent: update.change_percent,
                    time: update.time,
                    update: update.clone(),
                });
            } else if !holds && state.rule.repeat {
                state.armed = true;
            }
        }

        // Heartbeats and other priceless ticks must not poison the crossing
        // history with a zero.
        if update.price != 0.0 {
            self.last_price.insert(update.id.clone(), update.price);
        }
        fired
    }
}

/// A `Stream<Item = AlertEvent>` over any price stream.
pub struct AlertStream<S> {
    inner: S,
    evaluator: AlertEvaluator,
    pending: VecDeque<AlertEvent>,
}

impl<S> AlertStream<S> {
    /// Wrap `inner`, evaluating `rules` against every tick it yields.
    pub fn new(inner: S, rules: impl IntoIterator<Item = AlertRule>) -> Self {
        Self {
            inner,
            evaluator: AlertEvaluator::new(rules),
            pending: VecDeque::new(),
        }
    }
}

impl<S> Stream for AlertStream<S>
where
    S: Stream<Item = PriceUpdate> + Unpin,
{
    type Item = AlertEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            if let Some(event) = this.pending.pop_front() {
                return Poll::Ready(Some(event));
            }
            match Pin::new(&mut this.inner).poll_next(cx) {
                Poll::Ready(Some(update)) => this.pending.extend(this.evaluator.evaluate(&update)),
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Adds [`alerts`](AlertExt::alerts) to every price stream.
///
/// # Example
///
/// ```no_run
/// use finance_query::streaming::{AlertCondition, AlertExt, AlertRule, PriceStream};
/// use futures::StreamExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut alerts = PriceStream::subscribe(["AAPL"])
///     .await?
///     .alerts([AlertRule::new("AAPL", AlertCondition::CrossesAbove(200.0))]);
///
/// while let Some(alert) = alerts.next().await {
///     println!("{} crossed at {}", alert.symbol, alert.price);
/// }
/// # Ok(())
/// # }
/// ```
pub trait AlertExt: Stream<Item = PriceUpdate> + Sized + Unpin {
    /// Yield only the ticks that trigger one of `rules`.
    fn alerts(self, rules: impl IntoIterator<Item = AlertRule>) -> AlertStream<Self> {
        AlertStream::new(self, rules)
    }
}

impl<S> AlertExt for S where S: Stream<Item = PriceUpdate> + Sized + Unpin {}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    fn tick(symbol: &str, price: f32) -> PriceUpdate {
        PriceUpdate {
            id: symbol.to_string(),
            price,
            ..Default::default()
        }
    }

    #[test]
    fn crossing_needs_a_previous_price() {
        let mut evaluator =
            AlertEvaluator::new([AlertRule::new("AAPL", AlertCondition::CrossesAbove(150.0))]);

        // First tick is already above the threshold but has no history.
        assert!(evaluator.evaluate(&tick("AAPL", 155.0)).is_empty());
    }

    #[test]
    fn crossing_fires_once_on_the_upward_move() {
        let mut evaluator =
            AlertEvaluator::new([AlertRule::new("AAPL", AlertCondition::CrossesAbove(150.0))]);

        assert!(evaluator.evaluate(&tick("AAPL", 149.0)).is_empty());
        let fired = evaluator.evaluate(&tick("AAPL", 151.0));
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].previous_price, Some(149.0));

        // One-shot: no repeat while it stays above, and none on a re-cross.
        assert!(evaluator.evaluate(&tick("AAPL", 152.0)).is_empty());
        assert!(evaluator.evaluate(&tick("AAPL", 148.0)).is_empty());
        assert!(evaluator.evaluate(&tick("AAPL", 153.0)).is_empty());
        assert!(evaluator.is_exhausted());
    }

    #[test]
    fn repeating_rules_rearm_when_the_condition_clears() {
        let mut evaluator =
            AlertEvaluator::new([
                AlertRule::new("AAPL", AlertCondition::CrossesAbove(150.0)).repeating()
            ]);

        evaluator.evaluate(&tick("AAPL", 149.0));
        assert_eq!(evaluator.evaluate(&tick("AAPL", 151.0)).len(), 1);
        assert!(evaluator.evaluate(&tick("AAPL", 152.0)).is_empty());
        // Falls back below (clearing the condition), then crosses again.
        assert!(evaluator.evaluate(&tick("AAPL", 148.0)).is_empty());
        assert_eq!(evaluator.evaluate(&tick("AAPL", 151.0)).len(), 1);
        assert!(!evaluator.is_exhausted());
    }

    #[test]
    fn crossing_below_is_symmetric() {
        let mut evaluator =
            AlertEvaluator::new([AlertRule::new("AAPL", AlertCondition::CrossesBelow(100.0))]);
        evaluator.evaluate(&tick("AAPL", 101.0));
        assert_eq!(evaluator.evaluate(&tick("AAPL", 99.0)).len(), 1);
    }

    #[test]
    fn level_and_metric_conditions_fire_without_history() {
        let mut level =
            AlertEvaluator::new([AlertRule::new("AAPL", AlertCondition::PriceAbove(10.0))]);
        assert_eq!(level.evaluate(&tick("AAPL", 11.0)).len(), 1);

        let mut pct = AlertEvaluator::new([AlertRule::new(
            "AAPL",
            AlertCondition::PercentChangeAbove(5.0),
        )]);
        let mut update = tick("AAPL", 11.0);
        update.change_percent = 6.0;
        assert_eq!(pct.evaluate(&update).len(), 1);

        let mut vol =
            AlertEvaluator::new([AlertRule::new("AAPL", AlertCondition::VolumeAbove(1_000))]);
        let mut update = tick("AAPL", 11.0);
        update.day_volume = 1_500;
        assert_eq!(vol.evaluate(&update).len(), 1);
    }

    #[test]
    fn rules_only_see_their_own_symbol() {
        let mut evaluator = AlertEvaluator::new([
            AlertRule::new("AAPL", AlertCondition::PriceAbove(10.0)),
            AlertRule::new("NVDA", AlertCondition::PriceAbove(10.0)),
        ]);
        let fired = evaluator.evaluate(&tick("NVDA", 20.0));
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].symbol, "NVDA");
        assert_eq!(evaluator.symbols(), vec!["AAPL", "NVDA"]);
    }

    #[test]
    fn priceless_ticks_do_not_become_crossing_history() {
        let mut evaluator =
            AlertEvaluator::new([AlertRule::new("AAPL", AlertCondition::CrossesAbove(150.0))]);
        evaluator.evaluate(&tick("AAPL", 149.0));
        // A heartbeat-style tick with no price must not reset the baseline.
        evaluator.evaluate(&tick("AAPL", 0.0));
        assert_eq!(evaluator.evaluate(&tick("AAPL", 151.0)).len(), 1);
    }

    #[tokio::test]
    async fn stream_adapter_yields_only_triggering_ticks() {
        let updates = futures::stream::iter(vec![
            tick("AAPL", 149.0),
            tick("AAPL", 149.5),
            tick("AAPL", 151.0),
            tick("AAPL", 152.0),
        ]);

        let alerts: Vec<AlertEvent> = updates
            .alerts([AlertRule::new("AAPL", AlertCondition::CrossesAbove(150.0))])
            .collect()
            .await;

        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].price, 151.0);
        assert_eq!(alerts[0].update.id, "AAPL");
    }
}

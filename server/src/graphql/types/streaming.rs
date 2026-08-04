//! GraphQL type for real-time price streaming updates.

use async_graphql::{Enum, InputObject, SimpleObject};
use finance_query::streaming::{AlertConditionKind, AlertEvent, AlertRule, PriceUpdate};
use serde::Deserialize;

/// A real-time price update from the streaming WebSocket, mirroring `PriceUpdate`.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlPriceUpdate {
    pub id: String,
    pub price: f32,
    pub time: i64,
    pub currency: String,
    pub exchange: String,
    pub quote_type: String,
    pub market_hours: String,
    pub change_percent: f32,
    pub day_volume: i64,
    pub day_high: f32,
    pub day_low: f32,
    pub change: f32,
    pub short_name: String,
    pub expire_date: i64,
    pub open_price: f32,
    pub previous_close: f32,
    pub strike_price: f32,
    pub underlying_symbol: String,
    pub open_interest: i64,
    pub options_type: String,
    pub mini_option: i64,
    pub last_size: i64,
    pub bid: f32,
    pub bid_size: i64,
    pub ask: f32,
    pub ask_size: i64,
    pub price_hint: i64,
    pub vol_24hr: i64,
    pub vol_all_currencies: i64,
    pub from_currency: String,
    pub last_market: String,
    pub circulating_supply: f64,
    pub market_cap: f64,
}

impl From<PriceUpdate> for GqlPriceUpdate {
    fn from(u: PriceUpdate) -> Self {
        Self {
            id: u.id,
            price: u.price,
            time: u.time,
            currency: u.currency,
            exchange: u.exchange,
            quote_type: format!("{:?}", u.quote_type),
            market_hours: format!("{:?}", u.market_hours),
            change_percent: u.change_percent,
            day_volume: u.day_volume,
            day_high: u.day_high,
            day_low: u.day_low,
            change: u.change,
            short_name: u.short_name,
            expire_date: u.expire_date,
            open_price: u.open_price,
            previous_close: u.previous_close,
            strike_price: u.strike_price,
            underlying_symbol: u.underlying_symbol,
            open_interest: u.open_interest,
            options_type: format!("{:?}", u.options_type),
            mini_option: u.mini_option,
            last_size: u.last_size,
            bid: u.bid,
            bid_size: u.bid_size,
            ask: u.ask,
            ask_size: u.ask_size,
            price_hint: u.price_hint,
            vol_24hr: u.vol_24hr,
            vol_all_currencies: u.vol_all_currencies,
            from_currency: u.from_currency,
            last_market: u.last_market,
            circulating_supply: u.circulating_supply,
            market_cap: u.market_cap,
        }
    }
}

/// Threshold predicate for a price alert subscription.
///
/// Mirrors [`AlertConditionKind`] one-for-one; the `From` impls below are
/// exhaustive both ways, so a new library predicate fails to compile here
/// instead of silently degrading.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum GqlAlertConditionKind {
    /// Price moved from at-or-below `value` to above it.
    CrossesAbove,
    /// Price moved from at-or-above `value` to below it.
    CrossesBelow,
    /// Price is above `value`.
    PriceAbove,
    /// Price is below `value`.
    PriceBelow,
    /// Percent change is at or above `value`.
    PercentChangeAbove,
    /// Percent change is at or below `value`.
    PercentChangeBelow,
    /// Day volume is at or above `value`.
    VolumeAbove,
}

/// One alert rule supplied by a subscriber.
#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlAlertRuleInput {
    /// Symbol to watch.
    pub symbol: String,
    /// Predicate to evaluate.
    pub condition: GqlAlertConditionKind,
    /// Threshold the predicate compares against.
    pub value: f64,
    /// Fire every time the condition becomes true again (default: once).
    #[graphql(default = false)]
    pub repeat: bool,
}

impl From<GqlAlertConditionKind> for AlertConditionKind {
    fn from(kind: GqlAlertConditionKind) -> Self {
        match kind {
            GqlAlertConditionKind::CrossesAbove => Self::CrossesAbove,
            GqlAlertConditionKind::CrossesBelow => Self::CrossesBelow,
            GqlAlertConditionKind::PriceAbove => Self::PriceAbove,
            GqlAlertConditionKind::PriceBelow => Self::PriceBelow,
            GqlAlertConditionKind::PercentChangeAbove => Self::PercentChangeAbove,
            GqlAlertConditionKind::PercentChangeBelow => Self::PercentChangeBelow,
            GqlAlertConditionKind::VolumeAbove => Self::VolumeAbove,
        }
    }
}

impl From<AlertConditionKind> for GqlAlertConditionKind {
    fn from(kind: AlertConditionKind) -> Self {
        match kind {
            AlertConditionKind::CrossesAbove => Self::CrossesAbove,
            AlertConditionKind::CrossesBelow => Self::CrossesBelow,
            AlertConditionKind::PriceAbove => Self::PriceAbove,
            AlertConditionKind::PriceBelow => Self::PriceBelow,
            AlertConditionKind::PercentChangeAbove => Self::PercentChangeAbove,
            AlertConditionKind::PercentChangeBelow => Self::PercentChangeBelow,
            AlertConditionKind::VolumeAbove => Self::VolumeAbove,
        }
    }
}

impl From<GqlAlertRuleInput> for AlertRule {
    fn from(input: GqlAlertRuleInput) -> Self {
        let condition = AlertConditionKind::from(input.condition).with_value(input.value);
        let rule = AlertRule::new(input.symbol, condition);
        if input.repeat { rule.repeating() } else { rule }
    }
}

/// A fired price alert.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlAlertEvent {
    /// Symbol that triggered.
    pub symbol: String,
    /// Which predicate fired.
    pub condition: GqlAlertConditionKind,
    /// Threshold the predicate compared against.
    pub threshold: f64,
    /// Price on the triggering tick.
    pub price: f32,
    /// Last price seen before this tick.
    pub previous_price: Option<f32>,
    /// Percent change carried by the triggering tick.
    pub change_percent: f32,
    /// Tick timestamp (milliseconds).
    pub time: i64,
    /// The full triggering tick.
    pub update: GqlPriceUpdate,
}

impl From<AlertEvent> for GqlAlertEvent {
    fn from(event: AlertEvent) -> Self {
        Self {
            symbol: event.symbol,
            condition: event.condition.kind().into(),
            threshold: event.condition.threshold(),
            price: event.price,
            previous_price: event.previous_price,
            change_percent: event.change_percent,
            time: event.time,
            update: GqlPriceUpdate::from(event.update),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use finance_query::streaming::AlertCondition;

    fn input(condition: GqlAlertConditionKind, value: f64, repeat: bool) -> GqlAlertRuleInput {
        GqlAlertRuleInput {
            symbol: "AAPL".into(),
            condition,
            value,
            repeat,
        }
    }

    #[test]
    fn rule_input_maps_onto_library_conditions() {
        let rule: AlertRule = input(GqlAlertConditionKind::CrossesAbove, 150.0, false).into();
        assert_eq!(rule.symbol, "AAPL");
        assert_eq!(rule.condition, AlertCondition::CrossesAbove(150.0));
        assert!(!rule.repeat);

        let repeating: AlertRule = input(GqlAlertConditionKind::PriceBelow, 10.0, true).into();
        assert!(repeating.repeat);
        assert_eq!(repeating.condition, AlertCondition::PriceBelow(10.0));
    }

    #[test]
    fn volume_thresholds_round_trip_through_the_float_input() {
        let rule: AlertRule = input(GqlAlertConditionKind::VolumeAbove, 1_000.0, false).into();
        assert_eq!(rule.condition, AlertCondition::VolumeAbove(1_000));
        assert_eq!(
            (
                GqlAlertConditionKind::from(rule.condition.kind()),
                rule.condition.threshold()
            ),
            (GqlAlertConditionKind::VolumeAbove, 1_000.0)
        );
    }
}

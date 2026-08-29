//! GraphQL types for strategy backtesting.
//!
//! These mirror `finance_query::backtesting` and are deserialized from the
//! service layer's cached JSON, so field names must stay identical to the
//! library's own snake_case serialization.

use async_graphql::{ComplexObject, Result, SimpleObject};
use serde::Deserialize;

use crate::graphql::pagination::{Page, paginate};
use crate::graphql::types::enums::{GqlInterval, GqlTimeRange};

/// The one place a backtest's default window is defined. REST, GraphQL and the
/// MCP tool all derive theirs from these, so the three cannot drift apart.
pub const BACKTEST_DEFAULT_INTERVAL: GqlInterval = GqlInterval::OneDay;
/// A month of bars says nothing about a strategy, so this is not the chart default.
pub const BACKTEST_DEFAULT_RANGE: GqlTimeRange = GqlTimeRange::OneYear;

/// The signal that opened or closed a trade.
///
/// Carries the direction, conviction and context a trade log needs; the
/// engine-side ordering knobs on `finance_query::backtesting::Signal`
/// (order type, bracket overrides, expiry) are inputs, not results.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlTradeSignal {
    /// `SignalDirection` is `#[non_exhaustive]`, so a new library variant
    /// must not fail deserialization here.
    pub direction: String,
    pub strength: f64,
    pub timestamp: i64,
    pub price: f64,
    pub reason: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Portfolio value at one bar.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlEquityPoint {
    pub timestamp: i64,
    pub equity: f64,
    pub drawdown_pct: f64,
}

/// One completed round trip.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlTrade {
    pub side: String,
    pub entry_timestamp: i64,
    pub exit_timestamp: i64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub entry_quantity: f64,
    pub commission: f64,
    pub transaction_tax: f64,
    pub pnl: f64,
    pub return_pct: f64,
    pub dividend_income: f64,
    pub unreinvested_dividends: f64,
    pub financing_cost: f64,
    pub entry_signal: GqlTradeSignal,
    pub exit_signal: GqlTradeSignal,
    pub tags: Vec<String>,
    pub is_partial: bool,
    pub scale_sequence: u32,
}

/// Performance summary over the whole run.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlBacktestMetrics {
    pub total_return_pct: f64,
    pub annualized_return_pct: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown_pct: f64,
    pub max_drawdown_duration: i64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub avg_trade_return_pct: f64,
    pub avg_win_pct: f64,
    pub avg_loss_pct: f64,
    pub avg_trade_duration: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub max_consecutive_wins: u32,
    pub max_consecutive_losses: u32,
    pub calmar_ratio: f64,
    pub total_commission: f64,
    pub total_financing_cost: f64,
    pub long_trades: u32,
    pub short_trades: u32,
    pub total_signals: u32,
    pub executed_signals: u32,
    pub avg_win_duration: f64,
    pub avg_loss_duration: f64,
    pub time_in_market_pct: f64,
    pub max_idle_period: i64,
    pub total_dividend_income: f64,
    pub kelly_criterion: f64,
    pub sqn: f64,
    pub expectancy: f64,
    pub omega_ratio: f64,
    pub tail_ratio: f64,
    pub recovery_factor: f64,
    pub ulcer_index: f64,
    pub serenity_ratio: f64,
}

/// A completed backtest.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase", complex)]
pub struct GqlBacktestResult {
    pub symbol: String,
    pub strategy_name: String,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub initial_capital: f64,
    pub final_equity: f64,
    pub max_leverage_used: f64,
    pub metrics: GqlBacktestMetrics,
    #[serde(default)]
    pub diagnostics: Vec<String>,
    #[graphql(skip)]
    #[serde(default)]
    pub equity_curve: Vec<GqlEquityPoint>,
    #[graphql(skip)]
    #[serde(default)]
    pub trades: Vec<GqlTrade>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlBacktestResult {
    /// Portfolio value per bar, oldest first.
    async fn equity_curve(
        &self,
        first: Option<i32>,
        after: Option<String>,
    ) -> Result<Page<GqlEquityPoint>> {
        paginate(&self.equity_curve, first, after).await
    }

    /// Completed round trips, oldest first.
    async fn trades(&self, first: Option<i32>, after: Option<String>) -> Result<Page<GqlTrade>> {
        paginate(&self.trades, first, after).await
    }
}

/// The prebuilt strategies a backtest can run.
#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GqlStrategy {
    SmaCrossover,
    RsiReversal,
    MacdSignal,
    BollingerMeanReversion,
    SupertrendFollow,
    DonchianBreakout,
}

impl GqlStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SmaCrossover => "sma_crossover",
            Self::RsiReversal => "rsi_reversal",
            Self::MacdSignal => "macd_signal",
            Self::BollingerMeanReversion => "bollinger_mean_reversion",
            Self::SupertrendFollow => "supertrend_follow",
            Self::DonchianBreakout => "donchian_breakout",
        }
    }
}

/// Strategy and execution knobs. Every field falls back to the library default.
#[derive(async_graphql::InputObject, serde::Serialize, Deserialize, Default, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlBacktestParams {
    pub fast_period: Option<u32>,
    pub slow_period: Option<u32>,
    pub period: Option<u32>,
    pub signal_period: Option<u32>,
    pub std_dev: Option<f64>,
    pub multiplier: Option<f64>,
    pub oversold: Option<f64>,
    pub overbought: Option<f64>,
    pub exit_at_middle: Option<bool>,
    pub initial_capital: Option<f64>,
    pub commission_pct: Option<f64>,
    pub slippage_pct: Option<f64>,
    pub position_size_pct: Option<f64>,
    pub allow_short: Option<bool>,
    pub stop_loss_pct: Option<f64>,
    pub take_profit_pct: Option<f64>,
    pub max_leverage: Option<f64>,
    pub maintenance_margin_pct: Option<f64>,
    pub short_borrow_rate: Option<f64>,
    pub margin_interest_rate: Option<f64>,
}

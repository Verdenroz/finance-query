//! `run_backtest` tool: strategy simulation over prebuilt strategies.
//!
//! Unlike most tools in this crate, this one does not bridge through
//! `FinanceSchema`/GraphQL — the backtesting engine isn't part of the
//! GraphQL schema (it's a compute-heavy simulation, not a data fetch), so
//! this calls `finance_query::backtesting` directly via `Ticker::backtest`.
//! Pagination for `equity_curve`/`trades` is therefore hand-rolled here
//! rather than reusing the GraphQL `Connection` plumbing in `tools::gql`,
//! but follows the same `{items, pageInfo: {hasNextPage, endCursor}}` shape.

use finance_query::Ticker;
use finance_query::backtesting::{
    BacktestConfig, BacktestResult, BollingerMeanReversion, DonchianBreakout, MacdSignal,
    RsiReversal, SmaCrossover, Strategy, SuperTrendFollow,
};
use rmcp::{ErrorData as McpError, model::CallToolResult};
use serde::Serialize;

use crate::error::{invalid_params, lib_err, ser_err};
use crate::tools::RunBacktestParams;
use crate::tools::gql::DEFAULT_MCP_PAGE_SIZE;
use crate::tools::helpers::{parse_interval, parse_range};

const VALID_STRATEGIES: &str = "sma_crossover, rsi_reversal, macd_signal, bollinger_mean_reversion, supertrend_follow, donchian_breakout";

/// Build the requested prebuilt strategy as a trait object (the engine's
/// `Strategy` trait already has `impl Strategy for Box<dyn Strategy>`).
fn build_strategy(p: &RunBacktestParams) -> Result<Box<dyn Strategy>, McpError> {
    let strategy: Box<dyn Strategy> = match p.strategy.as_str() {
        "sma_crossover" => Box::new(SmaCrossover::new(
            p.fast_period.unwrap_or(10) as usize,
            p.slow_period.unwrap_or(20) as usize,
        )),
        "rsi_reversal" => {
            let mut s = RsiReversal::new(p.period.unwrap_or(14) as usize);
            if p.oversold.is_some() || p.overbought.is_some() {
                s = s.with_thresholds(p.oversold.unwrap_or(30.0), p.overbought.unwrap_or(70.0));
            }
            Box::new(s)
        }
        "macd_signal" => Box::new(MacdSignal::new(
            p.fast_period.unwrap_or(12) as usize,
            p.slow_period.unwrap_or(26) as usize,
            p.signal_period.unwrap_or(9) as usize,
        )),
        "bollinger_mean_reversion" => {
            let mut s = BollingerMeanReversion::new(
                p.period.unwrap_or(20) as usize,
                p.std_dev.unwrap_or(2.0),
            );
            if let Some(em) = p.exit_at_middle {
                s = s.exit_at_middle(em);
            }
            Box::new(s)
        }
        "supertrend_follow" => Box::new(SuperTrendFollow::new(
            p.period.unwrap_or(10) as usize,
            p.multiplier.unwrap_or(3.0),
        )),
        "donchian_breakout" => {
            let mut s = DonchianBreakout::new(p.period.unwrap_or(20) as usize);
            if let Some(em) = p.exit_at_middle {
                s = s.exit_at_middle(em);
            }
            Box::new(s)
        }
        other => {
            return Err(invalid_params(format!(
                "Unknown strategy '{other}'. Valid strategies: {VALID_STRATEGIES}"
            )));
        }
    };
    Ok(strategy)
}

/// Build the `BacktestConfig`. Every knob defaults to the library's own
/// `BacktestConfig::default()` value, so an all-omitted param set behaves
/// identically to calling the library directly with no config override.
fn build_config(p: &RunBacktestParams) -> Result<BacktestConfig, McpError> {
    let mut builder = BacktestConfig::builder()
        .initial_capital(p.initial_capital.unwrap_or(10_000.0))
        .commission_pct(p.commission_pct.unwrap_or(0.001))
        .slippage_pct(p.slippage_pct.unwrap_or(0.001))
        .position_size_pct(p.position_size_pct.unwrap_or(1.0))
        .allow_short(p.allow_short.unwrap_or(false));
    if let Some(pct) = p.stop_loss_pct {
        builder = builder.stop_loss_pct(pct);
    }
    if let Some(pct) = p.take_profit_pct {
        builder = builder.take_profit_pct(pct);
    }
    if let Some(leverage) = p.max_leverage {
        builder = builder.max_leverage(leverage);
    }
    if let Some(pct) = p.maintenance_margin_pct {
        builder = builder.maintenance_margin_pct(pct);
    }
    if let Some(rate) = p.short_borrow_rate {
        builder = builder.short_borrow_rate(rate);
    }
    if let Some(rate) = p.margin_interest_rate {
        builder = builder.margin_interest_rate(rate);
    }
    builder.build().map_err(lib_err)
}

/// Slice `items` starting at the offset encoded in `cursor` (default 0),
/// returning up to `limit` (default [`DEFAULT_MCP_PAGE_SIZE`]) entries plus
/// a `{hasNextPage, endCursor}` pair. `endCursor` here is simply the next
/// page's start offset as a string — an implementation detail, not a
/// guaranteed encoding, same as every other opaque MCP cursor in this crate.
fn paginate<T: Serialize>(
    items: &[T],
    limit: Option<u32>,
    cursor: Option<&str>,
) -> serde_json::Value {
    let start = cursor
        .and_then(|c| c.parse::<usize>().ok())
        .unwrap_or(0)
        .min(items.len());
    let page_size = limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE) as usize;
    let end = start.saturating_add(page_size).min(items.len());
    let has_next_page = end < items.len();
    serde_json::json!({
        "items": items[start..end],
        "pageInfo": {
            "hasNextPage": has_next_page,
            "endCursor": if has_next_page { Some(end.to_string()) } else { None::<String> },
        }
    })
}

fn build_response(result: &BacktestResult, p: &RunBacktestParams) -> serde_json::Value {
    serde_json::json!({
        "symbol": result.symbol,
        "strategy": result.strategy_name,
        "startTimestamp": result.start_timestamp,
        "endTimestamp": result.end_timestamp,
        "initialCapital": result.initial_capital,
        "finalEquity": result.final_equity,
        "metrics": result.metrics,
        "equityCurve": paginate(&result.equity_curve, p.equity_limit, p.equity_cursor.as_deref()),
        "trades": paginate(&result.trades, p.trades_limit, p.trades_cursor.as_deref()),
        "openPosition": result.open_position,
        "diagnostics": result.diagnostics,
    })
}

pub async fn run_backtest(p: RunBacktestParams) -> Result<CallToolResult, McpError> {
    let interval = parse_interval(p.interval.as_deref().unwrap_or("1d"));
    let range = parse_range(p.range.as_deref().unwrap_or("1y"));
    let strategy = build_strategy(&p)?;
    let config = build_config(&p)?;

    let ticker = Ticker::new(&p.symbol).await.map_err(lib_err)?;
    let result = ticker
        .backtest(strategy, interval, range, Some(config))
        .await
        .map_err(lib_err)?;

    let response = build_response(&result, &p);
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&response).map_err(ser_err)?,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_params(strategy: &str) -> RunBacktestParams {
        RunBacktestParams {
            symbol: "AAPL".to_string(),
            strategy: strategy.to_string(),
            interval: None,
            range: None,
            fast_period: None,
            slow_period: None,
            period: None,
            signal_period: None,
            std_dev: None,
            multiplier: None,
            oversold: None,
            overbought: None,
            exit_at_middle: None,
            initial_capital: None,
            commission_pct: None,
            slippage_pct: None,
            position_size_pct: None,
            allow_short: None,
            stop_loss_pct: None,
            take_profit_pct: None,
            equity_limit: None,
            equity_cursor: None,
            trades_limit: None,
            trades_cursor: None,
            max_leverage: None,
            maintenance_margin_pct: None,
            short_borrow_rate: None,
            margin_interest_rate: None,
        }
    }

    #[test]
    fn build_strategy_accepts_every_valid_name() {
        for name in [
            "sma_crossover",
            "rsi_reversal",
            "macd_signal",
            "bollinger_mean_reversion",
            "supertrend_follow",
            "donchian_breakout",
        ] {
            let p = base_params(name);
            assert!(build_strategy(&p).is_ok(), "strategy {name} should build");
        }
    }

    #[test]
    fn build_strategy_rejects_unknown_name() {
        let p = base_params("not_a_real_strategy");
        let err = build_strategy(&p);
        assert!(err.is_err());
    }

    #[test]
    fn build_strategy_uses_custom_periods() {
        let mut p = base_params("sma_crossover");
        p.fast_period = Some(5);
        p.slow_period = Some(15);
        let strategy = build_strategy(&p).unwrap();
        assert_eq!(strategy.required_indicators().len(), 2);
    }

    #[test]
    fn build_config_defaults_match_library_defaults() {
        let p = base_params("sma_crossover");
        let config = build_config(&p).unwrap();
        let default = BacktestConfig::default();
        assert_eq!(config.initial_capital, default.initial_capital);
        assert_eq!(config.commission_pct, default.commission_pct);
        assert_eq!(config.position_size_pct, default.position_size_pct);
        assert_eq!(config.max_leverage, default.max_leverage);
        assert_eq!(
            config.maintenance_margin_pct,
            default.maintenance_margin_pct
        );
        assert_eq!(config.short_borrow_rate, default.short_borrow_rate);
        assert_eq!(config.margin_interest_rate, default.margin_interest_rate);
        assert!(config.stop_loss_pct.is_none());
    }

    #[test]
    fn build_config_applies_overrides() {
        let mut p = base_params("sma_crossover");
        p.position_size_pct = Some(0.5);
        p.allow_short = Some(true);
        p.stop_loss_pct = Some(0.05);
        let config = build_config(&p).unwrap();
        assert_eq!(config.position_size_pct, 0.5);
        assert!(config.allow_short);
        assert_eq!(config.stop_loss_pct, Some(0.05));
    }

    #[test]
    fn build_config_applies_margin_overrides() {
        let mut p = base_params("sma_crossover");
        p.max_leverage = Some(2.0);
        p.maintenance_margin_pct = Some(0.3);
        p.short_borrow_rate = Some(0.05);
        p.margin_interest_rate = Some(0.07);
        let config = build_config(&p).unwrap();
        assert_eq!(config.max_leverage, 2.0);
        assert_eq!(config.maintenance_margin_pct, 0.3);
        assert_eq!(config.short_borrow_rate, 0.05);
        assert_eq!(config.margin_interest_rate, 0.07);
    }

    #[test]
    fn build_config_rejects_leverage_below_one() {
        let mut p = base_params("sma_crossover");
        p.max_leverage = Some(0.5);
        assert!(build_config(&p).is_err());
    }

    #[test]
    fn paginate_defaults_to_first_page_with_default_size() {
        let items: Vec<i32> = (0..30).collect();
        let page = paginate(&items, None, None);
        assert_eq!(page["items"].as_array().unwrap().len(), 25);
        assert_eq!(page["pageInfo"]["hasNextPage"], true);
        assert_eq!(page["pageInfo"]["endCursor"], "25");
    }

    #[test]
    fn paginate_honors_cursor_and_limit() {
        let items: Vec<i32> = (0..30).collect();
        let page = paginate(&items, Some(10), Some("25"));
        let got: Vec<i64> = page["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect();
        assert_eq!(got, vec![25, 26, 27, 28, 29]);
        assert_eq!(page["pageInfo"]["hasNextPage"], false);
        assert_eq!(page["pageInfo"]["endCursor"], serde_json::Value::Null);
    }
}

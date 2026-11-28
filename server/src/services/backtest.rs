use finance_query::backtesting::{
    BacktestConfig, BollingerMeanReversion, DonchianBreakout, MacdSignal, RsiReversal,
    SmaCrossover, Strategy, SuperTrendFollow,
};
use finance_query::{Interval, Ticker};

use super::{ServiceError, ServiceResult};
use crate::graphql::types::backtest::{GqlBacktestParams, GqlStrategy};
use crate::graphql::types::enums::{GqlInterval, GqlTimeRange};

fn build_strategy(strategy: GqlStrategy, p: &GqlBacktestParams) -> Box<dyn Strategy> {
    match strategy {
        GqlStrategy::SmaCrossover => Box::new(SmaCrossover::new(
            p.fast_period.unwrap_or(10) as usize,
            p.slow_period.unwrap_or(20) as usize,
        )),
        GqlStrategy::RsiReversal => {
            let mut s = RsiReversal::new(p.period.unwrap_or(14) as usize);
            if p.oversold.is_some() || p.overbought.is_some() {
                s = s.with_thresholds(p.oversold.unwrap_or(30.0), p.overbought.unwrap_or(70.0));
            }
            Box::new(s)
        }
        GqlStrategy::MacdSignal => Box::new(MacdSignal::new(
            p.fast_period.unwrap_or(12) as usize,
            p.slow_period.unwrap_or(26) as usize,
            p.signal_period.unwrap_or(9) as usize,
        )),
        GqlStrategy::BollingerMeanReversion => {
            let mut s = BollingerMeanReversion::new(
                p.period.unwrap_or(20) as usize,
                p.std_dev.unwrap_or(2.0),
            );
            if let Some(exit) = p.exit_at_middle {
                s = s.exit_at_middle(exit);
            }
            Box::new(s)
        }
        GqlStrategy::SupertrendFollow => Box::new(SuperTrendFollow::new(
            p.period.unwrap_or(10) as usize,
            p.multiplier.unwrap_or(3.0),
        )),
        GqlStrategy::DonchianBreakout => {
            let mut s = DonchianBreakout::new(p.period.unwrap_or(20) as usize);
            if let Some(exit) = p.exit_at_middle {
                s = s.exit_at_middle(exit);
            }
            Box::new(s)
        }
    }
}

/// Every knob falls back to the library default, except `bars_per_year`, which
/// follows the interval so annualized metrics and financing costs match the bar
/// size rather than assuming daily bars.
fn build_config(p: &GqlBacktestParams, interval: Interval) -> Result<BacktestConfig, ServiceError> {
    let mut builder = BacktestConfig::builder().bars_per_year(interval.bars_per_year());
    if let Some(v) = p.initial_capital {
        builder = builder.initial_capital(v);
    }
    if let Some(v) = p.commission_pct {
        builder = builder.commission_pct(v);
    }
    if let Some(v) = p.slippage_pct {
        builder = builder.slippage_pct(v);
    }
    if let Some(v) = p.position_size_pct {
        builder = builder.position_size_pct(v);
    }
    if let Some(v) = p.allow_short {
        builder = builder.allow_short(v);
    }
    if let Some(v) = p.stop_loss_pct {
        builder = builder.stop_loss_pct(v);
    }
    if let Some(v) = p.take_profit_pct {
        builder = builder.take_profit_pct(v);
    }
    if let Some(v) = p.max_leverage {
        builder = builder.max_leverage(v);
    }
    if let Some(v) = p.maintenance_margin_pct {
        builder = builder.maintenance_margin_pct(v);
    }
    if let Some(v) = p.short_borrow_rate {
        builder = builder.short_borrow_rate(v);
    }
    if let Some(v) = p.margin_interest_rate {
        builder = builder.margin_interest_rate(v);
    }
    builder.build().map_err(|e| Box::new(e) as ServiceError)
}

pub async fn run_backtest(
    cache: &crate::cache::Cache,
    symbol: &str,
    strategy: GqlStrategy,
    interval: GqlInterval,
    range: GqlTimeRange,
    params: GqlBacktestParams,
) -> ServiceResult {
    let (interval_str, range_str) = (interval.as_str(), range.as_str());
    let (interval, range) = (interval.into(), range.into());
    // A run is fully determined by its inputs, so the knobs belong in the key.
    let knobs = serde_json::to_string(&params).unwrap_or_default();
    let cache_key = crate::cache::Cache::key(
        "backtest",
        &[
            &symbol.to_uppercase(),
            strategy.as_str(),
            interval_str,
            range_str,
            &knobs,
        ],
    );
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            crate::cache::ttl::HISTORICAL,
            crate::cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let result = ticker
                    .backtest(
                        build_strategy(strategy, &params),
                        interval,
                        range,
                        Some(build_config(&params, interval)?),
                    )
                    .await?;
                serde_json::to_value(&result).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn indicators(strategy: GqlStrategy, params: &GqlBacktestParams) -> Vec<String> {
        build_strategy(strategy, params)
            .required_indicators()
            .into_iter()
            .map(|(_, indicator)| format!("{indicator:?}"))
            .collect()
    }

    #[test]
    fn each_strategy_variant_builds_its_own_strategy() {
        let params = GqlBacktestParams::default();
        let names: Vec<String> = [
            GqlStrategy::SmaCrossover,
            GqlStrategy::RsiReversal,
            GqlStrategy::MacdSignal,
            GqlStrategy::BollingerMeanReversion,
            GqlStrategy::SupertrendFollow,
            GqlStrategy::DonchianBreakout,
        ]
        .into_iter()
        .map(|s| build_strategy(s, &params).name().to_string())
        .collect();
        assert_eq!(
            names,
            vec![
                "SMA Crossover",
                "RSI Reversal",
                "MACD Signal",
                "Bollinger Mean Reversion",
                "SuperTrend Follow",
                "Donchian Breakout",
            ]
        );
    }

    #[test]
    fn strategy_defaults_follow_the_library_periods() {
        let params = GqlBacktestParams::default();
        assert_eq!(
            indicators(GqlStrategy::SmaCrossover, &params),
            vec!["Sma(10)", "Sma(20)"]
        );
        assert_eq!(
            indicators(GqlStrategy::RsiReversal, &params),
            vec!["Rsi(14)"]
        );
        assert_eq!(
            indicators(GqlStrategy::MacdSignal, &params),
            vec!["Macd { fast: 12, slow: 26, signal: 9 }"]
        );
        assert_eq!(
            indicators(GqlStrategy::BollingerMeanReversion, &params),
            vec!["Bollinger { period: 20, std_dev: 2.0 }"]
        );
        assert_eq!(
            indicators(GqlStrategy::SupertrendFollow, &params),
            vec!["Supertrend { period: 10, multiplier: 3.0 }"]
        );
    }

    #[test]
    fn strategy_params_override_defaults() {
        let params = GqlBacktestParams {
            fast_period: Some(5),
            slow_period: Some(30),
            period: Some(7),
            ..Default::default()
        };
        assert_eq!(
            indicators(GqlStrategy::SmaCrossover, &params),
            vec!["Sma(5)", "Sma(30)"]
        );
        assert_eq!(
            indicators(GqlStrategy::RsiReversal, &params),
            vec!["Rsi(7)"]
        );
    }

    #[test]
    fn bars_per_year_follows_the_interval() {
        let params = GqlBacktestParams::default();
        assert_eq!(
            build_config(&params, Interval::OneDay)
                .unwrap()
                .bars_per_year,
            252.0
        );
        assert_eq!(
            build_config(&params, Interval::OneHour)
                .unwrap()
                .bars_per_year,
            1_638.0
        );
    }

    #[test]
    fn config_knobs_are_applied() {
        let params = GqlBacktestParams {
            initial_capital: Some(50_000.0),
            commission_pct: Some(0.002),
            allow_short: Some(true),
            stop_loss_pct: Some(0.05),
            ..Default::default()
        };
        let config = build_config(&params, Interval::OneDay).unwrap();
        assert_eq!(config.initial_capital, 50_000.0);
        assert_eq!(config.commission_pct, 0.002);
        assert!(config.allow_short);
        assert_eq!(config.stop_loss_pct, Some(0.05));
    }

    #[test]
    fn invalid_knob_surfaces_as_a_service_error() {
        let params = GqlBacktestParams {
            initial_capital: Some(-1.0),
            ..Default::default()
        };
        let err = build_config(&params, Interval::OneDay).unwrap_err();
        assert!(err.to_string().contains("initial_capital"), "{err}");
    }
}

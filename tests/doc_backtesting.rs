//! Compile and runtime tests for docs/library/backtesting.md
//!
//! Requires the `backtesting` feature flag:
//!   cargo test --test doc_backtesting --features backtesting
//!   cargo test --test doc_backtesting --features backtesting -- --ignored   (network tests)

#![cfg(feature = "backtesting")]

use finance_query::backtesting::condition::*;
use finance_query::backtesting::refs::*;
use finance_query::backtesting::{
    BacktestComparison, BacktestConfig, BayesianSearch, BollingerMeanReversion, DonchianBreakout,
    EnsembleMode, EnsembleStrategy, GridSearch, MacdSignal, MonteCarloConfig, MonteCarloMethod,
    OptimizeMetric, ParamRange, RsiReversal, Signal, SmaCrossover, StrategyBuilder,
    SuperTrendFollow, WalkForwardConfig,
};

// ---------------------------------------------------------------------------
// Compile-time — BacktestConfig builder
// ---------------------------------------------------------------------------

#[test]
fn test_backtest_config_full_builder() {
    // From backtesting.md "Configuration" section
    let config = BacktestConfig::builder()
        .initial_capital(50_000.0)
        .commission_pct(0.001)
        .commission(1.0)
        .slippage_pct(0.0005)
        .spread_pct(0.0002)
        .transaction_tax_pct(0.005)
        .stop_loss_pct(0.05)
        .take_profit_pct(0.15)
        .trailing_stop_pct(0.03)
        .allow_short(true)
        .position_size_pct(0.5)
        .max_positions(3)
        .bars_per_year(252.0)
        .risk_free_rate(0.04)
        .reinvest_dividends(true)
        .close_at_end(true)
        .build()
        .unwrap();
    let _ = config;
}

#[test]
fn test_backtest_config_zero_cost() {
    // From backtesting.md "Zero-Cost Config" section
    let config = BacktestConfig::zero_cost();
    let _ = config;
}

#[test]
fn test_backtest_config_commission_fn() {
    // From backtesting.md "Custom Commission Function" section
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_fn(|size, price| {
            let value = size * price;
            if value < 1_000.0 { 1.0 } else { value * 0.0005 }
        })
        .build()
        .unwrap();
    let _ = config;
}

#[test]
fn test_backtest_config_defaults() {
    // From backtesting.md "Trading Modes" section
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .slippage_pct(0.0005)
        .allow_short(false)
        .build()
        .unwrap();
    let _ = config;
}

#[test]
fn test_backtest_config_no_costs() {
    // From backtesting.md "Best Practices" — less robust example
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .build()
        .unwrap();
    let _ = config;
}

// ---------------------------------------------------------------------------
// Compile-time — StrategyBuilder variations
// ---------------------------------------------------------------------------

#[test]
fn test_strategy_builder_rsi_mean_reversion() {
    // From backtesting.md "Custom Strategies" section
    let strategy = StrategyBuilder::new("RSI Mean Reversion")
        .entry(rsi(14).crosses_below(30.0).and(price().above_ref(sma(200))))
        .exit(rsi(14).crosses_above(70.0).or(stop_loss(0.05)))
        .build();
    let _ = strategy;
}

#[test]
fn test_strategy_builder_regime_filter() {
    // From backtesting.md "Regime Filter" section
    let strategy = StrategyBuilder::new("Trend-Filtered RSI")
        .entry(rsi(14).crosses_below(30.0))
        .exit(rsi(14).crosses_above(70.0))
        .regime_filter(price().above_ref(sma(200)))
        .build();
    let _ = strategy;
}

#[test]
fn test_strategy_builder_with_short() {
    // From backtesting.md "Separate Short Leg" section
    let strategy = StrategyBuilder::new("Long-Short RSI")
        .entry(rsi(14).crosses_below(30.0))
        .exit(rsi(14).crosses_above(70.0))
        .with_short(rsi(14).crosses_above(70.0), rsi(14).crosses_below(30.0))
        .build();
    let _ = strategy;
}

#[test]
fn test_strategy_builder_warmup() {
    // From backtesting.md "Warmup Period" section
    let strategy = StrategyBuilder::new("SMA with Warmup")
        .entry(price().crosses_above_ref(sma(200)))
        .exit(price().crosses_below_ref(sma(200)))
        .warmup(200)
        .build();
    let _ = strategy;
}

#[test]
fn test_strategy_builder_validated() {
    // From backtesting.md "Best Practices" section
    let strategy = StrategyBuilder::new("Validated Strategy")
        .entry(
            rsi(14)
                .crosses_below(30.0)
                .and(price().above_ref(sma(200)))
                .and(volume().above_ref(sma(20))),
        )
        .exit(
            rsi(14)
                .crosses_above(70.0)
                .or(stop_loss(0.05))
                .or(take_profit(0.15)),
        )
        .build();
    let _ = strategy;
}

// ---------------------------------------------------------------------------
// Compile-time — Signal / OrderType
// ---------------------------------------------------------------------------

#[test]
fn test_signal_order_types() {
    // From backtesting.md "Order Types" section
    let ts = 0i64;
    let px = 150.0;

    let _market_entry = Signal::long(ts, px);
    let _limit_entry = Signal::buy_limit(ts, px, 148.0);
    let _stop_entry = Signal::buy_stop(ts, px, 152.0);
    let _stop_limit = Signal::buy_stop_limit(ts, px, 152.0, 153.0);
    let _limit_exit = Signal::sell_limit(ts, px, 160.0);
    let _stop_exit = Signal::sell_stop(ts, px, 145.0);
}

#[test]
fn test_signal_expiry() {
    // From backtesting.md "Order Expiry" section
    let signal = Signal::buy_limit(0, 150.0, 148.0).expires_in_bars(5);
    let _ = signal;
}

#[test]
fn test_signal_per_trade_bracket() {
    // From backtesting.md "Per-Trade Bracket Orders" section
    let signal = Signal::long(0, 150.0)
        .stop_loss(0.03)
        .take_profit(0.10)
        .trailing_stop(0.02);
    let _ = signal;
}

#[test]
fn test_signal_scale() {
    // From backtesting.md "Scale In / Out" section
    let add_to_position = Signal::scale_in(0.25, 0, 150.0);
    let reduce_position = Signal::scale_out(0.50, 0, 150.0);
    let _ = (add_to_position, reduce_position);
}

#[test]
fn test_signal_tags() {
    // From backtesting.md "Signal Tags" section
    let signal = Signal::long(0, 150.0).tag("breakout").tag("high-volume");
    let _ = signal;
}

// ---------------------------------------------------------------------------
// Compile-time — EnsembleStrategy
// ---------------------------------------------------------------------------

#[test]
fn test_ensemble_strategy_build() {
    // From backtesting.md "Ensemble Strategy" section (compile only)
    let ensemble = EnsembleStrategy::new("Ensemble")
        .add(SmaCrossover::new(10, 50), 0.6)
        .add(RsiReversal::new(14), 0.4)
        .mode(EnsembleMode::WeightedMajority)
        .build();
    let _ = ensemble;
}

#[test]
fn test_ensemble_modes() {
    // Verify all EnsembleMode variants compile
    let _a = EnsembleMode::WeightedMajority;
    let _b = EnsembleMode::Unanimous;
    let _c = EnsembleMode::AnySignal;
    let _d = EnsembleMode::StrongestSignal;
}

// ---------------------------------------------------------------------------
// Compile-time — HTF conditions
// ---------------------------------------------------------------------------

#[test]
fn test_htf_condition_build() {
    // From backtesting.md "Higher-Timeframe Conditions" section
    use finance_query::Interval;
    use finance_query::backtesting::refs::htf;

    let strategy = StrategyBuilder::new("HTF RSI Filter")
        .entry(
            rsi(14)
                .crosses_below(30.0)
                .and(htf(Interval::OneDay, rsi(14).above(40.0))),
        )
        .exit(rsi(14).crosses_above(70.0))
        .build();
    let _ = strategy;
}

// ---------------------------------------------------------------------------
// Compile-time — MonteCarloMethod variants
// ---------------------------------------------------------------------------

#[test]
fn test_monte_carlo_methods_compile() {
    // From backtesting.md "Monte Carlo Simulation" section
    let _a = MonteCarloMethod::IidShuffle;
    let _b = MonteCarloMethod::BlockBootstrap { block_size: 10 };
    let _c = MonteCarloMethod::StationaryBootstrap {
        mean_block_size: 10,
    };
    let _d = MonteCarloMethod::Parametric;
}

#[test]
fn test_monte_carlo_config_build() {
    let mc_config = MonteCarloConfig::new()
        .seed(42)
        .num_simulations(1_000)
        .method(MonteCarloMethod::IidShuffle);
    let _ = mc_config;
}

// ---------------------------------------------------------------------------
// Compile-time — OptimizeMetric variants
// ---------------------------------------------------------------------------

#[test]
fn test_optimize_metric_variants() {
    let _a = OptimizeMetric::TotalReturn;
    let _b = OptimizeMetric::SharpeRatio;
    let _c = OptimizeMetric::SortinoRatio;
    let _d = OptimizeMetric::CalmarRatio;
    let _e = OptimizeMetric::ProfitFactor;
    let _f = OptimizeMetric::WinRate;
    let _g = OptimizeMetric::MinDrawdown;
}

// ---------------------------------------------------------------------------
// Compile-time — ParamRange constructors
// ---------------------------------------------------------------------------

#[test]
fn test_param_range_constructors() {
    let _a = ParamRange::int_range(5, 20, 5);
    let _b = ParamRange::float_range(0.1, 2.0, 0.1);
    let _c = ParamRange::int_bounds(5, 50);
    let _d = ParamRange::float_bounds(0.1, 3.0);
}

// ---------------------------------------------------------------------------
// Compile-time — BacktestComparison
// ---------------------------------------------------------------------------

#[test]
fn test_backtest_comparison_build() {
    // Verify BacktestComparison API compiles (ranking requires real results, tested in network tests)
    let _ = BacktestComparison::new;
    let _ = OptimizeMetric::SharpeRatio;
}

// ---------------------------------------------------------------------------
// Network tests — pre-built strategies
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_sma_crossover() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "SMA Crossover" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            SmaCrossover::new(10, 20),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();

    println!("Total Return: {:.2}%", result.metrics.total_return_pct);
    println!("Sharpe Ratio: {:.2}", result.metrics.sharpe_ratio);
    println!(
        "Max Drawdown: {:.2}%",
        result.metrics.max_drawdown_pct * 100.0
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_rsi_reversal() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "RSI Mean Reversion" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    let result = ticker
        .backtest(
            RsiReversal::new(14),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let _ = result;

    // With custom thresholds
    let result = ticker
        .backtest(
            RsiReversal::new(14).with_thresholds(30.0, 70.0),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_macd_signal() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "MACD Signal Crossover" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            MacdSignal::new(12, 26, 9),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_bollinger_mean_reversion() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Bollinger Band Mean Reversion" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            BollingerMeanReversion::new(20, 2.0),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_supertrend_follow() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "SuperTrend Trend Following" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            SuperTrendFollow::new(10, 3.0),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_donchian_breakout() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Donchian Breakout" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            DonchianBreakout::new(20),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let _ = result;
}

// ---------------------------------------------------------------------------
// Network tests — custom strategies
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_custom_strategy_basic() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Custom Strategies" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    let strategy = StrategyBuilder::new("RSI Mean Reversion")
        .entry(rsi(14).crosses_below(30.0).and(price().above_ref(sma(200))))
        .exit(rsi(14).crosses_above(70.0).or(stop_loss(0.05)))
        .build();

    let result = ticker
        .backtest(strategy, Interval::OneDay, TimeRange::OneYear, None)
        .await
        .unwrap();
    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_custom_strategy_regime_filter() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Regime Filter" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    let strategy = StrategyBuilder::new("Trend-Filtered RSI")
        .entry(rsi(14).crosses_below(30.0))
        .exit(rsi(14).crosses_above(70.0))
        .regime_filter(price().above_ref(sma(200)))
        .build();

    let result = ticker
        .backtest(strategy, Interval::OneDay, TimeRange::OneYear, None)
        .await
        .unwrap();
    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_custom_strategy_with_short() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Separate Short Leg" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    let strategy = StrategyBuilder::new("Long-Short RSI")
        .entry(rsi(14).crosses_below(30.0))
        .exit(rsi(14).crosses_above(70.0))
        .with_short(rsi(14).crosses_above(70.0), rsi(14).crosses_below(30.0))
        .build();

    let config = BacktestConfig::builder().allow_short(true).build().unwrap();

    let result = ticker
        .backtest(strategy, Interval::OneDay, TimeRange::OneYear, Some(config))
        .await
        .unwrap();
    let _ = result;
}

// ---------------------------------------------------------------------------
// Network tests — performance metrics
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_performance_metrics_access() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Performance Metrics" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            SmaCrossover::new(10, 20),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();

    // Returns
    println!("Total Return: {:.2}%", result.metrics.total_return_pct);
    println!(
        "Annualized Return: {:.2}%",
        result.metrics.annualized_return_pct
    );
    println!("Final Equity: ${:.2}", result.final_equity);

    // Risk-adjusted
    println!("Sharpe Ratio: {:.2}", result.metrics.sharpe_ratio);
    println!("Sortino Ratio: {:.2}", result.metrics.sortino_ratio);
    println!("Calmar Ratio: {:.2}", result.metrics.calmar_ratio);
    println!(
        "Max Drawdown: {:.2}%",
        result.metrics.max_drawdown_pct * 100.0
    );

    // Trade statistics
    println!("Total Trades: {}", result.metrics.total_trades);
    println!("Winning Trades: {}", result.metrics.winning_trades);
    println!("Losing Trades: {}", result.metrics.losing_trades);
    println!("Win Rate: {:.2}%", result.metrics.win_rate * 100.0);
    println!("Profit Factor: {:.2}", result.metrics.profit_factor);
    println!("Avg Trade: {:.2}%", result.metrics.avg_trade_return_pct);
    println!("Avg Win: {:.2}%", result.metrics.avg_win_pct);
    println!("Avg Loss: {:.2}%", result.metrics.avg_loss_pct);
    println!("Largest Win: {:.2}%", result.metrics.largest_win);
    println!("Largest Loss: {:.2}%", result.metrics.largest_loss);
    println!("Max Consec. Wins: {}", result.metrics.max_consecutive_wins);
    println!(
        "Max Consec. Losses: {}",
        result.metrics.max_consecutive_losses
    );

    // Position breakdown
    println!("Long Trades: {}", result.metrics.long_trades);
    println!("Short Trades: {}", result.metrics.short_trades);
    println!(
        "Time in Market: {:.1}%",
        result.metrics.time_in_market_pct * 100.0
    );

    // Signal execution
    println!("Total Signals: {}", result.metrics.total_signals);
    println!("Executed Signals: {}", result.metrics.executed_signals);
    println!("Total Commission: ${:.2}", result.metrics.total_commission);
    println!(
        "Dividend Income: ${:.2}",
        result.metrics.total_dividend_income
    );

    // Advanced statistics
    println!("Kelly Criterion: {:.2}", result.metrics.kelly_criterion);
    println!("SQN: {:.2}", result.metrics.sqn);
    println!("Expectancy: {:.2}", result.metrics.expectancy);
    println!("Omega Ratio: {:.2}", result.metrics.omega_ratio);
    println!("Tail Ratio: {:.2}", result.metrics.tail_ratio);
    println!("Recovery Factor: {:.2}", result.metrics.recovery_factor);
    println!("Ulcer Index: {:.2}", result.metrics.ulcer_index);
    println!("Serenity Ratio: {:.2}", result.metrics.serenity_ratio);
}

// ---------------------------------------------------------------------------
// Network tests — advanced result analysis
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_rolling_analytics() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Rolling Analytics" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            SmaCrossover::new(10, 20),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();

    let sharpe_30 = result.rolling_sharpe(30);
    let drawdowns = result.drawdown_series();
    let win_rate_20 = result.rolling_win_rate(20);
    let _ = (sharpe_30, drawdowns, win_rate_20);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_temporal_breakdown() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Temporal Breakdown" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            SmaCrossover::new(10, 20),
            Interval::OneDay,
            TimeRange::TwoYears,
            None,
        )
        .await
        .unwrap();

    let by_year = result.by_year();
    let by_month = result.by_month();
    let by_dow = result.by_day_of_week();

    for (year, metrics) in &by_year {
        println!("{year}: {:.2}%", metrics.total_return_pct);
    }
    let _ = (by_month, by_dow);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_tag_based_filtering() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Tag-Based Filtering" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            SmaCrossover::new(10, 20),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();

    let _tagged_trades = result.trades_by_tag("breakout");
    let _tagged_metrics = result.metrics_by_tag("breakout");
    let _all_tags = result.all_tags();
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_diagnostics() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Diagnostics" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            SmaCrossover::new(10, 20),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();

    for msg in &result.diagnostics {
        println!("⚠ {msg}");
    }
}

// ---------------------------------------------------------------------------
// Network tests — ensemble strategy
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ensemble_strategy_run() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Ensemble Strategy" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    let ensemble = EnsembleStrategy::new("Ensemble")
        .add(SmaCrossover::new(10, 50), 0.6)
        .add(RsiReversal::new(14), 0.4)
        .mode(EnsembleMode::WeightedMajority)
        .build();

    let result = ticker
        .backtest(ensemble, Interval::OneDay, TimeRange::OneYear, None)
        .await
        .unwrap();
    let _ = result;
}

// ---------------------------------------------------------------------------
// Network tests — benchmark comparison
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_backtest_with_benchmark() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Benchmark Comparison" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest_with_benchmark(
            SmaCrossover::new(10, 50),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
            "SPY",
        )
        .await
        .unwrap();

    if let Some(bench) = &result.benchmark {
        println!("Strategy return:   {:.2}%", result.metrics.total_return_pct);
        println!("Benchmark return:  {:.2}%", bench.benchmark_return_pct);
        println!("Buy & hold return: {:.2}%", bench.buy_and_hold_return_pct);
        println!("Alpha: {:.4}", bench.alpha);
        println!("Beta:  {:.4}", bench.beta);
        println!("Information Ratio: {:.4}", bench.information_ratio);
    }
}

// ---------------------------------------------------------------------------
// Network tests — strategy comparison
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_backtest_comparison() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Strategy Comparison" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    let result_sma = ticker
        .backtest(
            SmaCrossover::new(10, 20),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let result_rsi = ticker
        .backtest(
            RsiReversal::new(14),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let result_macd = ticker
        .backtest(
            MacdSignal::new(12, 26, 9),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();

    let report = BacktestComparison::new()
        .add("SMA Crossover", result_sma)
        .add("RSI Reversal", result_rsi)
        .add("MACD Signal", result_macd)
        .ranked_by(OptimizeMetric::SharpeRatio);

    println!("Winner: {}", report.winner());

    for row in report.table() {
        println!(
            "#{} {} — Sharpe {:.2}, Return {:.2}%",
            row.rank, row.label, row.sharpe_ratio, row.total_return_pct,
        );
    }
}

// ---------------------------------------------------------------------------
// Network tests — parameter optimization
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_grid_search() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Grid Search" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneYear)
        .await
        .unwrap();
    let candles = chart.candles.clone();

    let config = BacktestConfig::zero_cost();

    let report = GridSearch::new()
        .param("fast", ParamRange::int_range(5, 20, 5))
        .param("slow", ParamRange::int_range(20, 60, 10))
        .optimize_for(OptimizeMetric::SharpeRatio)
        .run("AAPL", &candles, &config, |params| {
            SmaCrossover::new(
                params["fast"].as_int() as usize,
                params["slow"].as_int() as usize,
            )
        })
        .unwrap();

    println!(
        "Best Sharpe: {:.2}",
        report.best.result.metrics.sharpe_ratio
    );
    println!(
        "Best params: fast={}, slow={}",
        report.best.params["fast"].as_int(),
        report.best.params["slow"].as_int(),
    );
    println!("Evaluated {} combinations", report.n_evaluations);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_bayesian_search() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Bayesian Search" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneYear)
        .await
        .unwrap();
    let candles = chart.candles.clone();

    let config = BacktestConfig::zero_cost();

    let report = BayesianSearch::new()
        .param("fast", ParamRange::int_bounds(5, 50))
        .param("slow", ParamRange::int_bounds(20, 200))
        .max_evaluations(100)
        .initial_points(10)
        .ucb_beta(2.0)
        .seed(42)
        .optimize_for(OptimizeMetric::SharpeRatio)
        .run("AAPL", &candles, &config, |params| {
            SmaCrossover::new(
                params["fast"].as_int() as usize,
                params["slow"].as_int() as usize,
            )
        })
        .unwrap();

    println!("Convergence: {:?}", report.convergence_curve);
    let _ = report;
}

// ---------------------------------------------------------------------------
// Network tests — walk-forward validation
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_walk_forward() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Walk-Forward Validation" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::FiveYears)
        .await
        .unwrap();
    let candles = chart.candles.clone();

    let grid = GridSearch::new()
        .param("fast", ParamRange::int_range(5, 20, 5))
        .param("slow", ParamRange::int_range(20, 60, 10))
        .optimize_for(OptimizeMetric::SharpeRatio);

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .build()
        .unwrap();

    let report = WalkForwardConfig::new(grid, config)
        .in_sample_bars(252)
        .out_of_sample_bars(63)
        .run("AAPL", &candles, |params| {
            SmaCrossover::new(
                params["fast"].as_int() as usize,
                params["slow"].as_int() as usize,
            )
        })
        .unwrap();

    println!(
        "OOS Return:     {:.2}%",
        report.aggregate_metrics.total_return_pct
    );
    println!("Consistency:    {:.1}%", report.consistency_ratio * 100.0);
    println!("Windows tested: {}", report.windows.len());

    for w in &report.windows {
        println!(
            "Window {}: IS {:.1}% → OOS {:.1}%",
            w.window,
            w.in_sample.metrics.total_return_pct,
            w.out_of_sample.metrics.total_return_pct,
        );
    }
}

// ---------------------------------------------------------------------------
// Network tests — Monte Carlo simulation
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_monte_carlo() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Monte Carlo Simulation" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            SmaCrossover::new(10, 20),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();

    let mc = MonteCarloConfig::new()
        .seed(42)
        .num_simulations(1_000)
        .method(MonteCarloMethod::IidShuffle)
        .run(&result);

    println!("Return p5:    {:.2}%", mc.total_return.p5);
    println!("Return p50:   {:.2}%", mc.total_return.p50);
    println!("Return p95:   {:.2}%", mc.total_return.p95);
    println!("Drawdown p95: {:.2}%", mc.max_drawdown.p95);
    println!("Sharpe p50:   {:.2}", mc.sharpe_ratio.p50);
}

// ---------------------------------------------------------------------------
// Network tests — portfolio backtesting
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_portfolio_engine() {
    use finance_query::backtesting::portfolio::{
        PortfolioConfig, PortfolioEngine, RebalanceMode, SymbolData,
    };
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Portfolio Backtesting" section
    let aapl = Ticker::new("AAPL").await.unwrap();
    let msft = Ticker::new("MSFT").await.unwrap();

    let aapl_candles = aapl
        .chart(Interval::OneDay, TimeRange::OneYear)
        .await
        .unwrap()
        .candles;
    let msft_candles = msft
        .chart(Interval::OneDay, TimeRange::OneYear)
        .await
        .unwrap()
        .candles;

    let config = PortfolioConfig::new(
        BacktestConfig::builder()
            .initial_capital(50_000.0)
            .commission_pct(0.001)
            .build()
            .unwrap(),
    )
    .max_total_positions(3)
    .rebalance(RebalanceMode::EqualWeight);

    let symbol_data = vec![
        SymbolData::new("AAPL", aapl_candles),
        SymbolData::new("MSFT", msft_candles),
    ];

    let result = PortfolioEngine::new(config)
        .run(&symbol_data, |_sym| SmaCrossover::new(10, 50))
        .unwrap();

    println!(
        "Portfolio Return: {:.2}%",
        result.portfolio_metrics.total_return_pct
    );
    println!("Final Equity:     ${:.2}", result.final_equity);

    for (sym, sym_result) in &result.symbols {
        println!("{}: {:.2}%", sym, sym_result.metrics.total_return_pct);
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_tickers_portfolio_backtest() {
    use finance_query::backtesting::portfolio::{PortfolioConfig, RebalanceMode};
    use finance_query::{Interval, Tickers, TimeRange};

    // From backtesting.md "Via Tickers::backtest()" section
    let tickers = Tickers::new(vec!["AAPL", "MSFT", "GOOGL"]).await.unwrap();

    let config = PortfolioConfig::new(BacktestConfig::default())
        .max_total_positions(3)
        .rebalance(RebalanceMode::EqualWeight);

    let result = tickers
        .backtest(Interval::OneDay, TimeRange::OneYear, Some(config), |_sym| {
            SmaCrossover::new(10, 50)
        })
        .await
        .unwrap();

    let _ = result;
}

// ---------------------------------------------------------------------------
// Network tests — complete strategy example
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_complete_strategy_example() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Example: Complete Strategy" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    let strategy = StrategyBuilder::new("Momentum with Risk Management")
        .entry({
            let m = macd(12, 26, 9);
            m.line()
                .crosses_above_ref(m.signal_line())
                .and(price().above_ref(ema(50)))
                .and(volume().above_ref(sma(20)))
        })
        .exit({
            let m = macd(12, 26, 9);
            m.line()
                .crosses_below_ref(m.signal_line())
                .or(stop_loss(0.08))
                .or(take_profit(0.15))
        })
        .regime_filter(price().above_ref(sma(200)))
        .warmup(200)
        .build();

    let config = BacktestConfig::builder()
        .initial_capital(100_000.0)
        .commission_pct(0.001)
        .slippage_pct(0.0005)
        .allow_short(false)
        .build()
        .unwrap();

    let result = ticker
        .backtest(
            strategy,
            Interval::OneDay,
            TimeRange::TwoYears,
            Some(config),
        )
        .await
        .unwrap();

    println!("Backtest Results for AAPL");
    println!("=========================");
    println!("Total Return: {:.2}%", result.metrics.total_return_pct);
    println!("Sharpe Ratio: {:.2}", result.metrics.sharpe_ratio);
    println!("Win Rate:     {:.2}%", result.metrics.win_rate * 100.0);
    println!("Total Trades: {}", result.metrics.total_trades);
    println!(
        "Max Drawdown: {:.2}%",
        result.metrics.max_drawdown_pct * 100.0
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_best_practices_validated_strategy() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Best Practices" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .slippage_pct(0.0005)
        .allow_short(false)
        .build()
        .unwrap();

    let strategy = StrategyBuilder::new("Validated Strategy")
        .entry(
            rsi(14)
                .crosses_below(30.0)
                .and(price().above_ref(sma(200)))
                .and(volume().above_ref(sma(20))),
        )
        .exit(
            rsi(14)
                .crosses_above(70.0)
                .or(stop_loss(0.05))
                .or(take_profit(0.15)),
        )
        .build();

    let result = ticker
        .backtest(strategy, Interval::OneDay, TimeRange::OneYear, Some(config))
        .await
        .unwrap();

    println!("1Y Return: {:.2}%", result.metrics.total_return_pct);
}

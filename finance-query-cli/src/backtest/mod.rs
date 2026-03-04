mod dynamic_strategy;
mod indicators;
mod input;
mod presets;
mod render;
mod results;
mod state;
mod types;
mod user_presets;

// Re-exports for public API (these are used by other parts of the crate)
#[allow(unused_imports)]
pub use indicators::{IndicatorCategory, IndicatorDef, ParamDef};
pub use presets::StrategyPreset;
pub use results::{ResultsAction, RunResult, run_results_tui};
pub use state::{App, Screen};
use types::bars_per_year_for_interval;
pub use types::{BacktestConfiguration, BuiltCondition, CompareTarget};

use crate::error::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use finance_query::Ticker;
use finance_query::backtesting::ParamValue;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::collections::HashMap;
use std::io;

use input::handle_input;
use render::ui;

use types::{OptimizerParamDef, StrategyConfig};

/// Rebuild a `DynamicStrategy` from a base strategy config by substituting
/// parameter values from an optimizer result.
const GROUP_ENTRY: usize = 0;
const GROUP_EXIT: usize = 1;
const GROUP_SHORT_ENTRY: usize = 2;
const GROUP_SHORT_EXIT: usize = 3;

fn build_strategy_from_params(
    strategy_cfg: &StrategyConfig,
    enabled_params: &[OptimizerParamDef],
    params: &HashMap<String, ParamValue>,
) -> dynamic_strategy::DynamicStrategy {
    let mut sc = strategy_cfg.clone();
    for p in enabled_params {
        let val = params.get(&p.name).map(|v| v.as_float()).unwrap_or(p.start);
        match p.group {
            GROUP_ENTRY => {
                if let Some(cond) = sc.entry_conditions.conditions.get_mut(p.condition_idx)
                    && let Some(pv) = cond.indicator.param_values.get_mut(p.param_idx)
                {
                    *pv = val;
                }
            }
            GROUP_EXIT => {
                if let Some(cond) = sc.exit_conditions.conditions.get_mut(p.condition_idx)
                    && let Some(pv) = cond.indicator.param_values.get_mut(p.param_idx)
                {
                    *pv = val;
                }
            }
            GROUP_SHORT_ENTRY => {
                if let Some(group) = sc.short_entry_conditions.as_mut()
                    && let Some(cond) = group.conditions.get_mut(p.condition_idx)
                    && let Some(pv) = cond.indicator.param_values.get_mut(p.param_idx)
                {
                    *pv = val;
                }
            }
            GROUP_SHORT_EXIT => {
                if let Some(group) = sc.short_exit_conditions.as_mut()
                    && let Some(cond) = group.conditions.get_mut(p.condition_idx)
                    && let Some(pv) = cond.indicator.param_values.get_mut(p.param_idx)
                {
                    *pv = val;
                }
            }
            _ => {}
        }
    }
    let mut strategy = dynamic_strategy::DynamicStrategy::new(
        sc.name.clone(),
        sc.entry_conditions,
        sc.exit_conditions,
    );
    strategy.short_entry = sc.short_entry_conditions;
    strategy.short_exit = sc.short_exit_conditions;
    strategy
}

#[derive(Debug, Clone)]
pub struct BacktestOptions {
    pub symbol: Option<String>,
    pub json: bool,
    pub no_tui: bool,
    pub preset: Option<String>,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn describe_condition(cond: &BuiltCondition) -> String {
    let ind_name = cond.indicator.display_name();
    let comp = cond.comparison.symbol();
    match &cond.target {
        CompareTarget::Value(v) => format!("{} {} {:.1}", ind_name, comp, v),
        CompareTarget::Range(low, high) => format!("{:.1} < {} < {:.1}", low, ind_name, high),
        CompareTarget::Indicator(other) => {
            format!("{} {} {}", ind_name, comp, other.display_name())
        }
    }
}

// ============================================================================
// TUI FUNCTIONS
// ============================================================================

fn run_tui_loop(mut app: App) -> Result<Option<BacktestConfiguration>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if app.should_quit || app.confirmed {
            break;
        }

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            handle_input(&mut app, key.code, key.modifiers);
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(if app.confirmed {
        Some(app.config)
    } else {
        None
    })
}

fn run_config_tui(initial_symbol: Option<String>) -> Result<Option<BacktestConfiguration>> {
    run_tui_loop(App::new(initial_symbol))
}

/// Run config TUI with an existing configuration for editing
fn run_config_tui_with_config(
    config: BacktestConfiguration,
) -> Result<Option<BacktestConfiguration>> {
    let mut app = App::new(None);
    app.config = config;
    app.screen = Screen::ConfigEditor;
    run_tui_loop(app)
}

// ============================================================================
// MAIN EXECUTE FUNCTION
// ============================================================================

pub async fn execute(args: BacktestOptions) -> Result<()> {
    // Handle preset mode
    if let Some(ref preset_name) = args.preset {
        let presets = StrategyPreset::all();
        let preset = presets
            .iter()
            .find(|p| p.name.to_lowercase().contains(&preset_name.to_lowercase()))
            .ok_or_else(|| {
                crate::error::CliError::InvalidArgument(format!(
                    "Unknown preset '{}'. Available: {}",
                    preset_name,
                    presets
                        .iter()
                        .map(|p| p.name)
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            })?;

        let mut config = (preset.config)();
        if let Some(ref sym) = args.symbol {
            config.symbol = sym.to_uppercase();
        }

        if config.symbol.is_empty() {
            return Err(crate::error::CliError::InvalidArgument(
                "Symbol required. Use: fq backtest <SYMBOL> --preset <name>".into(),
            ));
        }

        // Build and run backtest with preset
        run_backtest_with_config(config, args.json).await?;
        return Ok(());
    }

    // Interactive TUI mode
    if !args.no_tui {
        let mut config = match run_config_tui(args.symbol)? {
            Some(cfg) => cfg,
            None => {
                eprintln!("Backtest cancelled.");
                return Ok(());
            }
        };

        // Iteration loop: run backtest, show results, optionally edit and re-run
        loop {
            match run_backtest_with_config(config.clone(), args.json).await? {
                ResultsAction::Quit => break,
                ResultsAction::Retry => {
                    // Re-run with same config
                    continue;
                }
                ResultsAction::NewStrategy => {
                    // Re-open config TUI with current config
                    config = match run_config_tui_with_config(config)? {
                        Some(cfg) => cfg,
                        None => break,
                    };
                }
            }
        }

        return Ok(());
    }

    // --no-tui without --preset: run with defaults if a symbol was given
    let symbol = args.symbol.ok_or_else(|| {
        crate::error::CliError::InvalidArgument(
            "Symbol required. Use: fq backtest <SYMBOL> --no-tui".into(),
        )
    })?;

    let config = BacktestConfiguration {
        symbol: symbol.to_uppercase(),
        ..BacktestConfiguration::default()
    };

    run_backtest_with_config(config, args.json).await?;
    Ok(())
}

async fn run_backtest_with_config(
    config: BacktestConfiguration,
    json_output: bool,
) -> Result<ResultsAction> {
    use finance_query::backtesting::{
        BacktestConfig as LibBacktestConfig, GridSearch, ParamRange, ParamValue, WalkForwardConfig,
    };
    use std::collections::HashMap;

    let entry_conds = &config.strategy.entry_conditions.conditions;
    let exit_conds = &config.strategy.exit_conditions.conditions;

    // Log conditions to stderr
    if !json_output && !entry_conds.is_empty() {
        eprintln!(
            "Running backtest for {} — {}",
            config.symbol, config.strategy.name
        );
        for cond in entry_conds {
            eprintln!("  Entry: {}", describe_condition(cond));
        }
        for cond in exit_conds {
            eprintln!("  Exit:  {}", describe_condition(cond));
        }
        eprintln!();
    }

    let ticker = Ticker::new(&config.symbol).await?;

    // Build BacktestConfig
    let mut builder = LibBacktestConfig::builder()
        .initial_capital(config.capital)
        .commission(config.commission_flat)
        .commission_pct(config.commission)
        .slippage_pct(config.slippage)
        .position_size_pct(config.position_size)
        .allow_short(config.allow_short)
        .risk_free_rate(config.risk_free_rate)
        .bars_per_year(bars_per_year_for_interval(config.interval))
        .reinvest_dividends(config.reinvest_dividends);

    if let Some(sl) = config.stop_loss {
        builder = builder.stop_loss_pct(sl);
    }
    if let Some(tp) = config.take_profit {
        builder = builder.take_profit_pct(tp);
    }
    if let Some(ts) = config.trailing_stop {
        builder = builder.trailing_stop_pct(ts);
    }

    let backtest_config = builder
        .build()
        .map_err(|e| crate::error::CliError::InvalidArgument(format!("Invalid config: {}", e)))?;

    // ── Optimizer path ────────────────────────────────────────────────────────
    if let Some(ref opt_config) = config.optimizer {
        let enabled: Vec<_> = opt_config.params.iter().filter(|p| p.enabled).collect();

        if enabled.is_empty() {
            return Err(crate::error::CliError::InvalidArgument(
                "No optimizer parameters enabled. Enable at least one parameter.".into(),
            ));
        }

        // Fetch candles for the optimizer. GridSearch/WalkForwardConfig call
        // BacktestEngine::run (no-dividend variant) internally, so reinvest_dividends
        // has no effect during optimization runs. Dividend income is intentionally
        // excluded to keep combination results comparable.
        let chart = ticker.chart(config.interval, config.range).await?;
        let candles = chart.candles;

        // Build the grid search
        let mut grid = GridSearch::new().optimize_for(opt_config.metric);
        for p in &enabled {
            grid = grid.param(
                p.name.clone(),
                ParamRange::float_range(p.start, p.end, p.step),
            );
        }

        // Factory: clone the strategy config and substitute param values
        let strategy_cfg = config.strategy.clone();
        let enabled_params: Vec<_> = enabled.into_iter().cloned().collect();

        let factory = {
            let strategy_cfg = strategy_cfg.clone();
            let enabled_params = enabled_params.clone();
            move |params: &HashMap<String, ParamValue>| {
                build_strategy_from_params(&strategy_cfg, &enabled_params, params)
            }
        };

        // Keep a copy for the optional benchmark re-run below.
        let saved_config = backtest_config.clone();

        let run_result = if opt_config.walk_forward {
            let wf = WalkForwardConfig::new(grid, backtest_config)
                .in_sample_bars(opt_config.in_sample_bars)
                .out_of_sample_bars(opt_config.out_of_sample_bars);

            let wf_report = wf.run(&config.symbol, &candles, factory).map_err(|e| {
                crate::error::CliError::InvalidArgument(format!("Walk-forward failed: {}", e))
            })?;

            if wf_report.windows.is_empty() {
                return Err(crate::error::CliError::InvalidArgument(
                    "Walk-forward produced no windows".into(),
                ));
            }

            // Build an aggregate primary result from ALL out-of-sample windows so the
            // Overview tab reflects realistic OOS performance rather than the narrow
            // last window (which often has too few bars to generate trades).
            let primary = {
                let first_oos = &wf_report.windows[0].out_of_sample;
                let last_oos = &wf_report.windows.last().unwrap().out_of_sample;

                let all_trades: Vec<_> = wf_report
                    .windows
                    .iter()
                    .flat_map(|w| w.out_of_sample.trades.iter().cloned())
                    .collect();

                let all_signals: Vec<_> = wf_report
                    .windows
                    .iter()
                    .flat_map(|w| w.out_of_sample.signals.iter().cloned())
                    .collect();

                // Stitch OOS equity curves into one compounded series. Each OOS
                // window starts from its own initial capital, so we scale each
                // window by the running equity from the previous window and
                // recompute drawdowns on the stitched curve.
                let mut combined_equity = Vec::new();
                let mut running_equity = first_oos.initial_capital;

                for (window_idx, window) in wf_report.windows.iter().enumerate() {
                    let window_initial = window.out_of_sample.initial_capital;
                    if window_initial <= 0.0 {
                        // Blown-up window — collapse running equity to zero and
                        // propagate that forward so subsequent windows stay at zero.
                        running_equity = 0.0;
                        continue;
                    }

                    for (point_idx, point) in window.out_of_sample.equity_curve.iter().enumerate() {
                        // Avoid duplicating the boundary point between adjacent windows.
                        if window_idx > 0 && point_idx == 0 {
                            continue;
                        }

                        let scaled_equity = running_equity * (point.equity / window_initial);
                        let mut scaled_point = point.clone();
                        scaled_point.equity = scaled_equity;
                        scaled_point.drawdown_pct = 0.0;
                        combined_equity.push(scaled_point);
                    }

                    if let Some(last) = combined_equity.last() {
                        running_equity = last.equity;
                    }
                }

                let mut peak = f64::NEG_INFINITY;
                for point in &mut combined_equity {
                    peak = peak.max(point.equity);
                    point.drawdown_pct = if peak > 0.0 {
                        (peak - point.equity) / peak
                    } else {
                        0.0
                    };
                }

                let mut primary = first_oos.clone();
                primary.strategy_name = wf_report.strategy_name.clone();
                primary.end_timestamp = last_oos.end_timestamp;
                primary.final_equity = combined_equity
                    .last()
                    .map(|p| p.equity)
                    .unwrap_or(first_oos.initial_capital);
                primary.metrics = wf_report.aggregate_metrics.clone();
                primary.trades = all_trades;
                primary.equity_curve = combined_equity;
                primary.signals = all_signals;
                primary.open_position = None;
                primary.benchmark = None;
                primary
            };

            RunResult {
                backtest: primary,
                optimization: None,
                walk_forward: Some(wf_report),
                opt_metric: Some(opt_config.metric),
                bench_candles: None,
            }
        } else {
            let opt_report = grid
                .run(&config.symbol, &candles, &backtest_config, factory)
                .map_err(|e| {
                    crate::error::CliError::InvalidArgument(format!("Optimization failed: {}", e))
                })?;

            // Re-run the winning strategy through the full dividend-aware path so
            // the displayed results (equity curve, P&L, Sharpe) include dividend
            // income and match what the normal backtest path would produce.
            let best_strategy = build_strategy_from_params(
                &config.strategy,
                &enabled_params,
                &opt_report.best.params,
            );
            let primary = ticker
                .backtest(
                    best_strategy,
                    config.interval,
                    config.range,
                    Some(saved_config.clone()),
                )
                .await
                .unwrap_or_else(|_| opt_report.best.result.clone());

            RunResult {
                backtest: primary,
                optimization: Some(opt_report),
                walk_forward: None,
                opt_metric: Some(opt_config.metric),
                bench_candles: None,
            }
        };

        // ── Attach benchmark metrics if a benchmark symbol was configured ─────
        // The optimizer/walk-forward engines don't compute benchmark comparisons,
        // so we run the winning strategy once more via BacktestEngine::run_with_benchmark
        // to get alpha, beta, information ratio, etc.
        let mut run_result = run_result;
        if let Some(ref bench_sym) = config.benchmark {
            use finance_query::backtesting::BacktestEngine;

            let best_params = if let Some(ref wf) = run_result.walk_forward {
                // Use the last window's optimized params
                wf.windows
                    .last()
                    .map(|w| w.optimized_params.clone())
                    .unwrap_or_default()
            } else if let Some(ref opt) = run_result.optimization {
                opt.best.params.clone()
            } else {
                HashMap::new()
            };

            let best_strategy =
                build_strategy_from_params(&config.strategy, &enabled_params, &best_params);

            match Ticker::new(bench_sym).await {
                Ok(bench_ticker) => {
                    if let Ok(bench_chart) = bench_ticker.chart(config.interval, config.range).await
                    {
                        let engine = BacktestEngine::new(saved_config.clone());
                        if let Ok(bench_result) = engine.run_with_benchmark(
                            &config.symbol,
                            &candles,
                            best_strategy,
                            &[],
                            bench_sym,
                            &bench_chart.candles,
                        ) {
                            run_result.backtest.benchmark = bench_result.benchmark;
                        }
                        run_result.bench_candles = Some(bench_chart.candles);
                    }
                }
                Err(e) => {
                    if !json_output {
                        eprintln!("Warning: could not fetch benchmark {}: {}", bench_sym, e);
                    }
                }
            }
        }

        return if json_output {
            let json = serde_json::to_string_pretty(&run_result.backtest)?;
            println!("{}", json);
            Ok(ResultsAction::Quit)
        } else {
            run_results_tui(run_result)
        };
    }

    // ── Normal backtest path ──────────────────────────────────────────────────

    // Dispatch a concrete Strategy to either backtest() or backtest_with_benchmark().
    // backtest_config is cloned per call because the library takes ownership.
    macro_rules! run {
        ($strategy:expr) => {
            match &config.benchmark {
                Some(bench) => {
                    ticker
                        .backtest_with_benchmark(
                            $strategy,
                            config.interval,
                            config.range,
                            Some(backtest_config.clone()),
                            bench,
                        )
                        .await?
                }
                None => {
                    ticker
                        .backtest(
                            $strategy,
                            config.interval,
                            config.range,
                            Some(backtest_config.clone()),
                        )
                        .await?
                }
            }
        };
    }

    // If the user built custom entry/exit conditions, use DynamicStrategy so
    // ALL conditions are honoured exactly as configured. Only fall back to
    // prebuilt strategies when no conditions have been defined (e.g. preset
    // mode where the preset itself picks the strategy type below).
    let result = if !entry_conds.is_empty() && !exit_conds.is_empty() {
        let mut strategy = dynamic_strategy::DynamicStrategy::new(
            config.strategy.name.clone(),
            config.strategy.entry_conditions.clone(),
            config.strategy.exit_conditions.clone(),
        );
        strategy.short_entry = config.strategy.short_entry_conditions.clone();
        strategy.short_exit = config.strategy.short_exit_conditions.clone();
        run!(strategy)
    } else {
        // No conditions — default preset fallback
        use finance_query::backtesting::SmaCrossover;
        run!(SmaCrossover::new(20, 50))
    };

    let mut run_result = RunResult::simple(result);

    // Fetch benchmark candles for the actual buy-and-hold equity curve on the
    // Charts tab. The chart data will be cached inside the bench Ticker, so this
    // adds at most one network round-trip and is a no-op when the cache is warm.
    if let Some(ref bench_sym) = config.benchmark
        && let Ok(bench_ticker) = Ticker::new(bench_sym).await
        && let Ok(bench_chart) = bench_ticker.chart(config.interval, config.range).await
    {
        run_result.bench_candles = Some(bench_chart.candles);
    }

    if json_output {
        let json = serde_json::to_string_pretty(&run_result.backtest)?;
        println!("{}", json);
        Ok(ResultsAction::Quit)
    } else {
        run_results_tui(run_result)
    }
}

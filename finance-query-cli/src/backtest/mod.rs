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
const GROUP_SCALE_IN: usize = 4;
const GROUP_SCALE_OUT: usize = 5;
const GROUP_REGIME: usize = 6;

fn build_strategy_from_params(
    strategy_cfg: &StrategyConfig,
    enabled_params: &[OptimizerParamDef],
    params: &HashMap<String, ParamValue>,
) -> dynamic_strategy::DynamicStrategy {
    let mut sc = strategy_cfg.clone();
    for p in enabled_params {
        let val = params.get(&p.name).map(|v| v.as_float()).unwrap_or(p.start);
        let group_conditions = match p.group {
            GROUP_ENTRY => Some(&mut sc.entry_conditions.conditions),
            GROUP_EXIT => Some(&mut sc.exit_conditions.conditions),
            GROUP_SCALE_IN => Some(&mut sc.scale_in_conditions.conditions),
            GROUP_SCALE_OUT => Some(&mut sc.scale_out_conditions.conditions),
            GROUP_REGIME => Some(&mut sc.regime_conditions.conditions),
            _ => None,
        };
        if let Some(conds) = group_conditions {
            if let Some(cond) = conds.get_mut(p.condition_idx)
                && let Some(pv) = cond.indicator.param_values.get_mut(p.param_idx)
            {
                *pv = val;
            }
        } else {
            let opt_group = match p.group {
                GROUP_SHORT_ENTRY => sc.short_entry_conditions.as_mut(),
                GROUP_SHORT_EXIT => sc.short_exit_conditions.as_mut(),
                _ => None,
            };
            if let Some(group) = opt_group
                && let Some(cond) = group.conditions.get_mut(p.condition_idx)
                && let Some(pv) = cond.indicator.param_values.get_mut(p.param_idx)
            {
                *pv = val;
            }
        }
    }
    build_dynamic_strategy(&sc)
}

fn build_dynamic_strategy(sc: &StrategyConfig) -> dynamic_strategy::DynamicStrategy {
    let mut strategy = dynamic_strategy::DynamicStrategy::new(
        sc.name.clone(),
        sc.entry_conditions.clone(),
        sc.exit_conditions.clone(),
    );
    strategy.short_entry = sc.short_entry_conditions.clone();
    strategy.short_exit = sc.short_exit_conditions.clone();
    strategy.regime = sc.regime_conditions.clone();
    strategy.warmup_bars = sc.warmup_bars;
    strategy.scale_in = sc.scale_in_conditions.clone();
    strategy.scale_in_fraction = sc.scale_in_fraction;
    strategy.scale_out = sc.scale_out_conditions.clone();
    strategy.scale_out_fraction = sc.scale_out_fraction;
    strategy.entry_order_type = sc.entry_order_type;
    strategy.entry_price_offset_pct = sc.entry_price_offset_pct;
    strategy.entry_stop_limit_gap_pct = sc.entry_stop_limit_gap_pct;
    strategy.entry_expires_bars = sc.entry_expires_bars;
    strategy.entry_bracket_sl = sc.entry_bracket_sl;
    strategy.entry_bracket_tp = sc.entry_bracket_tp;
    strategy.entry_bracket_trail = sc.entry_bracket_trail;
    strategy.short_order_type = sc.short_order_type;
    strategy.short_price_offset_pct = sc.short_price_offset_pct;
    strategy.short_expires_bars = sc.short_expires_bars;
    strategy.short_bracket_sl = sc.short_bracket_sl;
    strategy.short_bracket_tp = sc.short_bracket_tp;
    strategy.short_bracket_trail = sc.short_bracket_trail;
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
        BacktestConfig as LibBacktestConfig, BayesianSearch, GridSearch, ParamRange, ParamValue,
        WalkForwardConfig,
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

    // Build BacktestConfig
    let mut builder = LibBacktestConfig::builder()
        .initial_capital(config.capital)
        .commission(config.commission_flat)
        .commission_pct(config.commission)
        .slippage_pct(config.slippage)
        .spread_pct(config.spread_pct)
        .transaction_tax_pct(config.transaction_tax_pct)
        .position_size_pct(config.position_size)
        .allow_short(config.allow_short)
        .min_signal_strength(config.min_signal_strength)
        .close_at_end(config.close_at_end)
        .risk_free_rate(config.risk_free_rate)
        .bars_per_year(config.bars_per_year)
        .reinvest_dividends(config.reinvest_dividends);

    if config.max_positions == 0 {
        builder = builder.unlimited_positions();
    } else {
        builder = builder.max_positions(config.max_positions);
    }

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

    if config.optimizer.is_some() && config.ensemble.is_some() {
        return Err(crate::error::CliError::InvalidArgument(
            "Optimizer is not supported for ensemble composition yet".into(),
        ));
    }

    if config.optimizer.is_some() && !config.portfolio_symbols.is_empty() {
        return Err(crate::error::CliError::InvalidArgument(
            "Optimizer is not supported in portfolio mode. \
             Run the optimizer on a single symbol first, then apply the best parameters to a portfolio run."
                .into(),
        ));
    }

    // ── Portfolio path ────────────────────────────────────────────────────────
    if !config.portfolio_symbols.is_empty() {
        if config.ensemble.is_some() {
            return Err(crate::error::CliError::InvalidArgument(
                "Ensemble strategies are not supported in portfolio mode. \
                 Use a single strategy or run each ensemble member as a separate portfolio symbol."
                    .into(),
            ));
        }
        return run_portfolio_backtest(config, backtest_config, json_output).await;
    }

    let ticker = Ticker::new(&config.symbol).await?;

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

        // Factory: clone the strategy config and substitute param values
        let strategy_cfg = config.strategy.clone();
        let enabled_params: Vec<_> = enabled.into_iter().cloned().collect();

        // Keep a copy for the optional benchmark re-run below.
        let saved_config = backtest_config.clone();

        let run_result = if opt_config.walk_forward {
            // Walk-forward always uses GridSearch (Bayesian not yet supported for WF)
            let mut grid = GridSearch::new().optimize_for(opt_config.metric);
            for p in &enabled_params {
                grid = grid.param(
                    p.name.clone(),
                    ParamRange::float_range(p.start, p.end, p.step),
                );
            }
            let factory = {
                let strategy_cfg = strategy_cfg.clone();
                let enabled_params = enabled_params.clone();
                move |params: &HashMap<String, ParamValue>| {
                    build_strategy_from_params(&strategy_cfg, &enabled_params, params)
                }
            };
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
                portfolio: None,
            }
        } else {
            use crate::backtest::types::SearchMethod;
            let opt_report = match opt_config.search_method {
                SearchMethod::Bayesian => {
                    let mut bayesian = BayesianSearch::new()
                        .optimize_for(opt_config.metric)
                        .max_evaluations(opt_config.bayesian_trials);
                    for p in &enabled_params {
                        // Use float bounds when the step is fractional OR when
                        // start/end have a fractional component (e.g. ATR
                        // multipliers like 2.0 with a coarse step of 1.0 should
                        // still search the continuous space). Integer parameters
                        // (periods, bars) use int bounds.
                        let range =
                            if p.step < 1.0 || p.start.fract() != 0.0 || p.end.fract() != 0.0 {
                                ParamRange::float_bounds(p.start, p.end)
                            } else {
                                ParamRange::int_bounds(p.start as i64, p.end as i64)
                            };
                        bayesian = bayesian.param(p.name.clone(), range);
                    }
                    let factory = {
                        let strategy_cfg = strategy_cfg.clone();
                        let enabled_params = enabled_params.clone();
                        move |params: &HashMap<String, ParamValue>| {
                            build_strategy_from_params(&strategy_cfg, &enabled_params, params)
                        }
                    };
                    bayesian
                        .run(&config.symbol, &candles, &backtest_config, factory)
                        .map_err(|e| {
                            crate::error::CliError::InvalidArgument(format!(
                                "Bayesian optimization failed: {}",
                                e
                            ))
                        })?
                }
                SearchMethod::Grid => {
                    let mut grid = GridSearch::new().optimize_for(opt_config.metric);
                    for p in &enabled_params {
                        grid = grid.param(
                            p.name.clone(),
                            ParamRange::float_range(p.start, p.end, p.step),
                        );
                    }
                    let factory = {
                        let strategy_cfg = strategy_cfg.clone();
                        let enabled_params = enabled_params.clone();
                        move |params: &HashMap<String, ParamValue>| {
                            build_strategy_from_params(&strategy_cfg, &enabled_params, params)
                        }
                    };
                    grid.run(&config.symbol, &candles, &backtest_config, factory)
                        .map_err(|e| {
                            crate::error::CliError::InvalidArgument(format!(
                                "Optimization failed: {}",
                                e
                            ))
                        })?
                }
            };

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
                portfolio: None,
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
    let result = if let Some(ensemble_cfg) = &config.ensemble {
        use finance_query::backtesting::{EnsembleMode, EnsembleStrategy};

        if ensemble_cfg.members.len() < 2 {
            return Err(crate::error::CliError::InvalidArgument(
                "Ensemble requires at least 2 member strategies".into(),
            ));
        }

        let mode = match ensemble_cfg.mode {
            crate::backtest::types::EnsembleModeChoice::WeightedMajority => {
                EnsembleMode::WeightedMajority
            }
            crate::backtest::types::EnsembleModeChoice::Unanimous => EnsembleMode::Unanimous,
            crate::backtest::types::EnsembleModeChoice::AnySignal => EnsembleMode::AnySignal,
            crate::backtest::types::EnsembleModeChoice::StrongestSignal => {
                EnsembleMode::StrongestSignal
            }
        };

        let mut ensemble = EnsembleStrategy::new(config.strategy.name.clone()).mode(mode);
        for member in &ensemble_cfg.members {
            if member.strategy.entry_conditions.conditions.is_empty()
                || member.strategy.exit_conditions.conditions.is_empty()
            {
                return Err(crate::error::CliError::InvalidArgument(format!(
                    "Ensemble member '{}' must define both entry and exit conditions",
                    member.name
                )));
            }
            let strategy = build_dynamic_strategy(&member.strategy);
            ensemble = ensemble.add(strategy, member.weight);
        }

        run!(ensemble.build())
    } else if !entry_conds.is_empty() && !exit_conds.is_empty() {
        run!(build_dynamic_strategy(&config.strategy))
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

// ── Portfolio backtest path ───────────────────────────────────────────────────

/// Runs the same strategy across all portfolio symbols concurrently using a
/// shared capital pool, then opens the results TUI with the Portfolio tab
/// as the default view.
async fn run_portfolio_backtest(
    config: BacktestConfiguration,
    backtest_config: finance_query::backtesting::BacktestConfig,
    json_output: bool,
) -> Result<ResultsAction> {
    use finance_query::backtesting::portfolio::{
        PortfolioConfig, PortfolioEngine, RebalanceMode, SymbolData,
    };

    // Combine the primary symbol with the extra portfolio symbols.
    let mut all_symbols = vec![config.symbol.clone()];
    for sym in &config.portfolio_symbols {
        let sym_upper = sym.trim().to_uppercase();
        if !sym_upper.is_empty() && !all_symbols.contains(&sym_upper) {
            all_symbols.push(sym_upper);
        }
    }

    if !json_output {
        eprintln!(
            "Running portfolio backtest across {} symbols: {}",
            all_symbols.len(),
            all_symbols.join(", ")
        );
    }

    let rebalance_mode = match config.rebalance_mode {
        types::RebalanceModeChoice::AvailableCapital => RebalanceMode::AvailableCapital,
        types::RebalanceModeChoice::EqualWeight => RebalanceMode::EqualWeight,
    };

    // Warn when benchmark is set — benchmark-relative analytics are not computed
    // in portfolio mode and the field will be silently ignored.
    if config.benchmark.is_some() {
        eprintln!(
            "Warning: benchmark analytics (alpha/beta/information ratio) are not available \
             in portfolio mode and will be skipped."
        );
    }

    let mut portfolio_config = PortfolioConfig::new(backtest_config).rebalance(rebalance_mode);

    // Wire per-symbol position cap: max_positions is interpreted as the global
    // concurrent-positions limit across all symbols in portfolio mode.
    if config.max_positions > 0 {
        portfolio_config = portfolio_config.max_total_positions(config.max_positions);
    }

    // Wire per-symbol allocation cap when the user has set a non-zero value.
    if config.max_allocation_per_symbol > 0.0 {
        portfolio_config =
            portfolio_config.max_allocation_per_symbol(config.max_allocation_per_symbol);
    }

    // Fetch candles (and optionally dividends) for every symbol.
    let mut symbol_data: Vec<SymbolData> = Vec::with_capacity(all_symbols.len());
    for sym in &all_symbols {
        let ticker = Ticker::new(sym).await.map_err(|e| {
            crate::error::CliError::InvalidArgument(format!(
                "Failed to fetch ticker {}: {}",
                sym, e
            ))
        })?;
        let chart = ticker
            .chart(config.interval, config.range)
            .await
            .map_err(|e| {
                crate::error::CliError::InvalidArgument(format!(
                    "Failed to fetch chart for {}: {}",
                    sym, e
                ))
            })?;

        // Always attach dividends so the engine can credit cash income even when
        // reinvest_dividends=false (the engine decides reinvest vs. cash; we
        // always provide the data). Falls back to empty vec on fetch failure,
        // consistent with the single-symbol Ticker::backtest() path.
        let divs = match ticker.dividends(config.range).await {
            Ok(d) => d,
            Err(e) => {
                eprintln!(
                    "Warning: failed to fetch dividends for {} — dividend income will be excluded. ({e})",
                    sym
                );
                Vec::new()
            }
        };
        let sd = SymbolData::new(sym.clone(), chart.candles).with_dividends(divs);
        symbol_data.push(sd);
    }

    // Build strategy factory — all symbols share the same strategy definition.
    let strategy_cfg = config.strategy.clone();

    let portfolio_result = PortfolioEngine::new(portfolio_config)
        .run(&symbol_data, |_sym| build_dynamic_strategy(&strategy_cfg))
        .map_err(|e| {
            crate::error::CliError::InvalidArgument(format!("Portfolio backtest failed: {}", e))
        })?;

    // Pick the primary single-symbol result for the existing per-symbol tabs.
    // Prefer the symbol with the most trades so the Overview / Charts tabs have
    // the richest data to display.
    let primary_sym = portfolio_result
        .symbols
        .iter()
        .max_by_key(|(_, r)| r.trades.len())
        .map(|(sym, _)| sym.clone())
        .unwrap_or_else(|| all_symbols[0].clone());

    let primary = portfolio_result
        .symbols
        .get(&primary_sym)
        .or_else(|| portfolio_result.symbols.values().next())
        .cloned()
        .ok_or_else(|| {
            crate::error::CliError::InvalidArgument(
                "Portfolio backtest returned no symbol results".into(),
            )
        })?;

    let mut run_result = RunResult::simple(primary);
    run_result.portfolio = Some(portfolio_result);

    if json_output {
        // JSON mode: emit the full PortfolioResult (aggregate metrics + equity
        // curve + per-symbol breakdown) so automated consumers get a complete
        // picture rather than only the per-symbol map.
        let json =
            serde_json::to_string_pretty(run_result.portfolio.as_ref().ok_or_else(|| {
                crate::error::CliError::InvalidArgument(
                    "Portfolio result unavailable for JSON output".into(),
                )
            })?)?;
        println!("{}", json);
        Ok(ResultsAction::Quit)
    } else {
        run_results_tui(run_result)
    }
}

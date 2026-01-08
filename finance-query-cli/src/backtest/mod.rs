mod indicators;
mod input;
mod presets;
mod render;
mod results;
mod state;
mod types;

// Re-exports for public API (these are used by other parts of the crate)
#[allow(unused_imports)]
pub use indicators::{IndicatorCategory, IndicatorDef, ParamDef};
pub use presets::StrategyPreset;
pub use results::{ResultsAction, run_results_tui};
pub use state::{App, Screen};
pub use types::{
    BacktestConfiguration, BuiltCondition, BuiltIndicator, CompareTarget, ComparisonType,
};

use crate::error::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use finance_query::Ticker;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use input::handle_input;
use render::ui;

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

/// Build an indicator reference from the TUI's BuiltIndicator
/// Returns a tuple of (indicator_key, params)
fn get_indicator_ref_params(ind: &BuiltIndicator) -> (String, Vec<usize>) {
    let params: Vec<usize> = ind.param_values.iter().map(|v| *v as usize).collect();
    (ind.indicator.code.to_string(), params)
}

/// Describe a condition for display purposes
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

/// Generate a code representation of the indicator for logging
fn indicator_to_code(ind: &BuiltIndicator) -> String {
    ind.code_string()
}

// ============================================================================
// TUI FUNCTIONS
// ============================================================================

fn run_config_tui(initial_symbol: Option<String>) -> Result<Option<BacktestConfiguration>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(initial_symbol);

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if app.should_quit {
            break;
        }

        if app.confirmed {
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

    if app.confirmed {
        Ok(Some(app.config))
    } else {
        Ok(None)
    }
}

/// Run config TUI with an existing configuration for editing
fn run_config_tui_with_config(
    config: BacktestConfiguration,
) -> Result<Option<BacktestConfiguration>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(None);
    app.config = config;
    app.screen = Screen::ConfigEditor;

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if app.should_quit {
            break;
        }

        if app.confirmed {
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

    if app.confirmed {
        Ok(Some(app.config))
    } else {
        Ok(None)
    }
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
                ResultsAction::Continue => {
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

    Err(crate::error::CliError::InvalidArgument(
        "Use interactive mode (default) or --preset <name>".into(),
    ))
}

async fn run_backtest_with_config(
    config: BacktestConfiguration,
    json_output: bool,
) -> Result<ResultsAction> {
    use finance_query::backtesting::{
        BacktestConfig as LibBacktestConfig, BollingerMeanReversion, DonchianBreakout, MacdSignal,
        RsiReversal, SmaCrossover, SuperTrendFollow,
    };

    // Show what strategy we're running
    let entry_conds = &config.strategy.entry_conditions.conditions;
    if !json_output && !entry_conds.is_empty() {
        eprintln!("Running backtest for {} with conditions:", config.symbol);
        for cond in entry_conds {
            eprintln!(
                "  Entry: {} [{}]",
                describe_condition(cond),
                indicator_to_code(&cond.indicator)
            );
        }
        for cond in &config.strategy.exit_conditions.conditions {
            eprintln!(
                "  Exit: {} [{}]",
                describe_condition(cond),
                indicator_to_code(&cond.indicator)
            );
        }
        eprintln!();
    }

    let ticker = Ticker::new(&config.symbol).await?;

    // Build config
    let mut builder = LibBacktestConfig::builder()
        .initial_capital(config.capital)
        .commission_pct(config.commission)
        .slippage_pct(config.slippage)
        .position_size_pct(config.position_size)
        .allow_short(config.allow_short);

    if let Some(sl) = config.stop_loss {
        builder = builder.stop_loss_pct(sl);
    }
    if let Some(tp) = config.take_profit {
        builder = builder.take_profit_pct(tp);
    }

    let backtest_config = builder
        .build()
        .map_err(|e| crate::error::CliError::InvalidArgument(format!("Invalid config: {}", e)))?;

    // Determine strategy from entry conditions
    let result = if let Some(first_cond) = entry_conds.first() {
        let (code, params) = get_indicator_ref_params(&first_cond.indicator);

        match code.as_str() {
            "sma" | "ema" | "wma" | "dema" | "tema" | "hma" | "vwma" | "mcginley" => {
                // Moving average crossover - look for two MA conditions or use defaults
                let periods: Vec<usize> = entry_conds
                    .iter()
                    .filter(|c| {
                        let (c_code, _) = get_indicator_ref_params(&c.indicator);
                        c_code == code
                    })
                    .filter_map(|c| c.indicator.param_values.first().map(|v| *v as usize))
                    .collect();

                let (fast, slow) = if periods.len() >= 2 {
                    (periods[0].min(periods[1]), periods[0].max(periods[1]))
                } else if !params.is_empty() {
                    (params[0], params[0] * 2)
                } else {
                    (20, 50)
                };

                let strategy = SmaCrossover::new(fast, slow).with_short(config.allow_short);
                ticker
                    .backtest(
                        strategy,
                        config.interval,
                        config.range,
                        Some(backtest_config),
                    )
                    .await?
            }
            "rsi" | "stochastic" | "cci" | "williams_r" | "cmo" | "mfi" => {
                let period = params.first().copied().unwrap_or(14);

                let (oversold, overbought) = match &first_cond.target {
                    CompareTarget::Value(v) => {
                        if first_cond.comparison == ComparisonType::Below
                            || first_cond.comparison == ComparisonType::CrossesBelow
                        {
                            (*v, 100.0 - *v)
                        } else {
                            (30.0, *v)
                        }
                    }
                    CompareTarget::Range(low, high) => (*low, *high),
                    _ => (30.0, 70.0),
                };

                let strategy = RsiReversal::new(period)
                    .with_thresholds(oversold, overbought)
                    .with_short(config.allow_short);

                ticker
                    .backtest(
                        strategy,
                        config.interval,
                        config.range,
                        Some(backtest_config),
                    )
                    .await?
            }
            "macd" => {
                let fast = params.first().copied().unwrap_or(12);
                let slow = params.get(1).copied().unwrap_or(26);
                let signal = params.get(2).copied().unwrap_or(9);

                let strategy = MacdSignal::new(fast, slow, signal).with_short(config.allow_short);

                ticker
                    .backtest(
                        strategy,
                        config.interval,
                        config.range,
                        Some(backtest_config),
                    )
                    .await?
            }
            "bollinger" => {
                let period = params.first().copied().unwrap_or(20);
                let std_dev = first_cond
                    .indicator
                    .param_values
                    .get(1)
                    .copied()
                    .unwrap_or(2.0);

                let strategy =
                    BollingerMeanReversion::new(period, std_dev).with_short(config.allow_short);

                ticker
                    .backtest(
                        strategy,
                        config.interval,
                        config.range,
                        Some(backtest_config),
                    )
                    .await?
            }
            "supertrend" => {
                let period = params.first().copied().unwrap_or(10);
                let multiplier = first_cond
                    .indicator
                    .param_values
                    .get(1)
                    .copied()
                    .unwrap_or(3.0);

                let strategy =
                    SuperTrendFollow::new(period, multiplier).with_short(config.allow_short);

                ticker
                    .backtest(
                        strategy,
                        config.interval,
                        config.range,
                        Some(backtest_config),
                    )
                    .await?
            }
            "donchian" => {
                let period = params.first().copied().unwrap_or(20);

                let strategy = DonchianBreakout::new(period).with_short(config.allow_short);

                ticker
                    .backtest(
                        strategy,
                        config.interval,
                        config.range,
                        Some(backtest_config),
                    )
                    .await?
            }
            _ => {
                // Default to SMA crossover
                let strategy = SmaCrossover::new(20, 50).with_short(config.allow_short);
                ticker
                    .backtest(
                        strategy,
                        config.interval,
                        config.range,
                        Some(backtest_config),
                    )
                    .await?
            }
        }
    } else {
        // No conditions, use default SMA crossover
        let strategy = SmaCrossover::new(20, 50).with_short(config.allow_short);
        ticker
            .backtest(
                strategy,
                config.interval,
                config.range,
                Some(backtest_config),
            )
            .await?
    };

    if json_output {
        let json = serde_json::to_string_pretty(&result)?;
        println!("{}", json);
        Ok(ResultsAction::Quit)
    } else {
        run_results_tui(result)
    }
}

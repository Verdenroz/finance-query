mod input;
mod render;
mod state;

use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use finance_query::indicators::{Indicator, IndicatorResult};
use finance_query::{Interval, Ticker, TimeRange};
use ratatui::{Terminal, backend::CrosstermBackend};
use serde::Serialize;
use std::io;
use tabled::Tabled;

use input::handle_input;
use render::ui;
use state::App;

#[derive(Parser)]
pub struct IndicatorArgs {
    /// Stock symbol to calculate indicators for
    symbol: Option<String>,

    /// Skip interactive TUI and use these parameters directly
    /// Format: indicator_name:param1,param2,...
    /// Examples: rsi:14, macd:12,26,9, bollinger:20,2.0
    #[arg(long)]
    indicator: Option<String>,

    /// Time interval (1m, 5m, 15m, 1h, 1d, 1wk, 1mo)
    #[arg(short, long, default_value = "1d")]
    interval: String,

    /// Time range (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    #[arg(short, long, default_value = "3mo")]
    range: String,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,

    /// Show only the latest value
    #[arg(long)]
    latest: bool,

    /// Skip interactive TUI (requires --indicator)
    #[arg(long)]
    no_tui: bool,
}

#[derive(Debug, Serialize, Tabled)]
struct IndicatorRow {
    #[tabled(rename = "Date")]
    date: String,

    #[tabled(rename = "Close")]
    close: String,

    #[tabled(rename = "Value")]
    value: String,
}

#[derive(Debug, Serialize, Tabled)]
struct MultiIndicatorRow {
    #[tabled(rename = "Date")]
    date: String,

    #[tabled(rename = "Close")]
    close: String,

    #[tabled(rename = "Line 1")]
    line1: String,

    #[tabled(rename = "Line 2")]
    line2: String,

    #[tabled(rename = "Line 3")]
    line3: String,
}

pub async fn execute(args: IndicatorArgs) -> Result<()> {
    // If --no-tui is set, require --indicator and run non-interactively
    if args.no_tui {
        if args.indicator.is_none() {
            return Err(crate::error::CliError::InvalidArgument(
                "--no-tui requires --indicator to be specified".to_string(),
            ));
        }
        let symbol = args.symbol.ok_or_else(|| {
            crate::error::CliError::InvalidArgument(
                "--no-tui requires a symbol to be specified".to_string(),
            )
        })?;
        return execute_non_interactive(
            &symbol,
            args.indicator.as_deref().unwrap(),
            &args.interval,
            &args.range,
            &args.output,
            args.latest,
        )
        .await;
    }

    // If --indicator is provided without --no-tui, also run non-interactively
    if let Some(ref indicator) = args.indicator {
        let symbol = args.symbol.ok_or_else(|| {
            crate::error::CliError::InvalidArgument(
                "Symbol is required when using --indicator".to_string(),
            )
        })?;
        return execute_non_interactive(
            &symbol,
            indicator,
            &args.interval,
            &args.range,
            &args.output,
            args.latest,
        )
        .await;
    }

    // Launch interactive TUI
    let config = run_indicator_tui(args.symbol)?;

    if let Some(config) = config {
        // Execute the indicator calculation with the TUI-configured parameters
        execute_with_config(&config).await
    } else {
        Ok(()) // User cancelled
    }
}

/// Configuration from TUI
pub struct IndicatorConfig {
    pub symbol: String,
    pub indicator: Indicator,
    pub interval: Interval,
    pub range: TimeRange,
    pub format: OutputFormat,
    pub latest: bool,
}

fn run_indicator_tui(initial_symbol: Option<String>) -> Result<Option<IndicatorConfig>> {
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

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if app.confirmed {
        Ok(Some(app.build_config()?))
    } else {
        Ok(None)
    }
}

async fn execute_with_config(config: &IndicatorConfig) -> Result<()> {
    let ticker = Ticker::new(&config.symbol).await?;
    let chart = ticker.chart(config.interval, config.range).await?;
    let result = ticker
        .indicator(config.indicator, config.interval, config.range)
        .await?;

    display_result(&result, &chart.candles, config.format, config.latest)
}

async fn execute_non_interactive(
    symbol: &str,
    indicator_str: &str,
    interval_str: &str,
    range_str: &str,
    output_str: &str,
    latest: bool,
) -> Result<()> {
    let format = OutputFormat::from_str(output_str)?;
    let interval = parse_interval(interval_str)?;
    let range = parse_range(range_str)?;
    let indicator = parse_indicator(indicator_str)?;

    let ticker = Ticker::new(symbol).await?;
    let chart = ticker.chart(interval, range).await?;
    let result = ticker.indicator(indicator, interval, range).await?;

    display_result(&result, &chart.candles, format, latest)
}

fn display_result(
    result: &IndicatorResult,
    candles: &[finance_query::Candle],
    format: OutputFormat,
    latest: bool,
) -> Result<()> {
    match result {
        IndicatorResult::Series(values) => {
            let mut rows = Vec::new();
            for (idx, candle) in candles.iter().enumerate() {
                if latest && idx != candles.len() - 1 {
                    continue;
                }

                let date = chrono::DateTime::from_timestamp(candle.timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                rows.push(IndicatorRow {
                    date,
                    close: format!("{:.2}", candle.close),
                    value: values
                        .get(idx)
                        .and_then(|&v| v.map(|val| format!("{:.4}", val)))
                        .unwrap_or_else(|| "-".to_string()),
                });
            }
            output::print_many(&rows, format)?;
        }
        IndicatorResult::Macd(data) => {
            let mut rows = Vec::new();
            for (idx, candle) in candles.iter().enumerate() {
                if latest && idx != candles.len() - 1 {
                    continue;
                }

                let date = chrono::DateTime::from_timestamp(candle.timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                rows.push(MultiIndicatorRow {
                    date,
                    close: format!("{:.2}", candle.close),
                    line1: data
                        .macd_line
                        .get(idx)
                        .and_then(|&v| v.map(|val| format!("{:.4}", val)))
                        .unwrap_or_else(|| "-".to_string()),
                    line2: data
                        .signal_line
                        .get(idx)
                        .and_then(|&v| v.map(|val| format!("{:.4}", val)))
                        .unwrap_or_else(|| "-".to_string()),
                    line3: data
                        .histogram
                        .get(idx)
                        .and_then(|&v| v.map(|val| format!("{:.4}", val)))
                        .unwrap_or_else(|| "-".to_string()),
                });
            }

            if format == OutputFormat::Table {
                println!("MACD (Line 1: MACD, Line 2: Signal, Line 3: Histogram)");
            }
            output::print_many(&rows, format)?;
        }
        IndicatorResult::Bollinger(data) => {
            let mut rows = Vec::new();
            for (idx, candle) in candles.iter().enumerate() {
                if latest && idx != candles.len() - 1 {
                    continue;
                }

                let date = chrono::DateTime::from_timestamp(candle.timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                rows.push(MultiIndicatorRow {
                    date,
                    close: format!("{:.2}", candle.close),
                    line1: data
                        .upper
                        .get(idx)
                        .and_then(|&v| v.map(|val| format!("{:.4}", val)))
                        .unwrap_or_else(|| "-".to_string()),
                    line2: data
                        .middle
                        .get(idx)
                        .and_then(|&v| v.map(|val| format!("{:.4}", val)))
                        .unwrap_or_else(|| "-".to_string()),
                    line3: data
                        .lower
                        .get(idx)
                        .and_then(|&v| v.map(|val| format!("{:.4}", val)))
                        .unwrap_or_else(|| "-".to_string()),
                });
            }

            if format == OutputFormat::Table {
                println!("Bollinger Bands (Line 1: Upper, Line 2: Middle, Line 3: Lower)");
            }
            output::print_many(&rows, format)?;
        }
        IndicatorResult::Stochastic(data) => {
            let mut rows = Vec::new();
            for (idx, candle) in candles.iter().enumerate() {
                if latest && idx != candles.len() - 1 {
                    continue;
                }

                let date = chrono::DateTime::from_timestamp(candle.timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                rows.push(MultiIndicatorRow {
                    date,
                    close: format!("{:.2}", candle.close),
                    line1: data
                        .k
                        .get(idx)
                        .and_then(|&v| v.map(|val| format!("{:.4}", val)))
                        .unwrap_or_else(|| "-".to_string()),
                    line2: data
                        .d
                        .get(idx)
                        .and_then(|&v| v.map(|val| format!("{:.4}", val)))
                        .unwrap_or_else(|| "-".to_string()),
                    line3: "-".to_string(),
                });
            }

            if format == OutputFormat::Table {
                println!("Stochastic Oscillator (Line 1: %K, Line 2: %D)");
            }
            output::print_many(&rows, format)?;
        }
        _ => {
            return Err(crate::error::CliError::InvalidArgument(
                "This indicator type is not yet fully supported in CLI. Use the library API or JSON output.".to_string()
            ));
        }
    }

    Ok(())
}

fn parse_indicator(s: &str) -> Result<Indicator> {
    let (name, params) = match s.split_once(':') {
        Some((n, p)) => (n.trim(), Some(p.trim())),
        None => (s.trim(), None),
    };

    match name.to_lowercase().as_str() {
        // Moving Averages
        "sma" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
            Ok(Indicator::Sma(period))
        }
        "ema" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(12);
            Ok(Indicator::Ema(period))
        }
        "wma" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
            Ok(Indicator::Wma(period))
        }
        "dema" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
            Ok(Indicator::Dema(period))
        }
        "tema" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
            Ok(Indicator::Tema(period))
        }
        "hma" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
            Ok(Indicator::Hma(period))
        }
        "vwma" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
            Ok(Indicator::Vwma(period))
        }
        "alma" => {
            let (period, offset, sigma) = if let Some(p) = params {
                let parts: Vec<&str> = p.split(',').collect();
                let period = parts.first().and_then(|s| s.parse().ok()).unwrap_or(9);
                let offset = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.85);
                let sigma = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(6.0);
                (period, offset, sigma)
            } else {
                (9, 0.85, 6.0)
            };
            Ok(Indicator::Alma {
                period,
                offset,
                sigma,
            })
        }
        "mcginley" | "mcginley_dynamic" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
            Ok(Indicator::McginleyDynamic(period))
        }

        // Momentum Indicators
        "rsi" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(14);
            Ok(Indicator::Rsi(period))
        }
        "stochastic" | "stoch" => {
            let (k_period, k_slow, d_period) = if let Some(p) = params {
                let parts: Vec<&str> = p.split(',').collect();
                let k = parts.first().and_then(|s| s.parse().ok()).unwrap_or(14);
                let ks = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
                let d = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
                (k, ks, d)
            } else {
                (14, 3, 3)
            };
            Ok(Indicator::Stochastic {
                k_period,
                k_slow,
                d_period,
            })
        }
        "stochrsi" | "stochastic_rsi" => {
            let (rsi_period, stoch_period, k_period, d_period) = if let Some(p) = params {
                let parts: Vec<&str> = p.split(',').collect();
                let rsi_p = parts.first().and_then(|s| s.parse().ok()).unwrap_or(14);
                let stoch_p = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(14);
                let k_p = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
                let d_p = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(3);
                (rsi_p, stoch_p, k_p, d_p)
            } else {
                (14, 14, 3, 3)
            };
            Ok(Indicator::StochasticRsi {
                rsi_period,
                stoch_period,
                k_period,
                d_period,
            })
        }
        "cci" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
            Ok(Indicator::Cci(period))
        }
        "williams" | "williamsr" | "williams_r" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(14);
            Ok(Indicator::WilliamsR(period))
        }
        "roc" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(12);
            Ok(Indicator::Roc(period))
        }
        "momentum" | "mom" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(10);
            Ok(Indicator::Momentum(period))
        }
        "cmo" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(14);
            Ok(Indicator::Cmo(period))
        }
        "ao" | "awesome" | "awesome_oscillator" => {
            let (fast, slow) = if let Some(p) = params {
                let parts: Vec<&str> = p.split(',').collect();
                let f = parts.first().and_then(|s| s.parse().ok()).unwrap_or(5);
                let s = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(34);
                (f, s)
            } else {
                (5, 34)
            };
            Ok(Indicator::AwesomeOscillator { fast, slow })
        }
        "coppock" | "coppock_curve" => {
            let (wma_period, long_roc, short_roc) = if let Some(p) = params {
                let parts: Vec<&str> = p.split(',').collect();
                let wma = parts.first().and_then(|s| s.parse().ok()).unwrap_or(10);
                let long = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(14);
                let short = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(11);
                (wma, long, short)
            } else {
                (10, 14, 11)
            };
            Ok(Indicator::CoppockCurve {
                wma_period,
                long_roc,
                short_roc,
            })
        }

        // Trend Indicators
        "macd" => {
            let (fast, slow, signal) = if let Some(p) = params {
                let parts: Vec<&str> = p.split(',').collect();
                let fast = parts.first().and_then(|s| s.parse().ok()).unwrap_or(12);
                let slow = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(26);
                let signal = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(9);
                (fast, slow, signal)
            } else {
                (12, 26, 9)
            };
            Ok(Indicator::Macd { fast, slow, signal })
        }
        "adx" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(14);
            Ok(Indicator::Adx(period))
        }
        "aroon" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(25);
            Ok(Indicator::Aroon(period))
        }
        "supertrend" => {
            let (period, multiplier) = if let Some(p) = params {
                let parts: Vec<&str> = p.split(',').collect();
                let per = parts.first().and_then(|s| s.parse().ok()).unwrap_or(10);
                let mult = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(3.0);
                (per, mult)
            } else {
                (10, 3.0)
            };
            Ok(Indicator::Supertrend { period, multiplier })
        }
        "ichimoku" => {
            let (conversion, base, lagging, displacement) = if let Some(p) = params {
                let parts: Vec<&str> = p.split(',').collect();
                let conv = parts.first().and_then(|s| s.parse().ok()).unwrap_or(9);
                let base = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(26);
                let lag = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(52);
                let disp = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(26);
                (conv, base, lag, disp)
            } else {
                (9, 26, 52, 26)
            };
            Ok(Indicator::Ichimoku {
                conversion,
                base,
                lagging,
                displacement,
            })
        }
        "psar" | "parabolic_sar" => {
            let (step, max) = if let Some(p) = params {
                let parts: Vec<&str> = p.split(',').collect();
                let s = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0.02);
                let m = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.2);
                (s, m)
            } else {
                (0.02, 0.2)
            };
            Ok(Indicator::ParabolicSar { step, max })
        }
        "bbp" | "bull_bear_power" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(13);
            Ok(Indicator::BullBearPower(period))
        }
        "elder" | "elder_ray" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(13);
            Ok(Indicator::ElderRay(period))
        }

        // Volatility Indicators
        "bollinger" | "bb" | "bollinger_bands" => {
            let (period, std_dev) = if let Some(p) = params {
                let parts: Vec<&str> = p.split(',').collect();
                let period = parts.first().and_then(|s| s.parse().ok()).unwrap_or(20);
                let std_dev = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(2.0);
                (period, std_dev)
            } else {
                (20, 2.0)
            };
            Ok(Indicator::Bollinger { period, std_dev })
        }
        "atr" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(14);
            Ok(Indicator::Atr(period))
        }
        "tr" | "true_range" => Ok(Indicator::TrueRange),
        "keltner" | "keltner_channels" => {
            let (period, multiplier, atr_period) = if let Some(p) = params {
                let parts: Vec<&str> = p.split(',').collect();
                let per = parts.first().and_then(|s| s.parse().ok()).unwrap_or(20);
                let mult = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(2.0);
                let atr = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
                (per, mult, atr)
            } else {
                (20, 2.0, 10)
            };
            Ok(Indicator::KeltnerChannels {
                period,
                multiplier,
                atr_period,
            })
        }
        "donchian" | "donchian_channels" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
            Ok(Indicator::DonchianChannels(period))
        }
        "chop" | "choppiness" | "choppiness_index" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(14);
            Ok(Indicator::ChoppinessIndex(period))
        }

        // Volume Indicators
        "obv" => Ok(Indicator::Obv),
        "vwap" => Ok(Indicator::Vwap),
        "mfi" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(14);
            Ok(Indicator::Mfi(period))
        }
        "cmf" | "chaikin_money_flow" => {
            let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
            Ok(Indicator::Cmf(period))
        }
        "chaikin" | "chaikin_oscillator" => Ok(Indicator::ChaikinOscillator),
        "ad" | "accumulation_distribution" => Ok(Indicator::AccumulationDistribution),
        "bop" | "balance_of_power" => {
            let period = params.and_then(|p| p.parse().ok());
            Ok(Indicator::BalanceOfPower(period))
        }

        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Unknown indicator '{}'. Use the interactive TUI to browse available indicators.",
            name
        ))),
    }
}

fn parse_interval(s: &str) -> Result<Interval> {
    match s.to_lowercase().as_str() {
        "1m" => Ok(Interval::OneMinute),
        "5m" => Ok(Interval::FiveMinutes),
        "15m" => Ok(Interval::FifteenMinutes),
        "1h" => Ok(Interval::OneHour),
        "1d" => Ok(Interval::OneDay),
        "1wk" => Ok(Interval::OneWeek),
        "1mo" => Ok(Interval::OneMonth),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid interval '{}'. Valid: 1m, 5m, 15m, 1h, 1d, 1wk, 1mo",
            s
        ))),
    }
}

fn parse_range(s: &str) -> Result<TimeRange> {
    match s.to_lowercase().as_str() {
        "1d" => Ok(TimeRange::OneDay),
        "5d" => Ok(TimeRange::FiveDays),
        "1mo" => Ok(TimeRange::OneMonth),
        "3mo" => Ok(TimeRange::ThreeMonths),
        "6mo" => Ok(TimeRange::SixMonths),
        "1y" => Ok(TimeRange::OneYear),
        "2y" => Ok(TimeRange::TwoYears),
        "5y" => Ok(TimeRange::FiveYears),
        "10y" => Ok(TimeRange::TenYears),
        "ytd" => Ok(TimeRange::YearToDate),
        "max" => Ok(TimeRange::Max),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid range '{}'. Valid: 1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max",
            s
        ))),
    }
}

use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use finance_query::{Interval, Ticker, TimeRange};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Chart as RatatuiChart, Dataset, GraphType, Paragraph},
};
use serde::Serialize;
use std::io;
use tabled::Tabled;

#[derive(Parser)]
pub struct ChartArgs {
    /// Stock symbol to get chart data for
    #[arg(required = true)]
    symbol: String,

    /// Time interval (1m, 5m, 15m, 1h, 1d, 1wk, 1mo)
    #[arg(short, long, default_value = "1d")]
    interval: String,

    /// Time range (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    #[arg(short, long, default_value = "1mo")]
    range: String,

    /// Output format (chart, table, json, csv)
    #[arg(short, long, default_value = "chart")]
    output: String,

    /// Technical indicators to calculate (comma-separated)
    /// Available: sma, ema, rsi, macd, bollinger, atr, stochastic, adx, obv, vwap, cci, williamsr,
    /// stochrsi, psar, supertrend, mfi, ichimoku, donchian
    /// Examples: --indicators sma,rsi or --indicators "sma:20,rsi:14,stochrsi:14:14,supertrend:10:3.0"
    #[arg(long)]
    indicators: Option<String>,
}

// Basic OHLCV display without indicators
#[derive(Debug, Serialize, Tabled)]
struct CandleDisplayBasic {
    #[tabled(rename = "Date")]
    date: String,

    #[tabled(rename = "Open")]
    open: String,

    #[tabled(rename = "High")]
    high: String,

    #[tabled(rename = "Low")]
    low: String,

    #[tabled(rename = "Close")]
    close: String,

    #[tabled(rename = "Volume")]
    volume: String,
}

#[derive(Debug, Clone)]
enum IndicatorType {
    Sma(usize),
    Ema(usize),
    Rsi(usize),
    Macd(usize, usize, usize),
    Bollinger(usize, f64),
    Atr(usize),
    Stochastic(usize, usize), // k_period, d_period
    Adx(usize),
    Obv,
    Vwap,
    Cci(usize),
    WilliamsR(usize),
    StochasticRsi(usize, usize), // rsi_period, stoch_period
    ParabolicSar(f64, f64),      // acceleration, maximum
    Supertrend(usize, f64),      // period, multiplier
    Mfi(usize),
    Ichimoku,
    DonchianChannels(usize),
}

/// Result of calculating an indicator
#[derive(Debug, Clone)]
enum IndicatorResult {
    Single(Vec<Option<f64>>),
    Macd(finance_query::indicators::MacdResult),
    Bollinger(finance_query::indicators::BollingerBands),
    Stochastic(finance_query::indicators::StochasticResult),
    Supertrend(finance_query::indicators::SuperTrendResult),
    Ichimoku(finance_query::indicators::IchimokuResult),
    Donchian(finance_query::indicators::DonchianChannelsResult),
}

impl IndicatorType {
    /// Calculate this indicator using the chart data
    fn calculate(&self, chart: &finance_query::Chart) -> Result<IndicatorResult> {
        match self {
            Self::Sma(period) => Ok(IndicatorResult::Single(chart.sma(*period))),
            Self::Ema(period) => Ok(IndicatorResult::Single(chart.ema(*period))),
            Self::Rsi(period) => Ok(IndicatorResult::Single(chart.rsi(*period)?)),
            Self::Macd(fast, slow, signal) => {
                Ok(IndicatorResult::Macd(chart.macd(*fast, *slow, *signal)?))
            }
            Self::Bollinger(period, std_dev) => Ok(IndicatorResult::Bollinger(
                chart.bollinger_bands(*period, *std_dev)?,
            )),
            Self::Atr(period) => Ok(IndicatorResult::Single(chart.atr(*period)?)),
            Self::Stochastic(k_period, d_period) => Ok(IndicatorResult::Stochastic(
                finance_query::indicators::stochastic(
                    &chart.high_prices(),
                    &chart.low_prices(),
                    &chart.close_prices(),
                    *k_period,
                    *d_period,
                )?,
            )),
            Self::Adx(period) => Ok(IndicatorResult::Single(finance_query::indicators::adx(
                &chart.high_prices(),
                &chart.low_prices(),
                &chart.close_prices(),
                *period,
            )?)),
            Self::Obv => Ok(IndicatorResult::Single(finance_query::indicators::obv(
                &chart.close_prices(),
                &chart.volumes(),
            )?)),
            Self::Vwap => Ok(IndicatorResult::Single(finance_query::indicators::vwap(
                &chart.high_prices(),
                &chart.low_prices(),
                &chart.close_prices(),
                &chart.volumes(),
            )?)),
            Self::Cci(period) => Ok(IndicatorResult::Single(finance_query::indicators::cci(
                &chart.high_prices(),
                &chart.low_prices(),
                &chart.close_prices(),
                *period,
            )?)),
            Self::WilliamsR(period) => Ok(IndicatorResult::Single(
                finance_query::indicators::williams_r(
                    &chart.high_prices(),
                    &chart.low_prices(),
                    &chart.close_prices(),
                    *period,
                )?,
            )),
            Self::StochasticRsi(rsi_period, stoch_period) => Ok(IndicatorResult::Single(
                finance_query::indicators::stochastic_rsi(
                    &chart.close_prices(),
                    *rsi_period,
                    *stoch_period,
                )?,
            )),
            Self::ParabolicSar(acceleration, maximum) => Ok(IndicatorResult::Single(
                finance_query::indicators::parabolic_sar(
                    &chart.high_prices(),
                    &chart.low_prices(),
                    &chart.close_prices(),
                    *acceleration,
                    *maximum,
                )?,
            )),
            Self::Supertrend(period, multiplier) => Ok(IndicatorResult::Supertrend(
                finance_query::indicators::supertrend(
                    &chart.high_prices(),
                    &chart.low_prices(),
                    &chart.close_prices(),
                    *period,
                    *multiplier,
                )?,
            )),
            Self::Mfi(period) => Ok(IndicatorResult::Single(finance_query::indicators::mfi(
                &chart.high_prices(),
                &chart.low_prices(),
                &chart.close_prices(),
                &chart.volumes(),
                *period,
            )?)),
            Self::Ichimoku => Ok(IndicatorResult::Ichimoku(
                finance_query::indicators::ichimoku(
                    &chart.high_prices(),
                    &chart.low_prices(),
                    &chart.close_prices(),
                )?,
            )),
            Self::DonchianChannels(period) => Ok(IndicatorResult::Donchian(
                finance_query::indicators::donchian_channels(
                    &chart.high_prices(),
                    &chart.low_prices(),
                    *period,
                )?,
            )),
        }
    }
}

/// Format timestamp based on time range - uses local timezone
fn format_timestamp_for_range(timestamp: i64, range: TimeRange) -> String {
    use chrono::{Local, TimeZone};

    Local
        .timestamp_opt(timestamp, 0)
        .single()
        .map(|dt| {
            match range {
                // 1D: show time only
                TimeRange::OneDay => dt.format("%H:%M").to_string(),
                // 5D: show weekday and time
                TimeRange::FiveDays => dt.format("%a %H:%M").to_string(),
                // 1M-1Y: show month and day
                TimeRange::OneMonth
                | TimeRange::ThreeMonths
                | TimeRange::SixMonths
                | TimeRange::YearToDate
                | TimeRange::OneYear => dt.format("%b %d").to_string(),
                // 2Y+: show month and year
                TimeRange::TwoYears
                | TimeRange::FiveYears
                | TimeRange::TenYears
                | TimeRange::Max => dt.format("%b %Y").to_string(),
            }
        })
        .unwrap_or_else(|| "N/A".to_string())
}

async fn render_interactive_chart(
    symbol: &str,
    _initial_interval: Interval,
    initial_range: TimeRange,
) -> Result<()> {
    // Range options with appropriate intervals for each range
    let range_options: [(&str, TimeRange, Interval); 8] = [
        ("1D", TimeRange::OneDay, Interval::FiveMinutes),
        ("5D", TimeRange::FiveDays, Interval::FifteenMinutes),
        ("1M", TimeRange::OneMonth, Interval::OneDay),
        ("6M", TimeRange::SixMonths, Interval::OneDay),
        ("YTD", TimeRange::YearToDate, Interval::OneDay),
        ("1Y", TimeRange::OneYear, Interval::OneDay),
        ("5Y", TimeRange::FiveYears, Interval::OneWeek),
        ("Max", TimeRange::Max, Interval::OneWeek),
    ];

    let mut selected_range_idx = range_options
        .iter()
        .position(|(_, r, _)| std::mem::discriminant(r) == std::mem::discriminant(&initial_range))
        .unwrap_or(2); // Default to 1M

    let mut current_range = initial_range;
    let mut current_interval = range_options[selected_range_idx].2;
    let mut chart_data: Option<finance_query::Chart> = None;
    let mut loading = true;
    let mut error_msg: Option<String> = None;
    let mut focus_mode = false;
    let mut focus_index: usize = 0;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        // Fetch data if needed
        if loading {
            let ticker = Ticker::new(symbol).await?;
            match ticker.chart(current_interval, current_range).await {
                Ok(chart) => {
                    chart_data = Some(chart);
                    loading = false;
                    error_msg = None;
                }
                Err(e) => {
                    error_msg = Some(format!("Error: {}", e));
                    loading = false;
                }
            }
        }

        terminal.draw(|f| {
            let size = f.area();

            // Create layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4), // Header with range selector
                    Constraint::Min(0),    // Chart
                    Constraint::Length(3), // Footer with status and controls
                ])
                .split(size);

            // Header with symbol and range selector
            if let Some(ref chart) = chart_data {
                let first_price = chart.candles.first().map(|c| c.close).unwrap_or(0.0);
                let last_price = chart.candles.last().map(|c| c.close).unwrap_or(0.0);
                let is_up = last_price >= first_price;
                let percent_change = if first_price != 0.0 {
                    ((last_price - first_price) / first_price) * 100.0
                } else {
                    0.0
                };
                let header_color = if is_up { Color::Green } else { Color::Red };

                // Build range selector buttons
                let range_buttons: Vec<Span> = range_options
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, (label, _, _))| {
                        let is_selected = idx == selected_range_idx;
                        vec![
                            if is_selected {
                                Span::styled(
                                    format!(" {} ", label),
                                    Style::default()
                                        .bg(Color::Blue)
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD),
                                )
                            } else {
                                Span::styled(
                                    format!(" {} ", label),
                                    Style::default().fg(Color::DarkGray),
                                )
                            },
                            Span::raw(" "),
                        ]
                    })
                    .collect();

                let header_text = vec![
                    Line::from(vec![
                        Span::styled(
                            format!("{} ", symbol),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("${:.2}  ", last_price),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!(
                                "{}{:.2} ({}{:.2}%)",
                                if is_up { "+" } else { "" },
                                last_price - first_price,
                                if is_up { "+" } else { "" },
                                percent_change
                            ),
                            Style::default()
                                .fg(header_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(range_buttons),
                ];

                let header = Paragraph::new(header_text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                );
                f.render_widget(header, chunks[0]);

                // Extract close prices and timestamps
                let data: Vec<(f64, f64)> = chart
                    .candles
                    .iter()
                    .enumerate()
                    .map(|(i, candle)| (i as f64, candle.close))
                    .collect();

                if !data.is_empty() {
                    // Calculate min/max for axes
                    let min_price = data.iter().map(|(_, p)| *p).fold(f64::INFINITY, f64::min);
                    let max_price = data
                        .iter()
                        .map(|(_, p)| *p)
                        .fold(f64::NEG_INFINITY, f64::max);
                    let max_index = (data.len().saturating_sub(1)) as f64;

                    // Add padding (y_min can't go below 0 for prices)
                    let price_padding = (max_price - min_price) * 0.1;
                    let y_min = (min_price - price_padding).max(0.0);
                    let y_max = max_price + price_padding;

                    let line_color = if is_up { Color::Green } else { Color::Red };

                    // Clamp focus_index to valid range
                    if focus_index >= data.len() {
                        focus_index = data.len().saturating_sub(1);
                    }

                    // Create main dataset
                    let dataset = Dataset::default()
                        .marker(symbols::Marker::Braille)
                        .graph_type(GraphType::Line)
                        .style(Style::default().fg(line_color))
                        .data(&data);

                    // Create focus marker dataset (vertical line at focus point)
                    let focus_point_data: Vec<(f64, f64)> = if focus_mode {
                        let x = focus_index as f64;
                        vec![(x, y_min), (x, y_max)]
                    } else {
                        vec![]
                    };
                    let focus_dataset = Dataset::default()
                        .marker(symbols::Marker::Braille)
                        .graph_type(GraphType::Line)
                        .style(Style::default().fg(Color::Yellow))
                        .data(&focus_point_data);

                    // Get first and last timestamps for minimal x-axis context
                    let first_time = chart.candles.first().map(|c| c.timestamp).unwrap_or(0);
                    let last_time = chart.candles.last().map(|c| c.timestamp).unwrap_or(0);
                    let x_labels = vec![
                        Span::styled(
                            format_timestamp_for_range(first_time, current_range),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            format_timestamp_for_range(last_time, current_range),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ];

                    // Format price labels
                    let y_labels = vec![
                        Span::raw(format!(" {:.2}", y_min)),
                        Span::raw(format!(" {:.2}", (y_min + y_max) / 2.0)),
                        Span::raw(format!(" {:.2}", y_max)),
                    ];

                    // Build title - show focus info when in focus mode
                    let title = if focus_mode {
                        let focused_candle = &chart.candles[focus_index];
                        let time_str =
                            format_timestamp_for_range(focused_candle.timestamp, current_range);
                        Line::from(vec![
                            Span::styled(
                                format!(" {} ", symbol),
                                Style::default()
                                    .fg(Color::White)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!(
                                    "│ {} │ O:{:.2} H:{:.2} L:{:.2} C:{:.2} ",
                                    time_str,
                                    focused_candle.open,
                                    focused_candle.high,
                                    focused_candle.low,
                                    focused_candle.close
                                ),
                                Style::default().fg(Color::Yellow),
                            ),
                        ])
                    } else {
                        Line::from(Span::styled(
                            format!(" {} ", symbol),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ))
                    };

                    // Create chart with both datasets
                    let datasets = if focus_mode {
                        vec![dataset, focus_dataset]
                    } else {
                        vec![dataset]
                    };

                    let chart_widget = RatatuiChart::new(datasets)
                        .block(
                            Block::default()
                                .title(title)
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(Color::DarkGray)),
                        )
                        .x_axis(
                            ratatui::widgets::Axis::default()
                                .style(Style::default().fg(Color::DarkGray))
                                .bounds([0.0, max_index])
                                .labels(x_labels),
                        )
                        .y_axis(
                            ratatui::widgets::Axis::default()
                                .style(Style::default().fg(Color::Gray))
                                .bounds([y_min, y_max])
                                .labels(y_labels),
                        );

                    f.render_widget(chart_widget, chunks[1]);
                }
            } else if loading {
                let loading_text = Paragraph::new("Loading chart data...").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                );
                f.render_widget(loading_text, chunks[1]);
            } else if let Some(ref err) = error_msg {
                let error_text = Paragraph::new(err.as_str())
                    .style(Style::default().fg(Color::Red))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Red)),
                    );
                f.render_widget(error_text, chunks[1]);
            }

            // Footer with status and controls
            let interval_str = match current_interval {
                Interval::OneMinute => "1m",
                Interval::FiveMinutes => "5m",
                Interval::FifteenMinutes => "15m",
                Interval::ThirtyMinutes => "30m",
                Interval::OneHour => "1h",
                Interval::OneDay => "1d",
                Interval::OneWeek => "1wk",
                Interval::OneMonth => "1mo",
                Interval::ThreeMonths => "3mo",
            };
            let status_text = if loading {
                Span::styled(" Loading...", Style::default().fg(Color::Yellow))
            } else if let Some(ref chart) = chart_data {
                Span::styled(
                    format!(" {} | {} pts", interval_str, chart.candles.len()),
                    Style::default().fg(Color::Cyan),
                )
            } else {
                Span::raw("")
            };

            let footer_spans = if focus_mode {
                vec![
                    status_text,
                    Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("←/→", Style::default().fg(Color::Yellow)),
                    Span::styled(":trace  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Enter/Esc", Style::default().fg(Color::White)),
                    Span::styled(":exit  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("q", Style::default().fg(Color::Red)),
                    Span::styled(":quit", Style::default().fg(Color::DarkGray)),
                ]
            } else {
                vec![
                    status_text,
                    Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Tab/←/→", Style::default().fg(Color::White)),
                    Span::styled(":range  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Enter", Style::default().fg(Color::Yellow)),
                    Span::styled(":trace  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("q", Style::default().fg(Color::Red)),
                    Span::styled(":quit", Style::default().fg(Color::DarkGray)),
                ]
            };

            let footer = Paragraph::new(Line::from(footer_spans))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            f.render_widget(footer, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            let data_len = chart_data.as_ref().map(|c| c.candles.len()).unwrap_or(0);

            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Esc => {
                    if focus_mode {
                        focus_mode = false;
                    } else {
                        break;
                    }
                }
                KeyCode::Enter => {
                    if focus_mode {
                        focus_mode = false;
                    } else if data_len > 0 {
                        focus_mode = true;
                        focus_index = data_len / 2; // Start in the middle
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if focus_mode {
                        focus_index = focus_index.saturating_sub(1);
                    } else if selected_range_idx > 0 {
                        selected_range_idx -= 1;
                        current_range = range_options[selected_range_idx].1;
                        current_interval = range_options[selected_range_idx].2;
                        loading = true;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if focus_mode {
                        if focus_index < data_len.saturating_sub(1) {
                            focus_index += 1;
                        }
                    } else if selected_range_idx < range_options.len() - 1 {
                        selected_range_idx += 1;
                        current_range = range_options[selected_range_idx].1;
                        current_interval = range_options[selected_range_idx].2;
                        loading = true;
                    }
                }
                KeyCode::BackTab => {
                    if !focus_mode && selected_range_idx > 0 {
                        selected_range_idx -= 1;
                        current_range = range_options[selected_range_idx].1;
                        current_interval = range_options[selected_range_idx].2;
                        loading = true;
                    }
                }
                KeyCode::Tab => {
                    if !focus_mode && selected_range_idx < range_options.len() - 1 {
                        selected_range_idx += 1;
                        current_range = range_options[selected_range_idx].1;
                        current_interval = range_options[selected_range_idx].2;
                        loading = true;
                    }
                }
                KeyCode::Home => {
                    if focus_mode {
                        focus_index = 0;
                    }
                }
                KeyCode::End => {
                    if focus_mode {
                        focus_index = data_len.saturating_sub(1);
                    }
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

pub async fn execute(args: ChartArgs) -> Result<()> {
    // Parse interval
    let interval = parse_interval(&args.interval)?;

    // Parse range
    let range = parse_range(&args.range)?;

    // Interactive mode is default for chart output
    match args.output.to_lowercase().as_str() {
        "chart" => {
            return render_interactive_chart(&args.symbol, interval, range).await;
        }
        "table" | "json" | "csv" => {}
        _ => {
            return Err(crate::error::CliError::InvalidArgument(format!(
                "Invalid output format '{}'. Valid: chart, table, json, csv",
                args.output
            )));
        }
    }

    // Non-interactive mode for table/json/csv
    let ticker = Ticker::new(&args.symbol).await?;
    let chart = ticker.chart(interval, range).await?;

    // Use table/json/csv output
    let format = OutputFormat::from_str(&args.output)?;

    // Parse indicators
    let indicators = if let Some(ref ind_str) = args.indicators {
        parse_indicators(ind_str)?
    } else {
        Vec::new()
    };

    // Calculate all indicators using enum dispatch pattern
    let indicator_results: Vec<(IndicatorType, IndicatorResult)> = indicators
        .iter()
        .map(|indicator| {
            let result = indicator.calculate(&chart)?;
            Ok((indicator.clone(), result))
        })
        .collect::<Result<Vec<_>>>()?;

    // Build display table with only requested indicator columns
    if indicators.is_empty() {
        // No indicators - use basic display
        let mut candles = Vec::new();
        for candle in chart.candles.iter() {
            let date = chrono::DateTime::from_timestamp(candle.timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "N/A".to_string());

            candles.push(CandleDisplayBasic {
                date,
                open: format!("{:.2}", candle.open),
                high: format!("{:.2}", candle.high),
                low: format!("{:.2}", candle.low),
                close: format!("{:.2}", candle.close),
                volume: candle.volume.to_string(),
            });
        }

        output::print_many(&candles, format)?;
    } else {
        // Build dynamic table with only requested indicators
        build_dynamic_indicator_table(&chart, &indicator_results, format)?;
    }

    Ok(())
}

fn build_dynamic_indicator_table(
    chart: &finance_query::Chart,
    indicator_results: &[(IndicatorType, IndicatorResult)],
    format: OutputFormat,
) -> Result<()> {
    // Build rows as Vec<Vec<String>> for dynamic columns
    let mut rows: Vec<Vec<String>> = Vec::new();

    // Build header row
    let mut header = vec![
        "Date".to_string(),
        "Open".to_string(),
        "High".to_string(),
        "Low".to_string(),
        "Close".to_string(),
        "Volume".to_string(),
    ];

    // Add indicator headers based on what was requested
    for (indicator, _) in indicator_results {
        match indicator {
            IndicatorType::Sma(_) => header.push("SMA".to_string()),
            IndicatorType::Ema(_) => header.push("EMA".to_string()),
            IndicatorType::Rsi(_) => header.push("RSI".to_string()),
            IndicatorType::Macd(_, _, _) => header.push("MACD".to_string()),
            IndicatorType::Bollinger(_, _) => {
                header.push("BB_Upper".to_string());
                header.push("BB_Middle".to_string());
                header.push("BB_Lower".to_string());
            }
            IndicatorType::Atr(_) => header.push("ATR".to_string()),
            IndicatorType::Stochastic(_, _) => {
                header.push("Stoch_%K".to_string());
                header.push("Stoch_%D".to_string());
            }
            IndicatorType::Adx(_) => header.push("ADX".to_string()),
            IndicatorType::Obv => header.push("OBV".to_string()),
            IndicatorType::Vwap => header.push("VWAP".to_string()),
            IndicatorType::Cci(_) => header.push("CCI".to_string()),
            IndicatorType::WilliamsR(_) => header.push("Williams_%R".to_string()),
            IndicatorType::StochasticRsi(_, _) => header.push("Stoch_RSI".to_string()),
            IndicatorType::ParabolicSar(_, _) => header.push("PSAR".to_string()),
            IndicatorType::Supertrend(_, _) => header.push("SuperTrend".to_string()),
            IndicatorType::Mfi(_) => header.push("MFI".to_string()),
            IndicatorType::Ichimoku => {
                header.push("Ichimoku_Conversion".to_string());
                header.push("Ichimoku_Base".to_string());
                header.push("Ichimoku_SpanA".to_string());
                header.push("Ichimoku_SpanB".to_string());
                header.push("Ichimoku_Lagging".to_string());
            }
            IndicatorType::DonchianChannels(_) => {
                header.push("Donchian_Upper".to_string());
                header.push("Donchian_Middle".to_string());
                header.push("Donchian_Lower".to_string());
            }
        }
    }

    rows.push(header);

    // Build data rows
    for (idx, candle) in chart.candles.iter().enumerate() {
        let date = chrono::DateTime::from_timestamp(candle.timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let mut row = vec![
            date,
            format!("{:.2}", candle.open),
            format!("{:.2}", candle.high),
            format!("{:.2}", candle.low),
            format!("{:.2}", candle.close),
            candle.volume.to_string(),
        ];

        // Add indicator values based on what was requested
        for (_, result) in indicator_results {
            match result {
                IndicatorResult::Single(values) => {
                    let val = values
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    row.push(val);
                }
                IndicatorResult::Macd(macd) => {
                    let val = macd
                        .macd_line
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    row.push(val);
                }
                IndicatorResult::Bollinger(bb) => {
                    let upper = bb
                        .upper
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    let middle = bb
                        .middle
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    let lower = bb
                        .lower
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    row.push(upper);
                    row.push(middle);
                    row.push(lower);
                }
                IndicatorResult::Stochastic(stoch) => {
                    let k_val = stoch
                        .k
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    let d_val = stoch
                        .d
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    row.push(k_val);
                    row.push(d_val);
                }
                IndicatorResult::Supertrend(st) => {
                    let val = st
                        .value
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    row.push(val);
                }
                IndicatorResult::Ichimoku(ich) => {
                    let conversion = ich
                        .conversion_line
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    let base = ich
                        .base_line
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    let span_a = ich
                        .leading_span_a
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    let span_b = ich
                        .leading_span_b
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    let lagging = ich
                        .lagging_span
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    row.push(conversion);
                    row.push(base);
                    row.push(span_a);
                    row.push(span_b);
                    row.push(lagging);
                }
                IndicatorResult::Donchian(don) => {
                    let upper = don
                        .upper
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    let middle = don
                        .middle
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    let lower = don
                        .lower
                        .get(idx)
                        .and_then(|&val| val.map(|v| format!("{:.2}", v)))
                        .unwrap_or_else(|| "-".to_string());
                    row.push(upper);
                    row.push(middle);
                    row.push(lower);
                }
            }
        }

        rows.push(row);
    }

    // Print based on format
    match format {
        OutputFormat::Table => {
            let mut builder = tabled::builder::Builder::default();
            for row in rows {
                builder.push_record(row);
            }
            let table = builder
                .build()
                .with(tabled::settings::Style::rounded())
                .to_string();
            println!("{}", table);
        }
        OutputFormat::Json => {
            // Convert rows to JSON
            let header = &rows[0];
            let data_rows = &rows[1..];
            let json: Vec<serde_json::Map<String, serde_json::Value>> = data_rows
                .iter()
                .map(|row| {
                    let mut map = serde_json::Map::new();
                    for (i, col) in row.iter().enumerate() {
                        map.insert(header[i].clone(), serde_json::Value::String(col.clone()));
                    }
                    map
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Csv => {
            for row in rows {
                println!("{}", row.join(","));
            }
        }
    }

    Ok(())
}

fn parse_indicators(s: &str) -> Result<Vec<IndicatorType>> {
    let mut indicators = Vec::new();

    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let (name, params) = match part.split_once(':') {
            Some((n, p)) => (n.trim(), Some(p.trim())),
            None => (part, None),
        };

        match name.to_lowercase().as_str() {
            "sma" => {
                let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
                indicators.push(IndicatorType::Sma(period));
            }
            "ema" => {
                let period = params.and_then(|p| p.parse().ok()).unwrap_or(12);
                indicators.push(IndicatorType::Ema(period));
            }
            "rsi" => {
                let period = params.and_then(|p| p.parse().ok()).unwrap_or(14);
                indicators.push(IndicatorType::Rsi(period));
            }
            "macd" => {
                // Parse MACD as "macd:12,26,9" or use defaults
                let (fast, slow, signal) = if let Some(p) = params {
                    let parts: Vec<&str> = p.split(',').collect();
                    let fast = parts.first().and_then(|s| s.parse().ok()).unwrap_or(12);
                    let slow = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(26);
                    let signal = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(9);
                    (fast, slow, signal)
                } else {
                    (12, 26, 9)
                };
                indicators.push(IndicatorType::Macd(fast, slow, signal));
            }
            "bollinger" | "bb" => {
                // Parse as "bollinger:20,2.0" or use defaults
                let (period, std_dev) = if let Some(p) = params {
                    let parts: Vec<&str> = p.split(',').collect();
                    let period = parts.first().and_then(|s| s.parse().ok()).unwrap_or(20);
                    let std_dev = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(2.0);
                    (period, std_dev)
                } else {
                    (20, 2.0)
                };
                indicators.push(IndicatorType::Bollinger(period, std_dev));
            }
            "atr" => {
                let period = params.and_then(|p| p.parse().ok()).unwrap_or(14);
                indicators.push(IndicatorType::Atr(period));
            }
            "stochastic" | "stoch" => {
                // Parse as "stochastic:14:3" or use defaults
                let (k_period, d_period) = if let Some(p) = params {
                    let parts: Vec<&str> = p.split(':').collect();
                    let k = parts.first().and_then(|s| s.parse().ok()).unwrap_or(14);
                    let d = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
                    (k, d)
                } else {
                    (14, 3)
                };
                indicators.push(IndicatorType::Stochastic(k_period, d_period));
            }
            "adx" => {
                let period = params.and_then(|p| p.parse().ok()).unwrap_or(14);
                indicators.push(IndicatorType::Adx(period));
            }
            "obv" => {
                indicators.push(IndicatorType::Obv);
            }
            "vwap" => {
                indicators.push(IndicatorType::Vwap);
            }
            "cci" => {
                let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
                indicators.push(IndicatorType::Cci(period));
            }
            "williamsr" | "williams" | "willr" => {
                let period = params.and_then(|p| p.parse().ok()).unwrap_or(14);
                indicators.push(IndicatorType::WilliamsR(period));
            }
            "stochrsi" | "stochastic_rsi" => {
                // Parse as "stochrsi:14:14" or use defaults
                let (rsi_period, stoch_period) = if let Some(p) = params {
                    let parts: Vec<&str> = p.split(':').collect();
                    let r = parts.first().and_then(|s| s.parse().ok()).unwrap_or(14);
                    let s = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(14);
                    (r, s)
                } else {
                    (14, 14)
                };
                indicators.push(IndicatorType::StochasticRsi(rsi_period, stoch_period));
            }
            "psar" | "parabolicsar" | "parabolic_sar" => {
                // Parse as "psar:0.02:0.2" or use defaults
                let (acceleration, maximum) = if let Some(p) = params {
                    let parts: Vec<&str> = p.split(':').collect();
                    let a = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0.02);
                    let m = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.2);
                    (a, m)
                } else {
                    (0.02, 0.2)
                };
                indicators.push(IndicatorType::ParabolicSar(acceleration, maximum));
            }
            "supertrend" | "st" => {
                // Parse as "supertrend:10:3.0" or use defaults
                let (period, multiplier) = if let Some(p) = params {
                    let parts: Vec<&str> = p.split(':').collect();
                    let period = parts.first().and_then(|s| s.parse().ok()).unwrap_or(10);
                    let mult = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(3.0);
                    (period, mult)
                } else {
                    (10, 3.0)
                };
                indicators.push(IndicatorType::Supertrend(period, multiplier));
            }
            "mfi" => {
                let period = params.and_then(|p| p.parse().ok()).unwrap_or(14);
                indicators.push(IndicatorType::Mfi(period));
            }
            "ichimoku" => {
                indicators.push(IndicatorType::Ichimoku);
            }
            "donchian" | "donchian_channels" => {
                let period = params.and_then(|p| p.parse().ok()).unwrap_or(20);
                indicators.push(IndicatorType::DonchianChannels(period));
            }
            _ => {
                return Err(crate::error::CliError::InvalidArgument(format!(
                    "Unknown indicator '{}'. Available: sma, ema, rsi, macd, bollinger, atr, stochastic, adx, obv, vwap, cci, williamsr, stochrsi, psar, supertrend, mfi, ichimoku, donchian",
                    name
                )));
            }
        }
    }

    Ok(indicators)
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

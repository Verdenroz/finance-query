use crate::error::Result;
use crate::output;
use clap::Parser;
use colored::Colorize;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use finance_query::{Frequency, StatementType, Ticker};
use std::collections::HashMap;
use std::io;
use std::time::Duration;

#[derive(Parser)]
pub struct FinancialsArgs {
    /// Stock symbol to get financials for
    #[arg(required = true)]
    symbol: String,

    /// Type of financial statement
    #[arg(short = 't', long, default_value = "income")]
    statement_type: String,

    /// Frequency (annual or quarterly)
    #[arg(short = 'p', long, default_value = "annual")]
    period: String,

    /// Number of metrics to display (0 for all, default shows all)
    #[arg(short, long, default_value = "0")]
    limit: usize,

    /// Show specific date/period (1 = most recent, 2 = second most recent, etc.)
    #[arg(short = 'd', long, default_value = "1")]
    date: usize,

    /// Show all periods at once (horizontal layout)
    #[arg(short = 'a', long)]
    all: bool,
}

pub async fn execute(args: FinancialsArgs) -> Result<()> {
    let statement_type = parse_statement_type(&args.statement_type)?;
    let frequency = parse_frequency(&args.period)?;

    let ticker = Ticker::new(&args.symbol).await?;
    let financials = ticker.financials(statement_type, frequency).await?;

    if financials.statement.is_empty() {
        output::print_info("No financial data available");
        return Ok(());
    }

    // Get all dates across all metrics and sort them
    let mut all_dates: Vec<String> = financials
        .statement
        .values()
        .flat_map(|dates| dates.keys().cloned())
        .collect();
    all_dates.sort_by(|a, b| b.cmp(a)); // Descending (newest first)
    all_dates.dedup();

    if all_dates.is_empty() {
        output::print_info("No financial data available");
        return Ok(());
    }

    // Sort metrics by name for consistent display
    let mut metrics: Vec<(&String, &HashMap<String, f64>)> = financials.statement.iter().collect();
    metrics.sort_by_key(|(name, _)| *name);

    // Apply limit
    let display_metrics: Vec<(&String, &HashMap<String, f64>)> = if args.limit == 0 {
        metrics
    } else {
        metrics.into_iter().take(args.limit).collect()
    };

    if args.all {
        // Horizontal layout showing all periods (original behavior but narrower)
        let display_dates: Vec<String> = all_dates.iter().take(5).cloned().collect();
        print_horizontal_layout(
            &financials.symbol,
            &financials.frequency.to_string(),
            &financials.statement_type.to_string(),
            &display_metrics,
            &display_dates,
        );

        if args.limit > 0 && financials.statement.len() > args.limit {
            println!(
                "\n{} {} more metrics (use --limit 0 to show all)",
                "...".dimmed(),
                financials.statement.len() - args.limit
            );
        }
    } else {
        // Interactive vertical layout with keyboard navigation
        let start_index = args.date.saturating_sub(1).min(all_dates.len() - 1);
        run_interactive_view(InteractiveViewConfig {
            symbol: &financials.symbol,
            frequency: &financials.frequency.to_string(),
            statement_type: &financials.statement_type.to_string(),
            metrics: &display_metrics,
            dates: &all_dates,
            start_index,
        })?;
    }

    Ok(())
}

struct InteractiveViewConfig<'a> {
    symbol: &'a str,
    frequency: &'a str,
    statement_type: &'a str,
    metrics: &'a [(&'a String, &'a HashMap<String, f64>)],
    dates: &'a [String],
    start_index: usize,
}

fn run_interactive_view(config: InteractiveViewConfig) -> Result<()> {
    use ratatui::{
        Terminal,
        backend::CrosstermBackend,
        layout::Constraint,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{
            Block, Borders, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState,
        },
    };

    let InteractiveViewConfig {
        symbol,
        frequency,
        statement_type,
        metrics,
        dates,
        start_index,
    } = config;

    let mut stdout = io::stdout();
    let mut current_index = start_index;
    let mut table_state = TableState::default();
    table_state.select(Some(0));

    // Setup terminal
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let date = &dates[current_index];

        terminal.draw(|frame| {
            let area = frame.area();

            // Build table rows
            let rows: Vec<Row> = metrics
                .iter()
                .map(|(name, values)| {
                    let display_name = humanize_metric_name(name);
                    let value = values
                        .get(date)
                        .map(|v| format_large_number(*v))
                        .unwrap_or_else(|| "-".to_string());
                    Row::new(vec![display_name, value])
                })
                .collect();

            let widths = [Constraint::Percentage(60), Constraint::Percentage(40)];

            let title = format!(
                " {} {} for {} — Period {}/{} ({} metrics) ",
                frequency.to_lowercase(),
                statement_type,
                symbol,
                current_index + 1,
                dates.len(),
                metrics.len()
            );

            let table = Table::new(rows, widths)
                .block(
                    Block::default()
                        .title(title)
                        .title_bottom(Line::from(vec![
                            Span::raw(" "),
                            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
                            Span::raw(" scroll  "),
                            Span::styled("←/→", Style::default().fg(Color::Yellow)),
                            Span::raw(" period  "),
                            Span::styled("q", Style::default().fg(Color::Yellow)),
                            Span::raw(" quit "),
                        ]))
                        .borders(Borders::ALL),
                )
                .header(
                    Row::new(vec![date.to_string(), "Value".to_string()])
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .row_highlight_style(Style::default().bg(Color::DarkGray));

            frame.render_stateful_widget(table, area, &mut table_state);

            // Render scrollbar
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            let mut scrollbar_state =
                ScrollbarState::new(metrics.len()).position(table_state.selected().unwrap_or(0));
            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Up | KeyCode::Char('k') => {
                    let i = table_state.selected().unwrap_or(0);
                    if i > 0 {
                        table_state.select(Some(i - 1));
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let i = table_state.selected().unwrap_or(0);
                    if i < metrics.len().saturating_sub(1) {
                        table_state.select(Some(i + 1));
                    }
                }
                KeyCode::PageUp => {
                    let i = table_state.selected().unwrap_or(0);
                    table_state.select(Some(i.saturating_sub(10)));
                }
                KeyCode::PageDown => {
                    let i = table_state.selected().unwrap_or(0);
                    table_state.select(Some((i + 10).min(metrics.len().saturating_sub(1))));
                }
                KeyCode::Home => {
                    table_state.select(Some(0));
                }
                KeyCode::End => {
                    table_state.select(Some(metrics.len().saturating_sub(1)));
                }
                KeyCode::Left | KeyCode::Char('h') | KeyCode::BackTab => {
                    // Wrap around to last period if at first
                    if current_index > 0 {
                        current_index -= 1;
                    } else {
                        current_index = dates.len() - 1;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
                    // Wrap around to first period if at last
                    if current_index < dates.len() - 1 {
                        current_index += 1;
                    } else {
                        current_index = 0;
                    }
                }
                KeyCode::Char('g') => table_state.select(Some(0)),
                KeyCode::Char('G') => table_state.select(Some(metrics.len().saturating_sub(1))),
                _ => {}
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn print_horizontal_layout(
    symbol: &str,
    frequency: &str,
    statement_type: &str,
    metrics: &[(&String, &HashMap<String, f64>)],
    dates: &[String],
) {
    println!(
        "{} {} {} for {}",
        "✓".green(),
        frequency.to_lowercase(),
        statement_type,
        symbol.cyan().bold()
    );
    println!();

    let metric_width = 32;
    let value_width = 12;

    // Print header
    print!("{:<width$}", "Metric", width = metric_width);
    for date in dates {
        // Show just year for compactness
        let short = date.split('-').next().unwrap_or(date);
        print!("{:>width$}", short, width = value_width);
    }
    println!();
    println!("{}", "─".repeat(metric_width + (dates.len() * value_width)));

    // Print data rows
    for (metric_name, date_values) in metrics {
        let display_name = humanize_metric_name(metric_name);
        let truncated = if display_name.len() > metric_width - 1 {
            format!("{}…", &display_name[..metric_width - 2])
        } else {
            display_name
        };

        print!("{:<width$}", truncated, width = metric_width);
        for date in dates {
            if let Some(value) = date_values.get(date) {
                print!(
                    "{:>width$}",
                    format_large_number(*value),
                    width = value_width
                );
            } else {
                print!("{:>width$}", "-", width = value_width);
            }
        }
        println!();
    }
}

/// Convert CamelCase metric names to human-readable format
fn humanize_metric_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len() + 10);
    for (i, c) in name.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            result.push(' ');
        }
        result.push(c);
    }
    result
}

fn parse_statement_type(s: &str) -> Result<StatementType> {
    match s.to_lowercase().as_str() {
        "income" | "income-statement" => Ok(StatementType::Income),
        "balance" | "balance-sheet" => Ok(StatementType::Balance),
        "cash" | "cashflow" | "cash-flow" => Ok(StatementType::CashFlow),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid statement type '{}'. Valid types: income, balance, cash-flow",
            s
        ))),
    }
}

fn parse_frequency(s: &str) -> Result<Frequency> {
    match s.to_lowercase().as_str() {
        "annual" | "yearly" | "year" => Ok(Frequency::Annual),
        "quarterly" | "quarter" | "q" => Ok(Frequency::Quarterly),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid frequency '{}'. Valid frequencies: annual, quarterly",
            s
        ))),
    }
}

/// Format large numbers with K, M, B, T suffixes
fn format_large_number(n: f64) -> String {
    let abs = n.abs();
    let sign = if n < 0.0 { "-" } else { "" };

    if abs >= 1_000_000_000_000.0 {
        format!("{}${:.2}T", sign, abs / 1_000_000_000_000.0)
    } else if abs >= 1_000_000_000.0 {
        format!("{}${:.2}B", sign, abs / 1_000_000_000.0)
    } else if abs >= 1_000_000.0 {
        format!("{}${:.2}M", sign, abs / 1_000_000.0)
    } else if abs >= 1_000.0 {
        format!("{}${:.2}K", sign, abs / 1_000.0)
    } else {
        format!("{}${:.2}", sign, abs)
    }
}

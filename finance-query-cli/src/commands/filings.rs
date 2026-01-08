use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use finance_query::Ticker;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use serde::Serialize;
use std::io;

#[derive(Parser)]
pub struct FilingsArgs {
    /// Stock symbol to get SEC filings for
    #[arg(required = true)]
    symbol: String,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,

    /// Filter by filing type (e.g., 10-K, 10-Q, 8-K)
    #[arg(short = 't', long)]
    filing_type: Option<String>,

    /// Maximum number of filings to show
    #[arg(short, long, default_value = "50")]
    limit: usize,
}

/// Local struct to hold filing data for display
#[derive(Debug, Clone)]
struct FilingDisplay {
    filing_type: Option<String>,
    title: Option<String>,
    date: Option<String>,
    edgar_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct FilingJson {
    symbol: String,
    filing_type: Option<String>,
    title: Option<String>,
    date: Option<String>,
    edgar_url: Option<String>,
}

pub async fn execute(args: FilingsArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;
    let ticker = Ticker::new(&args.symbol).await?;

    let filings_data = ticker.sec_filings().await?;

    if filings_data.is_none() {
        output::print_info(&format!(
            "No SEC filings data available for {}",
            args.symbol
        ));
        return Ok(());
    }

    let sec_filings = filings_data.unwrap();
    let mut filings = sec_filings.filings;

    // Filter by type if specified
    if let Some(ref filter_type) = args.filing_type {
        let filter_upper = filter_type.to_uppercase();
        filings.retain(|f| {
            f.filing_type
                .as_ref()
                .map(|t| t.to_uppercase().contains(&filter_upper))
                .unwrap_or(false)
        });
    }

    // Limit the results
    filings.truncate(args.limit);

    if filings.is_empty() {
        let msg = if args.filing_type.is_some() {
            format!(
                "No {} filings found for {}",
                args.filing_type.as_ref().unwrap(),
                args.symbol
            )
        } else {
            format!("No SEC filings found for {}", args.symbol)
        };
        output::print_info(&msg);
        return Ok(());
    }

    // For JSON/CSV output
    if format != OutputFormat::Table {
        let filings_json: Vec<FilingJson> = filings
            .iter()
            .map(|f| FilingJson {
                symbol: args.symbol.clone(),
                filing_type: f.filing_type.clone(),
                title: f.title.clone(),
                date: f.date.clone(),
                edgar_url: f.edgar_url.clone(),
            })
            .collect();

        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&filings_json)?);
            }
            OutputFormat::Csv => {
                println!("symbol,type,title,date,url");
                for filing in &filings_json {
                    println!(
                        "{},{},{},{},{}",
                        filing.symbol,
                        filing.filing_type.as_deref().unwrap_or("N/A"),
                        escape_csv(filing.title.as_deref().unwrap_or("N/A")),
                        filing.date.as_deref().unwrap_or("N/A"),
                        filing.edgar_url.as_deref().unwrap_or("N/A")
                    );
                }
            }
            _ => {}
        }
        return Ok(());
    }

    // Convert to display structs for the TUI
    let filings_display: Vec<FilingDisplay> = filings
        .iter()
        .map(|f| FilingDisplay {
            filing_type: f.filing_type.clone(),
            title: f.title.clone(),
            date: f.date.clone(),
            edgar_url: f.edgar_url.clone(),
        })
        .collect();

    // Table output - use interactive TUI
    render_interactive_filings(
        &filings_display,
        &format!("SEC Filings: {}", args.symbol.to_uppercase()),
    )?;

    Ok(())
}

fn render_interactive_filings(filings: &[FilingDisplay], title: &str) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Ensure terminal cleanup on early exit
    let cleanup = || -> Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    };

    let result = (|| -> Result<()> {
        let mut selected_index: usize = 0;
        let mut scroll_offset: usize = 0;

        loop {
            terminal.draw(|f| {
                let size = f.area();

                // Create layout
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // Header
                        Constraint::Min(0),    // Filings list
                        Constraint::Length(3), // Footer
                    ])
                    .split(size);

                // Header
                let header_text = vec![
                    Line::from(vec![Span::styled(
                        title,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![Span::styled(
                        format!("Filing {} of {}", selected_index + 1, filings.len()),
                        Style::default().fg(Color::DarkGray),
                    )]),
                ];
                let header = Paragraph::new(header_text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                );
                f.render_widget(header, chunks[0]);

                // Calculate visible area (each filing takes 3 lines)
                let list_height = chunks[1].height.saturating_sub(2) as usize;
                let items_per_page = list_height / 3;

                // Adjust scroll offset to keep selected item visible
                if selected_index < scroll_offset {
                    scroll_offset = selected_index;
                } else if selected_index >= scroll_offset + items_per_page {
                    scroll_offset = selected_index.saturating_sub(items_per_page - 1);
                }

                // Filings list
                let items: Vec<ListItem> = filings
                    .iter()
                    .enumerate()
                    .skip(scroll_offset)
                    .take(items_per_page)
                    .map(|(idx, filing)| {
                        let is_selected = idx == selected_index;

                        let style = if is_selected {
                            Style::default().bg(Color::DarkGray).fg(Color::White)
                        } else {
                            Style::default()
                        };

                        let filing_type = filing.filing_type.as_deref().unwrap_or("N/A");
                        let type_color = match filing_type {
                            "10-K" => Color::Green,
                            "10-Q" => Color::Cyan,
                            "8-K" => Color::Yellow,
                            _ => Color::White,
                        };

                        let type_style = if is_selected {
                            Style::default()
                                .bg(Color::DarkGray)
                                .fg(type_color)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(type_color).add_modifier(Modifier::BOLD)
                        };

                        let title_text = filing.title.as_deref().unwrap_or("No title");
                        let date = filing.date.as_deref().unwrap_or("N/A");

                        let content = vec![
                            Line::from(vec![
                                Span::styled(format!("[{}] ", filing_type), type_style),
                                Span::styled(title_text, style),
                            ]),
                            Line::from(vec![
                                Span::styled("Filed: ", style.fg(Color::DarkGray)),
                                Span::styled(date, style.fg(Color::Yellow)),
                            ]),
                            Line::from(""),
                        ];

                        ListItem::new(content).style(style)
                    })
                    .collect();

                let list = List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title("Filings"),
                );
                f.render_widget(list, chunks[1]);

                // Footer with help text
                let footer = Paragraph::new(vec![Line::from(vec![
                    Span::styled(
                        "↑/↓",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" Navigate  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        "Enter",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" Open in Edgar  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        "q/Esc/Ctrl+C",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" Quit", Style::default().fg(Color::DarkGray)),
                ])])
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
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        selected_index = selected_index.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected_index < filings.len() - 1 {
                            selected_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        // Open selected filing in browser
                        if let Some(filing) = filings.get(selected_index)
                            && let Some(url) = &filing.edgar_url
                        {
                            // Disable raw mode temporarily to run the command
                            disable_raw_mode()?;
                            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                            // Open URL in browser
                            let _ = open_url(url);

                            // Re-enable raw mode
                            execute!(io::stdout(), EnterAlternateScreen)?;
                            enable_raw_mode()?;

                            // Clear and redraw
                            terminal.clear()?;
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    })();

    // Always cleanup terminal
    cleanup()?;

    result
}

fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", url])
        .spawn()?;

    Ok(())
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

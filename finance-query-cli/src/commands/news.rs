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
use tabled::Tabled;

#[derive(Parser)]
pub struct NewsArgs {
    /// Stock symbol to get news for (omit for general market news)
    symbol: Option<String>,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,

    /// Maximum number of news items to return
    #[arg(short, long, default_value = "10")]
    limit: usize,
}

#[derive(Debug, Serialize, Tabled)]
struct NewsDisplay {
    #[tabled(rename = "Source")]
    source: String,

    #[tabled(rename = "Title")]
    title: String,

    #[tabled(rename = "Time")]
    time: String,

    #[tabled(rename = "Link")]
    link: String,
}

fn render_interactive_news(news: &[finance_query::News], title: &str) -> Result<()> {
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
                        Constraint::Min(0),    // News list
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
                        format!("Article {} of {}", selected_index + 1, news.len()),
                        Style::default().fg(Color::DarkGray),
                    )]),
                ];
                let header = Paragraph::new(header_text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                );
                f.render_widget(header, chunks[0]);

                // Calculate visible area (each article takes 3 lines)
                let list_height = chunks[1].height.saturating_sub(2) as usize; // Subtract borders
                let items_per_page = list_height / 3;

                // Adjust scroll offset to keep selected item visible
                if selected_index < scroll_offset {
                    scroll_offset = selected_index;
                } else if selected_index >= scroll_offset + items_per_page {
                    scroll_offset = selected_index.saturating_sub(items_per_page - 1);
                }

                // News list
                let items: Vec<ListItem> = news
                    .iter()
                    .enumerate()
                    .skip(scroll_offset)
                    .take(items_per_page)
                    .map(|(idx, article)| {
                        let is_selected = idx == selected_index;

                        let style = if is_selected {
                            Style::default().bg(Color::DarkGray).fg(Color::White)
                        } else {
                            Style::default()
                        };

                        let title_style = if is_selected {
                            Style::default()
                                .bg(Color::DarkGray)
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Cyan)
                        };

                        let content = vec![
                            Line::from(vec![Span::styled(&article.title, title_style)]),
                            Line::from(vec![
                                Span::styled(&article.source, style.fg(Color::Yellow)),
                                Span::styled(" • ", style.fg(Color::DarkGray)),
                                Span::styled(&article.time, style.fg(Color::DarkGray)),
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
                        .title("Articles"),
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
                    Span::styled(" Open in browser  ", Style::default().fg(Color::DarkGray)),
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
                        if selected_index < news.len() - 1 {
                            selected_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        // Open selected article in browser
                        if let Some(article) = news.get(selected_index) {
                            // Disable raw mode temporarily to run the command
                            disable_raw_mode()?;
                            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                            // Open URL in browser
                            let _ = open_url(&article.link);

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
        .args(&["/C", "start", url])
        .spawn()?;

    Ok(())
}

pub async fn execute(args: NewsArgs) -> Result<()> {
    // Fetch news - general market news or symbol-specific
    let (news, title) = if let Some(ref symbol) = args.symbol {
        let ticker = Ticker::new(symbol).await?;
        (ticker.news().await?, symbol.clone())
    } else {
        (
            finance_query::finance::news().await?,
            "Market News".to_string(),
        )
    };

    // Limit news items
    let limited_news: Vec<_> = news.iter().take(args.limit).cloned().collect();

    if limited_news.is_empty() {
        let msg = if args.symbol.is_some() {
            "No news found for this symbol"
        } else {
            "No market news found"
        };
        output::print_info(msg);
        return Ok(());
    }

    // Check if user wants non-interactive output (json/csv)
    let format = OutputFormat::from_str(&args.output)?;

    // Interactive mode is default for table output
    if format == OutputFormat::Table {
        return render_interactive_news(&limited_news, &title);
    }

    // Non-interactive output for JSON/CSV
    let mut news_items = Vec::new();
    for article in limited_news.iter() {
        news_items.push(NewsDisplay {
            source: article.source.clone(),
            title: article.title.clone(),
            time: article.time.clone(),
            link: article.link.clone(),
        });
    }

    output::print_many(&news_items, format)?;

    Ok(())
}

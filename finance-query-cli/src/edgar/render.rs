use super::state::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
};

pub fn render_ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Create layout - add input area if in input mode, error area if error exists
    let has_error = app.error_message.is_some();
    let chunks = match (app.input_mode, has_error) {
        (super::state::InputMode::Normal, false) => Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Header
                Constraint::Min(0),    // Filings list
                Constraint::Length(3), // Footer
            ])
            .split(size),
        (super::state::InputMode::Normal, true) => Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Header
                Constraint::Min(0),    // Filings list
                Constraint::Length(3), // Error box
                Constraint::Length(3), // Footer
            ])
            .split(size),
        (super::state::InputMode::SearchInput | super::state::InputMode::SymbolInput, false) => {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4), // Header
                    Constraint::Min(0),    // Filings list
                    Constraint::Length(3), // Input box
                    Constraint::Length(3), // Footer
                ])
                .split(size)
        }
        (super::state::InputMode::SearchInput | super::state::InputMode::SymbolInput, true) => {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4), // Header
                    Constraint::Min(0),    // Filings list
                    Constraint::Length(3), // Error box
                    Constraint::Length(3), // Input box
                    Constraint::Length(3), // Footer
                ])
                .split(size)
        }
    };

    render_header(f, app, chunks[0]);
    render_filings_list(f, app, chunks[1]);

    match (app.input_mode, has_error) {
        (super::state::InputMode::Normal, false) => {
            render_footer(f, app, chunks[2]);
        }
        (super::state::InputMode::Normal, true) => {
            render_error(f, app, chunks[2]);
            render_footer(f, app, chunks[3]);
        }
        (super::state::InputMode::SearchInput, false) => {
            render_search_input(f, app, chunks[2]);
            render_footer(f, app, chunks[3]);
        }
        (super::state::InputMode::SearchInput, true) => {
            render_error(f, app, chunks[2]);
            render_search_input(f, app, chunks[3]);
            render_footer(f, app, chunks[4]);
        }
        (super::state::InputMode::SymbolInput, false) => {
            render_symbol_input(f, app, chunks[2]);
            render_footer(f, app, chunks[3]);
        }
        (super::state::InputMode::SymbolInput, true) => {
            render_error(f, app, chunks[2]);
            render_symbol_input(f, app, chunks[3]);
            render_footer(f, app, chunks[4]);
        }
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let filter_text = if let Some(filter) = &app.filter_form {
        format!(" [Filter: {}]", filter)
    } else {
        String::new()
    };

    let header_lines = match &app.mode {
        super::state::AppMode::Symbol {
            symbol,
            submissions,
        } => {
            let company_name = submissions.name.as_deref().unwrap_or("Unknown Company");
            let cik = submissions.cik.as_deref().unwrap_or("N/A");

            vec![
                Line::from(vec![
                    Span::styled(
                        format!("SEC EDGAR Filings: {}", symbol.to_uppercase()),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(filter_text, Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![Span::styled(
                    format!("{} (CIK: {})", company_name, cik),
                    Style::default().fg(Color::DarkGray),
                )]),
                Line::from(vec![Span::styled(
                    format!(
                        "Showing filing {} of {}",
                        app.selected_index + 1,
                        app.filings.len()
                    ),
                    Style::default().fg(Color::DarkGray),
                )]),
            ]
        }
        super::state::AppMode::Search {
            query,
            total_results,
            page_size,
            current_offset,
        } => {
            if query.is_empty() {
                // Empty state - show welcome message
                vec![
                    Line::from(vec![Span::styled(
                        "SEC EDGAR Filing Browser",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![Span::styled(
                        "Type your search query below to find filings across all companies",
                        Style::default().fg(Color::Yellow),
                    )]),
                    Line::from(vec![Span::styled(
                        "Or press Esc to quit and run 'fq edgar SYMBOL' to browse a specific company",
                        Style::default().fg(Color::DarkGray),
                    )]),
                ]
            } else {
                let current_page = (current_offset / page_size) + 1;
                let total_pages = total_results.div_ceil(*page_size);

                vec![
                    Line::from(vec![
                        Span::styled(
                            format!("SEC EDGAR Search: \"{}\"", query),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(filter_text, Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(vec![Span::styled(
                        format!(
                            "Page {} of {} ({} results, {} total matches)",
                            current_page,
                            total_pages,
                            app.filings.len(),
                            total_results
                        ),
                        Style::default().fg(Color::DarkGray),
                    )]),
                    Line::from(vec![Span::styled(
                        format!(
                            "Viewing filing {} of {} on this page",
                            app.selected_index + 1,
                            app.filings.len()
                        ),
                        Style::default().fg(Color::DarkGray),
                    )]),
                ]
            }
        }
    };

    let header = Paragraph::new(header_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(header, area);
}

fn render_filings_list(f: &mut Frame, app: &mut App, area: Rect) {
    if app.filings.is_empty() {
        let empty_msg = Paragraph::new("No filings found.")
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title("Filings"),
            );
        f.render_widget(empty_msg, area);
        return;
    }

    // Calculate visible area (each filing takes 4 lines)
    let list_height = area.height.saturating_sub(2) as usize;
    let items_per_page = list_height / 4;

    // Adjust scroll offset to keep selected item visible
    if app.selected_index < app.scroll_offset {
        app.scroll_offset = app.selected_index;
    } else if app.selected_index >= app.scroll_offset + items_per_page {
        app.scroll_offset = app.selected_index.saturating_sub(items_per_page - 1);
    }

    // Create list items
    let items: Vec<ListItem> = app
        .filings
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(items_per_page)
        .map(|(idx, filing)| {
            let is_selected = idx == app.selected_index;

            let style = if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            // Color-code form types
            let form_color = match filing.form.as_str() {
                "10-K" => Color::Green,
                "10-Q" => Color::Cyan,
                "8-K" => Color::Yellow,
                "4" => Color::Magenta,
                "DEF 14A" => Color::Blue,
                _ => Color::White,
            };

            let form_style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(form_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(form_color).add_modifier(Modifier::BOLD)
            };

            // Format size
            let size_str = format_bytes(filing.size);

            // Build filing display
            let content = vec![
                Line::from(vec![
                    Span::styled(format!("[{}] ", filing.form), form_style),
                    Span::styled(filing.primary_doc_description.clone(), style),
                ]),
                Line::from(vec![
                    Span::styled("Filed: ", style.fg(Color::DarkGray)),
                    Span::styled(filing.filing_date.clone(), style.fg(Color::Yellow)),
                    Span::styled("  Report: ", style.fg(Color::DarkGray)),
                    Span::styled(filing.report_date.clone(), style.fg(Color::Yellow)),
                    Span::styled("  Size: ", style.fg(Color::DarkGray)),
                    Span::styled(size_str, style.fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::styled("Accession: ", style.fg(Color::DarkGray)),
                    Span::styled(filing.accession_number.clone(), style.fg(Color::Gray)),
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

    f.render_widget(list, area);

    // Render scrollbar
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let mut scrollbar_state = ScrollbarState::new(app.filings.len()).position(app.selected_index);

    let scrollbar_area = Rect {
        x: area.x + area.width - 1,
        y: area.y + 1,
        width: 1,
        height: area.height.saturating_sub(2),
    };

    f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
}

fn render_search_input(f: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(app.search_input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Search EDGAR (Enter to search, Esc to cancel)"),
        );
    f.render_widget(input, area);
}

fn render_symbol_input(f: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(app.symbol_input.as_str())
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title("Symbol Lookup (Enter to browse, Esc to cancel)"),
        );
    f.render_widget(input, area);
}

fn render_error(f: &mut Frame, app: &App, area: Rect) {
    if let Some(error_msg) = &app.error_message {
        let error = Paragraph::new(error_msg.as_str())
            .style(Style::default().fg(Color::Red))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .title("Error (press any key to dismiss)"),
            );
        f.render_widget(error, area);
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut footer_items = vec![
        Span::styled(
            "↑/↓/j/k",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "Enter/o",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Open  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "f",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Filter  ", Style::default().fg(Color::DarkGray)),
    ];

    // Add pagination controls only in search mode
    if matches!(app.mode, super::state::AppMode::Search { .. }) {
        footer_items.extend_from_slice(&[
            Span::styled(
                "←/→",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Page  ", Style::default().fg(Color::DarkGray)),
        ]);
    }

    footer_items.extend_from_slice(&[
        Span::styled(
            "/",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Search  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "s",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Symbol  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "q/Esc",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Quit", Style::default().fg(Color::DarkGray)),
    ]);

    let footer = Paragraph::new(vec![Line::from(footer_items)]).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(footer, area);
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

use super::state::{OptionsApp, SortField, ViewMode};
use chrono::{DateTime, Utc};
use finance_query::OptionContract;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

pub fn ui(f: &mut Frame, app: &mut OptionsApp) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Expiration selector
            Constraint::Min(0),    // Main content
            Constraint::Length(2), // Status bar
        ])
        .split(size);

    render_header(f, app, chunks[0]);
    render_expiration_bar(f, app, chunks[1]);
    render_options_chain(f, app, chunks[2]);
    render_status_bar(f, app, chunks[3]);

    // Show help overlay if enabled
    if app.show_help {
        render_help_overlay(f, size);
    }
}

fn render_header(f: &mut Frame, app: &OptionsApp, area: Rect) {
    let price_str = app
        .underlying_price
        .map(|p| format!("${:.2}", p))
        .unwrap_or_else(|| "N/A".to_string());

    let title = format!(
        " {} Options Chain │ Underlying: {} │ View: {} ",
        app.symbol,
        price_str,
        app.view_mode.name()
    );

    let header = Paragraph::new(Line::from(vec![Span::styled(
        title,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(header, area);
}

fn render_expiration_bar(f: &mut Frame, app: &OptionsApp, area: Rect) {
    if app.expiration_dates.is_empty() {
        let msg = Paragraph::new("No expirations available")
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Expirations"));
        f.render_widget(msg, area);
        return;
    }

    let mut spans = vec![Span::raw(" ")];

    // Show a window of expirations around the selected one
    let total = app.expiration_dates.len();
    let selected = app.selected_expiration_idx;
    let window_size = 7;
    let half_window = window_size / 2;

    let start = if selected <= half_window {
        0
    } else if selected >= total - half_window {
        total.saturating_sub(window_size)
    } else {
        selected - half_window
    };
    let end = (start + window_size).min(total);

    if start > 0 {
        spans.push(Span::styled("◀ ", Style::default().fg(Color::DarkGray)));
    }

    for (i, exp) in app.expiration_dates[start..end].iter().enumerate() {
        let idx = start + i;
        let date_str = format_expiration(*exp);

        let style = if idx == selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::Gray)
        };

        spans.push(Span::styled(format!(" {} ", date_str), style));

        if idx < end - 1 {
            spans.push(Span::raw("│"));
        }
    }

    if end < total {
        spans.push(Span::styled(" ▶", Style::default().fg(Color::DarkGray)));
    }

    spans.push(Span::styled(
        format!("  ({}/{})", selected + 1, total),
        Style::default().fg(Color::DarkGray),
    ));

    let exp_bar = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Expirations [/] to navigate, [n/p] next/prev"),
    );

    f.render_widget(exp_bar, area);
}

fn render_options_chain(f: &mut Frame, app: &mut OptionsApp, area: Rect) {
    if app.is_loading {
        let loading = Paragraph::new("Loading options data...")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(loading, area);
        return;
    }

    if let Some(ref err) = app.error_message {
        let error = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        f.render_widget(error, area);
        return;
    }

    match app.view_mode {
        ViewMode::Straddle => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            render_calls_table(f, app, chunks[0], app.calls_focused);
            render_puts_table(f, app, chunks[1], !app.calls_focused);
        }
        ViewMode::CallsOnly => {
            render_calls_table(f, app, area, true);
        }
        ViewMode::PutsOnly => {
            render_puts_table(f, app, area, true);
        }
    }
}

fn render_calls_table(f: &mut Frame, app: &mut OptionsApp, area: Rect, focused: bool) {
    let contracts = app.filtered_calls();
    let underlying = app.underlying_price;

    let border_style = if focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = format!(
        " CALLS ({}) {} ",
        contracts.len(),
        if focused { "●" } else { "" }
    );

    let table = create_options_table(&contracts, underlying, true, app.sort_field);
    let table = table.block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(title, Style::default().fg(Color::Green))),
    );

    f.render_stateful_widget(table, area, &mut app.calls_table_state);
}

fn render_puts_table(f: &mut Frame, app: &mut OptionsApp, area: Rect, focused: bool) {
    let contracts = app.filtered_puts();
    let underlying = app.underlying_price;

    let border_style = if focused {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = format!(
        " PUTS ({}) {} ",
        contracts.len(),
        if focused { "●" } else { "" }
    );

    let table = create_options_table(&contracts, underlying, false, app.sort_field);
    let table = table.block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(title, Style::default().fg(Color::Red))),
    );

    f.render_stateful_widget(table, area, &mut app.puts_table_state);
}

fn create_options_table<'a>(
    contracts: &[&OptionContract],
    underlying: Option<f64>,
    is_call: bool,
    sort_field: SortField,
) -> Table<'a> {
    let header_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);

    // Build header with sort indicator
    let header_cells = [
        ("Strike", SortField::Strike),
        ("Last", SortField::LastPrice),
        ("Chg%", SortField::Change),
        ("Bid", SortField::Strike), // No sort for bid
        ("Ask", SortField::Strike), // No sort for ask
        ("Vol", SortField::Volume),
        ("OI", SortField::OpenInterest),
        ("IV", SortField::ImpliedVolatility),
    ]
    .iter()
    .map(|(name, field)| {
        let indicator = if *field == sort_field && *name != "Bid" && *name != "Ask" {
            " ▼"
        } else {
            ""
        };
        Cell::from(format!("{}{}", name, indicator)).style(header_style)
    })
    .collect::<Vec<_>>();

    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = contracts
        .iter()
        .map(|c| {
            let itm = c.in_the_money.unwrap_or(false);
            let row_style = if itm {
                Style::default().bg(Color::Rgb(30, 30, 50))
            } else {
                Style::default()
            };

            // Calculate moneyness indicator
            let strike_style = if let Some(price) = underlying {
                if is_call {
                    if c.strike < price {
                        Style::default().fg(Color::Green) // ITM call
                    } else if c.strike > price * 1.1 {
                        Style::default().fg(Color::DarkGray) // Deep OTM
                    } else {
                        Style::default().fg(Color::Yellow) // Near ATM
                    }
                } else if c.strike > price {
                    Style::default().fg(Color::Green) // ITM put
                } else if c.strike < price * 0.9 {
                    Style::default().fg(Color::DarkGray) // Deep OTM
                } else {
                    Style::default().fg(Color::Yellow) // Near ATM
                }
            } else {
                Style::default()
            };

            let change_pct = c.percent_change.unwrap_or(0.0);
            let change_style = if change_pct >= 0.0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let iv = c.implied_volatility.unwrap_or(0.0);
            let iv_style = if iv > 1.0 {
                Style::default().fg(Color::Red) // High IV
            } else if iv > 0.5 {
                Style::default().fg(Color::Yellow) // Medium IV
            } else {
                Style::default().fg(Color::Gray)
            };

            Row::new(vec![
                Cell::from(format!("{:.2}", c.strike)).style(strike_style),
                Cell::from(format_price(c.last_price)),
                Cell::from(format!("{:+.1}%", change_pct)).style(change_style),
                Cell::from(format_price(c.bid)),
                Cell::from(format_price(c.ask)),
                Cell::from(format_volume(c.volume)),
                Cell::from(format_volume(c.open_interest)),
                Cell::from(format!("{:.1}%", iv * 100.0)).style(iv_style),
            ])
            .style(row_style)
        })
        .collect();

    Table::new(
        rows,
        [
            Constraint::Length(9), // Strike
            Constraint::Length(8), // Last
            Constraint::Length(7), // Chg%
            Constraint::Length(8), // Bid
            Constraint::Length(8), // Ask
            Constraint::Length(8), // Vol
            Constraint::Length(8), // OI
            Constraint::Length(7), // IV
        ],
    )
    .header(header)
    .row_highlight_style(Style::default().bg(Color::Rgb(50, 50, 80)))
}

fn render_status_bar(f: &mut Frame, app: &OptionsApp, area: Rect) {
    let mut help_spans = vec![
        Span::styled(" q", Style::default().fg(Color::Yellow)),
        Span::raw(":quit "),
        Span::styled("?", Style::default().fg(Color::Yellow)),
        Span::raw(":help "),
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(":navigate "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(":switch "),
        Span::styled("[/]", Style::default().fg(Color::Yellow)),
        Span::raw(":expiry "),
        Span::styled("v", Style::default().fg(Color::Yellow)),
        Span::raw(":view "),
        Span::styled("s", Style::default().fg(Color::Yellow)),
        Span::raw(format!(":sort({}) ", app.sort_field.name())),
    ];

    // Show active filters
    if app.filter_itm_only {
        help_spans.push(Span::styled(" [ITM]", Style::default().fg(Color::Cyan)));
    }
    if let Some(vol) = app.filter_min_volume {
        help_spans.push(Span::styled(
            format!(" [Vol>{}]", vol),
            Style::default().fg(Color::Cyan),
        ));
    }

    let status = Paragraph::new(Line::from(help_spans));
    f.render_widget(status, area);
}

fn render_help_overlay(f: &mut Frame, area: Rect) {
    let help_area = centered_rect(60, 70, area);

    // Clear the area first
    f.render_widget(Clear, help_area);

    let help_text = vec![
        Line::from(Span::styled(
            " Options Chain Help ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  ↑/↓ or j/k     Move selection up/down"),
        Line::from("  PgUp/PgDn      Move 10 rows"),
        Line::from("  Home/End       Go to first/last row"),
        Line::from("  Tab or ←/→    Switch between calls/puts"),
        Line::from(""),
        Line::from(Span::styled(
            "Expiration",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  [ or p         Previous expiration"),
        Line::from("  ] or n         Next expiration"),
        Line::from(""),
        Line::from(Span::styled(
            "View & Sorting",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  v              Cycle view (Straddle/Calls/Puts)"),
        Line::from("  s              Cycle sort field"),
        Line::from("  o              Toggle sort order (asc/desc)"),
        Line::from(""),
        Line::from(Span::styled(
            "Filters",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  i              Toggle ITM only filter"),
        Line::from("  V (shift)      Cycle volume filter (off/100/1k/10k)"),
        Line::from(""),
        Line::from(Span::styled(
            "Other",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  r              Refresh data"),
        Line::from("  q or Esc       Quit"),
        Line::from("  ? or h         Toggle this help"),
        Line::from(""),
        Line::from(Span::styled(
            "Color Legend",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Green strike   In-the-money (ITM)"),
        Line::from("  Yellow strike  Near-the-money (ATM)"),
        Line::from("  Gray strike    Out-of-the-money (OTM)"),
        Line::from("  Red IV         High implied volatility (>100%)"),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let help = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Help "),
    );

    f.render_widget(help, help_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn format_expiration(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .map(|dt: DateTime<Utc>| dt.format("%b %d").to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

fn format_price(price: Option<f64>) -> String {
    price
        .map(|p| format!("{:.2}", p))
        .unwrap_or_else(|| "-".to_string())
}

fn format_volume(vol: Option<i64>) -> String {
    vol.map(|v| {
        if v >= 1_000_000 {
            format!("{:.1}M", v as f64 / 1_000_000.0)
        } else if v >= 1_000 {
            format!("{:.1}K", v as f64 / 1_000.0)
        } else {
            v.to_string()
        }
    })
    .unwrap_or_else(|| "-".to_string())
}

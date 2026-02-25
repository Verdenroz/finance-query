use super::state::{App, FocusPane, InputMode, ScreenerCategory, SectorsViewMode, Tab};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Block, Borders, Chart, Dataset, GraphType, List, ListItem, ListState, Paragraph, Sparkline,
    },
};

pub fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    render_tabs(f, app, chunks[0]);

    // Alerts and Sectors tabs use full width, others use left/right split
    if app.active_tab == Tab::Alerts {
        render_alerts_tab(f, app, chunks[1]);
    } else if app.active_tab == Tab::Sectors {
        render_sectors_tab(f, app, chunks[1]);
    } else {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(chunks[1]);

        render_watchlist(f, app, content_chunks[0]);
        render_details(f, app, content_chunks[1]);
    }

    render_status(f, app, chunks[2]);
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tabs = [
        Tab::Watchlist,
        Tab::Charts,
        Tab::News,
        Tab::Lookup,
        Tab::Screeners,
        Tab::Sectors,
        Tab::Portfolio,
        Tab::Alerts,
    ];

    let tab_titles: Vec<Span> = tabs
        .iter()
        .map(|tab| {
            let style = if *tab == app.active_tab {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Span::styled(format!(" {} ", tab.title()), style)
        })
        .collect();

    let tabs_line = Line::from(tab_titles);
    let tabs_widget = Paragraph::new(tabs_line).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Tabs (Tab/Shift+Tab)"),
    );

    f.render_widget(tabs_widget, area);
}

fn render_watchlist(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        "Watchlist ({}) [a:add d:delete]",
        app.current_watchlist.name
    );
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.current_watchlist.symbols.is_empty() {
        let empty =
            Paragraph::new("Press 'a' to add symbols").style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty, inner);
        return;
    }

    let visible_rows = inner.height as usize;

    for (i, symbol) in app.current_watchlist.symbols.iter().enumerate() {
        if i >= visible_rows {
            break;
        }

        let row_area = Rect {
            x: inner.x,
            y: inner.y + i as u16,
            width: inner.width,
            height: 1,
        };

        // Split row: text on left (~28 chars), sparkline on right
        let text_width = 28u16.min(row_area.width.saturating_sub(8));
        let sparkline_width = row_area.width.saturating_sub(text_width);

        let text_area = Rect {
            x: row_area.x,
            y: row_area.y,
            width: text_width,
            height: 1,
        };
        let spark_area = Rect {
            x: row_area.x + text_width,
            y: row_area.y,
            width: sparkline_width,
            height: 1,
        };

        // Build price info spans
        let (price_info, _change_pct) = if let Some(update) = app.price_updates.get(symbol) {
            let change_color = if update.change >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            let arrow = if update.change >= 0.0 { "↑" } else { "↓" };
            (
                vec![
                    Span::styled(
                        format!("{:<6}", symbol),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" ${:>8.2} ", update.price)),
                    Span::styled(
                        format!("{:>+6.2}%{}", update.change_percent, arrow),
                        Style::default().fg(change_color),
                    ),
                ],
                Some(update.change_percent as f64),
            )
        } else if let Some(quote) = app.quotes.get(symbol) {
            let price = quote
                .regular_market_price
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0);
            let change_pct = quote
                .regular_market_change_percent
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0);

            let change_color = if change_pct >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            let arrow = if change_pct >= 0.0 { "↑" } else { "↓" };

            (
                vec![
                    Span::styled(
                        format!("{:<6}", symbol),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" ${:>8.2} ", price)),
                    Span::styled(
                        format!("{:>+6.2}%{}", change_pct, arrow),
                        Style::default().fg(change_color),
                    ),
                ],
                Some(change_pct),
            )
        } else {
            (
                vec![
                    Span::styled(format!("{:<6}", symbol), Style::default().fg(Color::Cyan)),
                    Span::styled(" Loading...", Style::default().fg(Color::DarkGray)),
                ],
                None,
            )
        };

        // Apply selection highlight
        let row_style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        // Render text
        let text = Paragraph::new(Line::from(price_info)).style(row_style);
        f.render_widget(text, text_area);

        // Render sparkline with per-bar colors (green above open, red below)
        if sparkline_width >= 4
            && let Some(spark) = app.sparklines.get(symbol)
            && !spark.closes.is_empty()
        {
            use ratatui::widgets::SparklineBar;

            let min = spark.min_close().unwrap_or(0.0);
            let max = spark.max_close().unwrap_or(100.0);
            let range = (max - min).max(0.01);

            // Reference price is the first price (open)
            let open_price = spark.closes[0];

            // Create bars with individual colors based on price vs open
            let bars: Vec<SparklineBar> = spark
                .closes
                .iter()
                .map(|&price| {
                    let value = ((price - min) / range * 100.0) as u64;
                    let color = if price >= open_price {
                        Color::Green
                    } else {
                        Color::Red
                    };
                    SparklineBar::from(value).style(Style::default().fg(color))
                })
                .collect();

            let sparkline = Sparkline::default().data(bars);

            f.render_widget(sparkline, spark_area);
        }
    }
}

fn render_sectors_tab(f: &mut Frame, app: &App, area: Rect) {
    if app.is_loading_sectors {
        // Show animated loading indicator
        let frame_num = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() / 200)
            .unwrap_or(0) as usize;
        let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let spinner_char = spinner[frame_num % spinner.len()];

        let loading = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    format!(" {} ", spinner_char),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled("Loading sector data...", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Fetching performance data from 11 market sectors",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Sectors Overview"),
        );
        f.render_widget(loading, area);
        return;
    }

    if app.sectors_data.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Press 'r' to load sector data",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from("This view shows:"),
            Line::from("  • All 11 market sectors"),
            Line::from("  • Visual treemap with performance colors"),
            Line::from("  • Daily & YTD performance"),
            Line::from("  • Industries within each sector"),
            Line::from(""),
            Line::from(Span::styled(
                "Controls:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  j/k or ↑/↓ : Navigate sectors"),
            Line::from("  Enter      : Drill into industries"),
            Line::from("  Esc        : Back to sectors view"),
            Line::from("  r          : Refresh data"),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Sectors Overview"),
        );
        f.render_widget(empty, area);
        return;
    }

    // Main layout: treemap on left, details on right
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_sectors_treemap(f, app, chunks[0]);
    render_sectors_details(f, app, chunks[1]);
}

/// Render a visual treemap showing sector performance
fn render_sectors_treemap(f: &mut Frame, app: &App, area: Rect) {
    let sectors = app.get_sorted_sectors();
    if sectors.is_empty() {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Sector Treemap [j/k:nav Enter:details]")
        .border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create a treemap-like layout
    // For 11 sectors, we'll use a 4-row layout: 3+3+3+2
    let row_configs = [3, 3, 3, 2];
    let mut sector_idx = 0;

    let row_height = inner.height / row_configs.len() as u16;
    let mut y = inner.y;

    for (row_idx, &cols) in row_configs.iter().enumerate() {
        let row_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: if row_idx == row_configs.len() - 1 {
                inner.height.saturating_sub(y - inner.y)
            } else {
                row_height
            },
        };

        // Split row into columns
        let col_constraints: Vec<Constraint> = (0..cols)
            .map(|_| Constraint::Ratio(1, cols as u32))
            .collect();
        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(row_area);

        for col_area in col_chunks.iter() {
            if sector_idx >= sectors.len() {
                break;
            }

            let (sector_type, sector) = sectors[sector_idx];
            let day_change_raw = sector
                .performance
                .as_ref()
                .and_then(|p| p.day_change_percent.as_ref())
                .and_then(|v| v.raw)
                .unwrap_or(0.0);
            // Convert decimal to percentage (0.01 -> 1%)
            let day_change = day_change_raw * 100.0;

            let is_selected = sector_idx == app.sectors_selected_idx;
            let bg_color = get_sector_bg_color(day_change);
            let fg_color = if day_change.abs() > 2.0 {
                Color::White
            } else {
                Color::Black
            };

            // Build cell content
            let name = sector_type.display_name();
            let short_name = if name.len() > col_area.width as usize - 2 {
                truncate_str(name, (col_area.width as usize).saturating_sub(3))
            } else {
                name.to_string()
            };

            let change_str = format!("{:+.2}%", day_change);

            let mut lines = vec![];
            if col_area.height >= 3 {
                lines.push(Line::from(Span::styled(
                    short_name,
                    Style::default().fg(fg_color).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    change_str,
                    Style::default().fg(fg_color),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    format!("{} {}", short_name, change_str),
                    Style::default().fg(fg_color).add_modifier(Modifier::BOLD),
                )));
            }

            let style = Style::default().bg(bg_color);
            // No need to change style for selected - border highlight is enough

            let cell = Paragraph::new(lines).style(style).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if is_selected {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    }),
            );
            f.render_widget(cell, *col_area);

            sector_idx += 1;
        }

        y += row_height;
    }
}

fn render_sectors_details(f: &mut Frame, app: &App, area: Rect) {
    let Some((_sector_type, sector)) = app.selected_sector() else {
        let empty = Paragraph::new("Select a sector")
            .block(Block::default().borders(Borders::ALL).title("Details"));
        f.render_widget(empty, area);
        return;
    };

    match app.sectors_view_mode {
        SectorsViewMode::Overview => render_sector_overview(f, sector, area),
        SectorsViewMode::Industries => render_industries_view(f, app, sector, area),
    }
}

/// Get background color for sector treemap cells
fn get_sector_bg_color(change: f64) -> Color {
    if change >= 3.0 {
        Color::Rgb(0, 128, 0) // Dark green
    } else if change >= 2.0 {
        Color::Rgb(34, 139, 34) // Forest green
    } else if change >= 1.0 {
        Color::Rgb(60, 179, 113) // Medium sea green
    } else if change >= 0.0 {
        Color::Rgb(144, 238, 144) // Light green
    } else if change >= -1.0 {
        Color::Rgb(255, 160, 122) // Light salmon
    } else if change >= -2.0 {
        Color::Rgb(205, 92, 92) // Indian red
    } else if change >= -3.0 {
        Color::Rgb(178, 34, 34) // Firebrick
    } else {
        Color::Rgb(139, 0, 0) // Dark red
    }
}

fn render_sector_overview(f: &mut Frame, sector: &finance_query::SectorData, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Performance (increased for 5 year)
            Constraint::Length(10), // Top companies
            Constraint::Min(0),     // Industries preview
        ])
        .split(area);

    // Performance section
    let perf = sector.performance.as_ref();
    let bench = sector.benchmark.as_ref();

    // Raw values are decimal fractions (0.01 = 1%), multiply by 100 for display
    let day_change = perf
        .and_then(|p| p.day_change_percent.as_ref())
        .and_then(|v| v.raw)
        .unwrap_or(0.0)
        * 100.0;
    let ytd_change = perf
        .and_then(|p| p.ytd_change_percent.as_ref())
        .and_then(|v| v.raw)
        .unwrap_or(0.0)
        * 100.0;
    let one_year = perf
        .and_then(|p| p.one_year_change_percent.as_ref())
        .and_then(|v| v.raw)
        .unwrap_or(0.0)
        * 100.0;
    let three_year = perf
        .and_then(|p| p.three_year_change_percent.as_ref())
        .and_then(|v| v.raw)
        .map(|v| v * 100.0);
    let five_year = perf
        .and_then(|p| p.five_year_change_percent.as_ref())
        .and_then(|v| v.raw)
        .map(|v| v * 100.0);

    let bench_day = bench
        .and_then(|p| p.day_change_percent.as_ref())
        .and_then(|v| v.raw)
        .unwrap_or(0.0)
        * 100.0;
    let bench_ytd = bench
        .and_then(|p| p.ytd_change_percent.as_ref())
        .and_then(|v| v.raw)
        .unwrap_or(0.0)
        * 100.0;

    let day_color = get_change_color(day_change);
    let ytd_color = get_change_color(ytd_change);
    let one_year_color = get_change_color(one_year);

    let perf_lines = vec![
        Line::from(vec![Span::styled(
            sector.name.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Day:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:>+7.2}%", day_change),
                Style::default().fg(day_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  (S&P: {:>+.2}%)", bench_day),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("YTD:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:>+7.2}%", ytd_change),
                Style::default().fg(ytd_color),
            ),
            Span::styled(
                format!("  (S&P: {:>+.2}%)", bench_ytd),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("1 Year: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:>+7.2}%", one_year),
                Style::default().fg(one_year_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("3 Year: ", Style::default().fg(Color::DarkGray)),
            Span::raw(
                three_year
                    .map(|v| format!("{:>+7.2}%", v))
                    .unwrap_or_else(|| "    N/A".to_string()),
            ),
        ]),
        Line::from(vec![
            Span::styled("5 Year: ", Style::default().fg(Color::DarkGray)),
            Span::raw(
                five_year
                    .map(|v| format!("{:>+7.2}%", v))
                    .unwrap_or_else(|| "    N/A".to_string()),
            ),
        ]),
    ];

    let perf_widget = Paragraph::new(perf_lines)
        .block(Block::default().borders(Borders::ALL).title("Performance"));
    f.render_widget(perf_widget, chunks[0]);

    // Top companies section
    let company_lines: Vec<Line> = sector
        .top_companies
        .iter()
        .take(4)
        .map(|company| {
            let price = company
                .last_price
                .as_ref()
                .and_then(|v| v.fmt.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("N/A");
            let change = company
                .day_change_percent
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0)
                * 100.0;
            let color = get_change_color(change);

            Line::from(vec![
                Span::styled(
                    format!("{:<6}", company.symbol),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(format!(" {:>10}", price)),
                Span::styled(format!(" {:>+6.2}%", change), Style::default().fg(color)),
            ])
        })
        .collect();

    let companies_widget = Paragraph::new(company_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Top Companies"),
    );
    f.render_widget(companies_widget, chunks[1]);

    // Industries preview
    let industry_lines: Vec<Line> = sector
        .industries
        .iter()
        .take(8)
        .map(|industry| {
            let change = industry
                .day_change_percent
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0)
                * 100.0;
            let weight = industry
                .market_weight
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0)
                * 100.0;
            let color = get_change_color(change);

            Line::from(vec![
                Span::raw(format!("{:<25}", truncate_str(&industry.name, 24))),
                Span::styled(format!("{:>+6.2}%", change), Style::default().fg(color)),
                Span::styled(
                    format!(" ({:.1}%)", weight),
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        })
        .collect();

    let industries_widget = Paragraph::new(industry_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Industries [Enter to expand]"),
    );
    f.render_widget(industries_widget, chunks[2]);
}

fn render_industries_view(
    f: &mut Frame,
    app: &App,
    sector: &finance_query::SectorData,
    area: Rect,
) {
    let items: Vec<ListItem> = sector
        .industries
        .iter()
        .enumerate()
        .map(|(idx, industry)| {
            // Raw values are decimal fractions, multiply by 100 for display
            let day_change = industry
                .day_change_percent
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0)
                * 100.0;
            let color = get_change_color(day_change);

            let is_selected = idx == app.sectors_selected_industry;

            // Keep it simple - just name and day change to fit within panel
            let line = Line::from(vec![
                Span::styled(
                    format!("{:<35}", truncate_str(&industry.name, 34)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:>+7.2}%", day_change),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
            ]);

            let style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let title = format!("{} Industries [Esc:back j/k:nav]", sector.name);
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(list, area);
}

/// Get color based on percentage change (green gradient for positive, red for negative)
fn get_change_color(change: f64) -> Color {
    if change >= 3.0 {
        Color::Rgb(0, 255, 0) // Bright green
    } else if change >= 2.0 {
        Color::Rgb(50, 205, 50) // Lime green
    } else if change >= 1.0 {
        Color::Rgb(144, 238, 144) // Light green
    } else if change >= 0.0 {
        Color::Rgb(152, 251, 152) // Pale green
    } else if change >= -1.0 {
        Color::Rgb(255, 182, 193) // Light pink
    } else if change >= -2.0 {
        Color::Rgb(255, 99, 71) // Tomato
    } else if change >= -3.0 {
        Color::Rgb(220, 20, 60) // Crimson
    } else {
        Color::Rgb(139, 0, 0) // Dark red
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

fn render_alerts_tab(f: &mut Frame, app: &App, area: Rect) {
    // Check if we're in add alert mode
    if app.input_mode == InputMode::AddAlert {
        render_add_alert_form(f, app, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left side: Alert list
    let items: Vec<ListItem> = app
        .alerts
        .iter()
        .enumerate()
        .map(|(idx, alert)| {
            let triggered = alert.last_triggered.is_some();
            let icon = if triggered { "*" } else { "o" };
            let status_color = if triggered {
                Color::Red
            } else if !alert.enabled {
                Color::DarkGray
            } else {
                Color::Green
            };
            let is_selected = idx == app.selected_alert_idx;

            let text = format!(
                "{} {:<6} {:<12} {}",
                icon,
                alert.symbol,
                alert.alert_type.short_display(),
                alert.alert_type.format_threshold(alert.threshold)
            );

            let style = if is_selected {
                Style::default()
                    .fg(status_color)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(status_color)
            };

            ListItem::new(Line::from(vec![Span::styled(text, style)]))
        })
        .collect();

    let list_title = format!(
        "Alerts ({}) [a:add d:del e:toggle j/k:nav]",
        app.alerts.len()
    );

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(list_title)
            .border_style(Style::default().fg(if app.focus_pane == FocusPane::Left {
                Color::Yellow
            } else {
                Color::White
            })),
    );

    f.render_widget(list, chunks[0]);

    // Right side: Selected alert details or help
    let detail_lines = if app.alerts.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "No alerts configured",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from("Press 'a' to add your first alert."),
            Line::from(""),
            Line::from(Span::styled(
                "Available Alert Types:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("  Price >    - Price goes above value"),
            Line::from("  Price <    - Price goes below value"),
            Line::from("  Chg% >     - Daily change exceeds %"),
            Line::from("  Chg% <     - Daily change below %"),
            Line::from("  Vol spike  - Volume exceeds average"),
            Line::from("  52W High   - Near 52-week high"),
            Line::from("  52W Low    - Near 52-week low"),
            Line::from("  MCap >/<   - Market cap threshold"),
            Line::from("  DivYld >   - Dividend yield above"),
            Line::from("  P/E >/<    - P/E ratio threshold"),
        ]
    } else if app.selected_alert_idx < app.alerts.len() {
        let alert = &app.alerts[app.selected_alert_idx];
        let status = if alert.last_triggered.is_some() {
            Span::styled(
                "TRIGGERED",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )
        } else if !alert.enabled {
            Span::styled("Disabled", Style::default().fg(Color::DarkGray))
        } else {
            Span::styled("Active", Style::default().fg(Color::Green))
        };

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Symbol:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &alert.symbol,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Type:      ", Style::default().fg(Color::DarkGray)),
                Span::raw(alert.alert_type.display()),
            ]),
            Line::from(vec![
                Span::styled("Threshold: ", Style::default().fg(Color::DarkGray)),
                Span::raw(alert.alert_type.format_threshold(alert.threshold)),
            ]),
            Line::from(vec![
                Span::styled("Status:    ", Style::default().fg(Color::DarkGray)),
                status,
            ]),
            Line::from(vec![
                Span::styled("Created:   ", Style::default().fg(Color::DarkGray)),
                Span::raw(alert.created_at.format("%Y-%m-%d %H:%M").to_string()),
            ]),
        ];

        if let Some(label) = &alert.label {
            lines.push(Line::from(vec![
                Span::styled("Label:     ", Style::default().fg(Color::DarkGray)),
                Span::raw(label),
            ]));
        }

        if let Some(last_triggered) = alert.last_triggered {
            lines.push(Line::from(vec![
                Span::styled("Triggered: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    last_triggered.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(Color::Yellow),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Count:     ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{} time(s)", alert.trigger_count)),
            ]));
        }

        // Show current value if we have quote data
        if let Some(quote) = app.quotes.get(&alert.symbol) {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "─ Current Value ─",
                Style::default().fg(Color::DarkGray),
            )));
            let current = alert.format_current_value(quote);
            let triggered_now = alert.check(quote);
            let current_style = if triggered_now {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(vec![
                Span::styled("Current:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(current, current_style),
            ]));
        }

        lines
    } else {
        vec![Line::from("Select an alert")]
    };

    let detail_block = Paragraph::new(detail_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Alert Details")
            .border_style(Style::default().fg(if app.focus_pane == FocusPane::Right {
                Color::Yellow
            } else {
                Color::White
            })),
    );

    f.render_widget(detail_block, chunks[1]);
}

fn render_add_alert_form(f: &mut Frame, app: &App, area: Rect) {
    use crate::alerts::AlertType;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        "Create New Alert",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Form fields
    let all_types = AlertType::all();
    let alert_type = all_types[app.alert_form_type_idx];

    let form_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[1]);

    // Symbol field
    let symbol_style = if app.alert_form_field == 0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let symbol_text = if app.alert_form_field == 0 {
        format!("{}▌", app.alert_form_symbol)
    } else if app.alert_form_symbol.is_empty() {
        "Enter symbol...".to_string()
    } else {
        app.alert_form_symbol.clone()
    };
    let symbol_field = Paragraph::new(symbol_text)
        .style(symbol_style)
        .block(Block::default().borders(Borders::ALL).title("Symbol"));
    f.render_widget(symbol_field, form_chunks[0]);

    // Type field
    let type_style = if app.alert_form_field == 1 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let type_text = if app.alert_form_field == 1 {
        format!("< {} >", alert_type.display())
    } else {
        alert_type.display().to_string()
    };
    let type_field = Paragraph::new(type_text)
        .style(type_style)
        .block(Block::default().borders(Borders::ALL).title("Type (←/→)"));
    f.render_widget(type_field, form_chunks[1]);

    // Threshold field
    let threshold_style = if app.alert_form_field == 2 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let threshold_text = if app.alert_form_field == 2 {
        format!("{}▌", app.alert_form_threshold)
    } else if app.alert_form_threshold.is_empty() {
        "Enter value...".to_string()
    } else {
        app.alert_form_threshold.clone()
    };
    let threshold_field = Paragraph::new(threshold_text)
        .style(threshold_style)
        .block(Block::default().borders(Borders::ALL).title("Value"));
    f.render_widget(threshold_field, form_chunks[2]);

    // Help text
    let help_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Next field   "),
            Span::styled("←/→", Style::default().fg(Color::Yellow)),
            Span::raw(": Change type   "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Create   "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Cancel"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Alert Types:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Price >/< : Alert when price crosses threshold (e.g., 150.00)"),
        Line::from("  Chg% >/<  : Alert when daily % change crosses threshold (e.g., 5.0)"),
        Line::from("  Vol spike : Alert when volume exceeds multiplier of average (e.g., 2.0)"),
        Line::from("  52W High/Low : Alert when within % of 52-week high/low (e.g., 5.0)"),
        Line::from("  MCap >/<  : Alert when market cap crosses threshold in billions (e.g., 100)"),
        Line::from("  DivYld >  : Alert when dividend yield exceeds % (e.g., 4.0)"),
        Line::from("  P/E >/<   : Alert when P/E ratio crosses threshold (e.g., 30)"),
    ];

    let help = Paragraph::new(help_lines)
        .block(Block::default().borders(Borders::ALL).title("Instructions"));
    f.render_widget(help, chunks[2]);
}

fn render_details(f: &mut Frame, app: &App, area: Rect) {
    match app.active_tab {
        Tab::Watchlist => render_symbol_details(f, app, area),
        Tab::Charts => render_chart(f, app, area),
        Tab::News => render_news(f, app, area),
        Tab::Lookup => render_lookup(f, app, area),
        Tab::Screeners => render_screeners(f, app, area),
        Tab::Sectors => {} // Sectors tab renders full-width, not here
        Tab::Portfolio => render_portfolio(f, app, area),
        Tab::Alerts => {} // Alerts tab renders full-width, not here
    }
}

fn render_symbol_details(f: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    let Some(symbol) = app.selected_symbol() else {
        let paragraph = Paragraph::new(vec![
            Line::from(Span::styled(
                "No symbol selected",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from("Use j/k to navigate watchlist"),
            Line::from("Press 'a' to add a symbol"),
        ])
        .block(Block::default().borders(Borders::ALL).title("Details"));
        f.render_widget(paragraph, area);
        return;
    };

    let quote = app.quotes.get(symbol);

    let (price, change, change_pct, day_low, day_high, volume, streaming_market_hours) =
        if let Some(update) = app.price_updates.get(symbol) {
            (
                update.price as f64,
                update.change as f64,
                update.change_percent as f64,
                Some(update.day_low as f64),
                Some(update.day_high as f64),
                update.day_volume,
                Some(update.market_hours),
            )
        } else if let Some(q) = quote {
            let p = q
                .regular_market_price
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0);
            let c = q
                .regular_market_change
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0);
            let cp = q
                .regular_market_change_percent
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0);
            let dl = q.regular_market_day_low.as_ref().and_then(|v| v.raw);
            let dh = q.regular_market_day_high.as_ref().and_then(|v| v.raw);
            let vol = q
                .regular_market_volume
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0);
            (p, c, cp, dl, dh, vol, None)
        } else {
            let paragraph = Paragraph::new(vec![
                Line::from(Span::styled(
                    symbol,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Loading quote data...",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(Block::default().borders(Borders::ALL).title("Details"));
            f.render_widget(paragraph, area);
            return;
        };

    let change_sign = if change >= 0.0 { "+" } else { "" };
    let change_color = if change >= 0.0 {
        Color::Green
    } else {
        Color::Red
    };

    let mut left_lines = vec![];
    // Use streaming market_hours when available, otherwise fall back to quote's market_state
    let status = if let Some(market_hours) = streaming_market_hours {
        use finance_query::streaming::MarketHoursType;
        match market_hours {
            MarketHoursType::RegularMarket => ("● LIVE", Color::Green),
            MarketHoursType::PreMarket => {
                // Distinguish overnight (8 PM - 4 AM ET) from pre-market (4 AM - 9:30 AM ET)
                if is_overnight_session() {
                    ("◐ OVERNIGHT", Color::Cyan)
                } else {
                    ("◐ PRE", Color::Yellow)
                }
            }
            MarketHoursType::PostMarket => ("◑ POST", Color::Yellow),
            MarketHoursType::ExtendedHoursMarket => ("◑ EXT", Color::Yellow),
        }
    } else if let Some(q) = quote {
        match q.market_state.as_deref().unwrap_or("CLOSED") {
            "REGULAR" => ("● OPEN", Color::Green),
            "PRE" => ("◐ PRE", Color::Yellow),
            "POST" => ("◑ POST", Color::Yellow),
            _ => ("○ CLOSED", Color::DarkGray),
        }
    } else {
        ("○ CLOSED", Color::DarkGray)
    };

    left_lines.push(Line::from(vec![
        Span::styled(
            symbol,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {}", status.0), Style::default().fg(status.1)),
    ]));

    if let Some(q) = quote
        && let Some(name) = q.short_name.clone().or_else(|| q.long_name.clone())
    {
        left_lines.push(Line::from(Span::styled(
            name,
            Style::default().fg(Color::DarkGray),
        )));
    }

    left_lines.push(Line::from(""));
    left_lines.push(Line::from(vec![
        Span::styled(
            format!("${:.2}", price),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled(
            format!(
                "{}{:.2} ({}{:.2}%)",
                change_sign, change, change_sign, change_pct
            ),
            Style::default()
                .fg(change_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Pre/Post market data right after main price
    if let Some(q) = quote {
        let market_state = q.market_state.as_deref().unwrap_or("");
        if market_state == "PRE" {
            if let Some(pre_price) = q.pre_market_price.as_ref().and_then(|v| v.raw) {
                let pre_pct = q
                    .pre_market_change_percent
                    .as_ref()
                    .and_then(|v| v.raw)
                    .unwrap_or(0.0);
                let pre_color = if pre_pct >= 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };
                left_lines.push(Line::from(vec![
                    Span::styled("Pre-Mkt ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("${:.2} ", pre_price)),
                    Span::styled(
                        format!("({:+.2}%)", pre_pct),
                        Style::default().fg(pre_color),
                    ),
                ]));
            }
        } else if market_state == "POST"
            && let Some(post_price) = q.post_market_price.as_ref().and_then(|v| v.raw)
        {
            let post_pct = q
                .post_market_change_percent
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0);
            let post_color = if post_pct >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            left_lines.push(Line::from(vec![
                Span::styled("After   ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("${:.2} ", post_price)),
                Span::styled(
                    format!("({:+.2}%)", post_pct),
                    Style::default().fg(post_color),
                ),
            ]));
        }
    }

    left_lines.push(Line::from(""));

    if let Some(q) = quote {
        if let Some(o) = q.regular_market_open.as_ref().and_then(|v| v.raw) {
            left_lines.push(Line::from(vec![
                Span::styled("Open        ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("${:.2}", o)),
            ]));
        }
        if let Some(pc) = q.regular_market_previous_close.as_ref().and_then(|v| v.raw) {
            left_lines.push(Line::from(vec![
                Span::styled("Prev Close  ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("${:.2}", pc)),
            ]));
        }
    }
    if let (Some(low), Some(high)) = (day_low, day_high) {
        left_lines.push(Line::from(vec![
            Span::styled("Day Range   ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("${:.2} - ${:.2}", low, high)),
        ]));
    }
    left_lines.push(Line::from(vec![
        Span::styled("Volume      ", Style::default().fg(Color::DarkGray)),
        Span::raw(format_volume(volume)),
    ]));

    if let Some(q) = quote {
        if let (Some(low), Some(high)) = (
            q.fifty_two_week_low.as_ref().and_then(|v| v.raw),
            q.fifty_two_week_high.as_ref().and_then(|v| v.raw),
        ) {
            left_lines.push(Line::from(vec![
                Span::styled("52W Range   ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("${:.2} - ${:.2}", low, high)),
            ]));
        }
        if let Some(avg_vol) = q.average_volume.as_ref().and_then(|v| v.raw) {
            let vol_ratio = if avg_vol > 0 {
                volume as f64 / avg_vol as f64
            } else {
                0.0
            };
            let vol_indicator = if vol_ratio > 1.5 {
                " ▲"
            } else if vol_ratio < 0.5 {
                " ▼"
            } else {
                ""
            };
            left_lines.push(Line::from(vec![
                Span::styled("Avg Volume  ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{}{}", format_volume(avg_vol), vol_indicator)),
            ]));
        }

        // 52 Week Change
        if let Some(w52_change) = q.week_52_change.as_ref().and_then(|v| v.raw) {
            let w52_color = if w52_change >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            left_lines.push(Line::from(vec![
                Span::styled("52W Change  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:+.2}%", w52_change * 100.0),
                    Style::default().fg(w52_color),
                ),
            ]));
        }

        // Bid/Ask spread
        if let (Some(bid), Some(ask)) = (
            q.bid.as_ref().and_then(|v| v.raw),
            q.ask.as_ref().and_then(|v| v.raw),
        ) && bid > 0.0
            && ask > 0.0
        {
            left_lines.push(Line::from(vec![
                Span::styled("Bid/Ask     ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("${:.2} / ${:.2}", bid, ask)),
            ]));
        }

        // Shares outstanding and float
        if let Some(shares) = q.shares_outstanding.as_ref().and_then(|v| v.raw) {
            left_lines.push(Line::from(vec![
                Span::styled("Shares Out  ", Style::default().fg(Color::DarkGray)),
                Span::raw(format_volume(shares)),
            ]));
        }
        if let Some(float) = q.float_shares.as_ref().and_then(|v| v.raw) {
            left_lines.push(Line::from(vec![
                Span::styled("Float       ", Style::default().fg(Color::DarkGray)),
                Span::raw(format_volume(float)),
            ]));
        }

        // Short ratio (days to cover)
        if let Some(short_ratio) = q.short_ratio.as_ref().and_then(|v| v.raw) {
            let sr_color = if short_ratio > 5.0 {
                Color::Red // High short interest
            } else if short_ratio > 2.0 {
                Color::Yellow
            } else {
                Color::White
            };
            left_lines.push(Line::from(vec![
                Span::styled("Short Ratio ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.1} days", short_ratio),
                    Style::default().fg(sr_color),
                ),
            ]));
        }

        // Price to Book
        if let Some(pb) = q.price_to_book.as_ref().and_then(|v| v.raw) {
            left_lines.push(Line::from(vec![
                Span::styled("P/B         ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{:.2}", pb)),
            ]));
        }

        // Ex-Dividend Date
        if let Some(ex_div) = q.ex_dividend_date.as_ref().and_then(|v| v.raw) {
            use chrono::{TimeZone, Utc};
            if let Some(date) = Utc.timestamp_opt(ex_div, 0).single() {
                let today = Utc::now().date_naive();
                let ex_date = date.date_naive();
                let is_upcoming = ex_date >= today;
                let date_color = if is_upcoming {
                    Color::Yellow
                } else {
                    Color::DarkGray
                };
                left_lines.push(Line::from(vec![
                    Span::styled("Ex-Div Date ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        date.format("%Y-%m-%d").to_string(),
                        Style::default().fg(date_color),
                    ),
                ]));
            }
        }
    }

    let trading_height = columns[0].height.saturating_sub(2) as usize; // Account for borders
    let trading_content_len = left_lines.len();
    let trading_max_scroll = trading_content_len.saturating_sub(trading_height);
    let trading_scroll = if trading_max_scroll > 0 {
        (app.detail_scroll[0] as usize) % (trading_max_scroll + 1)
    } else {
        0
    };
    let trading_scroll_indicator = if trading_max_scroll > 0 {
        format!(" [{}↕]", trading_content_len)
    } else {
        String::new()
    };
    let left_paragraph = Paragraph::new(left_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Trading{}", trading_scroll_indicator)),
        )
        .scroll((trading_scroll as u16, 0));
    f.render_widget(left_paragraph, columns[0]);

    let mut right_lines = vec![];
    if let Some(q) = quote {
        right_lines.push(Line::from(Span::styled(
            "─ Valuation ─",
            Style::default().fg(Color::DarkGray),
        )));
        if let Some(mcap) = q.market_cap.as_ref().and_then(|v| v.raw) {
            right_lines.push(Line::from(vec![
                Span::styled("Mkt Cap   ", Style::default().fg(Color::DarkGray)),
                Span::raw(format_market_cap(mcap)),
            ]));
        }
        if let Some(pe) = q.trailing_pe.as_ref().and_then(|v| v.raw) {
            right_lines.push(Line::from(vec![
                Span::styled("P/E TTM   ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{:.2}", pe)),
            ]));
        }
        if let Some(fwd_pe) = q.forward_pe.as_ref().and_then(|v| v.raw) {
            right_lines.push(Line::from(vec![
                Span::styled("P/E Fwd   ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{:.2}", fwd_pe)),
            ]));
        }
        if let Some(eps) = q.trailing_eps.as_ref().and_then(|v| v.raw) {
            right_lines.push(Line::from(vec![
                Span::styled("EPS TTM   ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("${:.2}", eps)),
            ]));
        }
        if let Some(div_yield) = q.dividend_yield.as_ref().and_then(|v| v.raw) {
            right_lines.push(Line::from(vec![
                Span::styled("Div Yld   ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{:.2}%", div_yield * 100.0)),
            ]));
        }
        if let Some(beta) = q.beta.as_ref().and_then(|v| v.raw) {
            right_lines.push(Line::from(vec![
                Span::styled("Beta      ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{:.2}", beta)),
            ]));
        }

        let has_ma = q.fifty_day_average.as_ref().and_then(|v| v.raw).is_some()
            || q.two_hundred_day_average
                .as_ref()
                .and_then(|v| v.raw)
                .is_some();
        if has_ma {
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(Span::styled(
                "─ Technicals ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(ma50) = q.fifty_day_average.as_ref().and_then(|v| v.raw) {
                let vs_ma = if price > ma50 { "▲" } else { "▼" };
                right_lines.push(Line::from(vec![
                    Span::styled("50D MA    ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("${:.2} {}", ma50, vs_ma)),
                ]));
            }
            if let Some(ma200) = q.two_hundred_day_average.as_ref().and_then(|v| v.raw) {
                let vs_ma = if price > ma200 { "▲" } else { "▼" };
                right_lines.push(Line::from(vec![
                    Span::styled("200D MA   ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("${:.2} {}", ma200, vs_ma)),
                ]));
            }
        }

        let has_analyst = q.target_mean_price.as_ref().and_then(|v| v.raw).is_some()
            || q.recommendation_key.is_some();
        if has_analyst {
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(Span::styled(
                "─ Analysts ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(target) = q.target_mean_price.as_ref().and_then(|v| v.raw) {
                let upside = ((target - price) / price) * 100.0;
                let upside_color = if upside > 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };
                right_lines.push(Line::from(vec![
                    Span::styled("Target    ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("${:.2} ", target)),
                    Span::styled(
                        format!("({:+.1}%)", upside),
                        Style::default().fg(upside_color),
                    ),
                ]));
            }
            if let Some(rec) = &q.recommendation_key {
                let rec_color = match rec.to_lowercase().as_str() {
                    "buy" | "strongbuy" => Color::Green,
                    "hold" => Color::Yellow,
                    "sell" | "strongsell" => Color::Red,
                    _ => Color::White,
                };
                right_lines.push(Line::from(vec![
                    Span::styled("Rating    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(rec.to_uppercase(), Style::default().fg(rec_color)),
                ]));
            }
        }

        if q.sector.is_some() || q.industry.is_some() {
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(Span::styled(
                "─ Company ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(sector) = &q.sector {
                right_lines.push(Line::from(vec![
                    Span::styled("Sector    ", Style::default().fg(Color::DarkGray)),
                    Span::raw(sector.clone()),
                ]));
            }
            if let Some(industry) = &q.industry {
                right_lines.push(Line::from(vec![
                    Span::styled("Industry  ", Style::default().fg(Color::DarkGray)),
                    Span::raw(industry.clone()),
                ]));
            }
        }
    } else {
        right_lines.push(Line::from(Span::styled(
            "No fundamental data",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let fund_height = columns[1].height.saturating_sub(2) as usize;
    let fund_content_len = right_lines.len();
    let fund_max_scroll = fund_content_len.saturating_sub(fund_height);
    let fund_scroll = if fund_max_scroll > 0 {
        (app.detail_scroll[1] as usize) % (fund_max_scroll + 1)
    } else {
        0
    };
    let fund_scroll_indicator = if fund_max_scroll > 0 {
        format!(" [{}↕]", fund_content_len)
    } else {
        String::new()
    };
    let right_paragraph = Paragraph::new(right_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Fundamentals{}", fund_scroll_indicator)),
        )
        .scroll((fund_scroll as u16, 0));
    f.render_widget(right_paragraph, columns[1]);

    let mut third_lines = vec![];
    if let Some(q) = quote {
        let has_financials = q.total_revenue.as_ref().and_then(|v| v.raw).is_some()
            || q.profit_margins.as_ref().and_then(|v| v.raw).is_some();
        if has_financials {
            third_lines.push(Line::from(Span::styled(
                "─ Financials ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(rev) = q.total_revenue.as_ref().and_then(|v| v.raw) {
                third_lines.push(Line::from(vec![
                    Span::styled("Revenue   ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format_market_cap(rev)),
                ]));
            }
            if let Some(margin) = q.profit_margins.as_ref().and_then(|v| v.raw) {
                let margin_color = if margin > 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };
                third_lines.push(Line::from(vec![
                    Span::styled("Profit %  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.1}%", margin * 100.0),
                        Style::default().fg(margin_color),
                    ),
                ]));
            }
        }

        let has_efficiency = q.return_on_equity.as_ref().and_then(|v| v.raw).is_some()
            || q.return_on_assets.as_ref().and_then(|v| v.raw).is_some()
            || q.debt_to_equity.as_ref().and_then(|v| v.raw).is_some();
        if has_efficiency {
            if !third_lines.is_empty() {
                third_lines.push(Line::from(""));
            }
            third_lines.push(Line::from(Span::styled(
                "─ Efficiency ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(roe) = q.return_on_equity.as_ref().and_then(|v| v.raw) {
                let roe_color = if roe > 0.15 {
                    Color::Green
                } else if roe > 0.0 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                third_lines.push(Line::from(vec![
                    Span::styled("ROE       ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.1}%", roe * 100.0),
                        Style::default().fg(roe_color),
                    ),
                ]));
            }
            if let Some(roa) = q.return_on_assets.as_ref().and_then(|v| v.raw) {
                let roa_color = if roa > 0.05 {
                    Color::Green
                } else if roa > 0.0 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                third_lines.push(Line::from(vec![
                    Span::styled("ROA       ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.1}%", roa * 100.0),
                        Style::default().fg(roa_color),
                    ),
                ]));
            }
            if let Some(de) = q.debt_to_equity.as_ref().and_then(|v| v.raw) {
                let de_color = if de < 50.0 {
                    Color::Green
                } else if de < 100.0 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                third_lines.push(Line::from(vec![
                    Span::styled("D/E       ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{:.1}", de), Style::default().fg(de_color)),
                ]));
            }
        }

        let has_ownership = q
            .held_percent_insiders
            .as_ref()
            .and_then(|v| v.raw)
            .is_some()
            || q.held_percent_institutions
                .as_ref()
                .and_then(|v| v.raw)
                .is_some()
            || q.short_percent_of_float
                .as_ref()
                .and_then(|v| v.raw)
                .is_some();
        if has_ownership {
            if !third_lines.is_empty() {
                third_lines.push(Line::from(""));
            }
            third_lines.push(Line::from(Span::styled(
                "─ Ownership ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(insider) = q.held_percent_insiders.as_ref().and_then(|v| v.raw) {
                third_lines.push(Line::from(vec![
                    Span::styled("Insider % ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("{:.1}%", insider * 100.0)),
                ]));
            }
            if let Some(inst) = q.held_percent_institutions.as_ref().and_then(|v| v.raw) {
                third_lines.push(Line::from(vec![
                    Span::styled("Instit %  ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("{:.1}%", inst * 100.0)),
                ]));
            }
            if let Some(short_pct) = q.short_percent_of_float.as_ref().and_then(|v| v.raw) {
                let short_color = if short_pct > 0.20 {
                    Color::Red
                } else if short_pct > 0.10 {
                    Color::Yellow
                } else {
                    Color::White
                };
                third_lines.push(Line::from(vec![
                    Span::styled("Short %   ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.1}%", short_pct * 100.0),
                        Style::default().fg(short_color),
                    ),
                ]));
            }
        }

        let has_growth = q.earnings_growth.as_ref().and_then(|v| v.raw).is_some()
            || q.revenue_growth.as_ref().and_then(|v| v.raw).is_some();
        if has_growth {
            if !third_lines.is_empty() {
                third_lines.push(Line::from(""));
            }
            third_lines.push(Line::from(Span::styled(
                "─ Growth ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(eg) = q.earnings_growth.as_ref().and_then(|v| v.raw) {
                let eg_color = if eg > 0.0 { Color::Green } else { Color::Red };
                third_lines.push(Line::from(vec![
                    Span::styled("Earnings  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:+.1}%", eg * 100.0),
                        Style::default().fg(eg_color),
                    ),
                ]));
            }
            if let Some(rg) = q.revenue_growth.as_ref().and_then(|v| v.raw) {
                let rg_color = if rg > 0.0 { Color::Green } else { Color::Red };
                third_lines.push(Line::from(vec![
                    Span::styled("Revenue   ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:+.1}%", rg * 100.0),
                        Style::default().fg(rg_color),
                    ),
                ]));
            }
        }
    }

    if third_lines.is_empty() {
        // Check if we're loading detailed quote for this symbol
        let is_loading = app.is_loading_detailed_quote
            && app.loading_detailed_symbol.as_ref() == Some(&symbol.to_string());

        if is_loading {
            third_lines.push(Line::from(Span::styled(
                "Loading detailed data...",
                Style::default().fg(Color::Yellow),
            )));
        } else {
            third_lines.push(Line::from(Span::styled(
                "Data not available",
                Style::default().fg(Color::DarkGray),
            )));
            third_lines.push(Line::from(Span::styled(
                "for this symbol",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    let growth_height = columns[2].height.saturating_sub(2) as usize;
    let growth_content_len = third_lines.len();
    let growth_max_scroll = growth_content_len.saturating_sub(growth_height);
    let growth_scroll = if growth_max_scroll > 0 {
        (app.detail_scroll[2] as usize) % (growth_max_scroll + 1)
    } else {
        0
    };
    let growth_scroll_indicator = if growth_max_scroll > 0 {
        format!(" [{}↕]", growth_content_len)
    } else {
        String::new()
    };
    let third_paragraph = Paragraph::new(third_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Growth & Ownership{}", growth_scroll_indicator)),
        )
        .scroll((growth_scroll as u16, 0));
    f.render_widget(third_paragraph, columns[2]);
}

fn render_chart(f: &mut Frame, app: &App, area: Rect) {
    let Some(symbol) = app.selected_symbol() else {
        let paragraph = Paragraph::new("No symbol selected")
            .block(Block::default().borders(Borders::ALL).title("Chart"));
        f.render_widget(paragraph, area);
        return;
    };

    let is_focused = app.focus_pane == FocusPane::Right;
    let range_options = ["1D", "5D", "1M", "6M", "YTD", "1Y", "5Y", "Max"];
    let current_range = range_options[app.selected_chart_range_idx];

    if is_focused {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let range_buttons: Vec<Span> = range_options
            .iter()
            .enumerate()
            .flat_map(|(idx, label)| {
                let is_selected = idx == app.selected_chart_range_idx;
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
                        Span::styled(format!(" {} ", label), Style::default().fg(Color::DarkGray))
                    },
                    Span::raw(" "),
                ]
            })
            .collect();

        let range_selector = Paragraph::new(Line::from(range_buttons)).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Range [←/→:change]")
                .border_style(Style::default().fg(Color::Yellow)),
        );
        f.render_widget(range_selector, chunks[0]);

        render_chart_widget(f, app, chunks[1], symbol, current_range);
    } else {
        render_chart_widget(f, app, area, symbol, current_range);
    }
}

fn render_chart_widget(f: &mut Frame, app: &App, area: Rect, symbol: &str, range_label: &str) {
    let Some(ref chart_data) = app.chart_data else {
        let paragraph = Paragraph::new("Loading chart data...").block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} - {} Chart", symbol, range_label)),
        );
        f.render_widget(paragraph, area);
        return;
    };

    if chart_data.is_empty() {
        let paragraph = Paragraph::new("No chart data available").block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} - {} Chart", symbol, range_label)),
        );
        f.render_widget(paragraph, area);
        return;
    }

    let prices: Vec<f64> = chart_data.iter().map(|(_, p)| *p).collect();
    let min_price = prices.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_price = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let padding = (max_price - min_price) * 0.05;
    let y_min = (min_price - padding).max(0.0); // Prices can't be negative
    let y_max = max_price + padding;

    let first_price = prices.first().copied().unwrap_or(0.0);
    let last_price = prices.last().copied().unwrap_or(0.0);
    let line_color = if last_price >= first_price {
        Color::Green
    } else {
        Color::Red
    };

    let dataset = vec![
        Dataset::default()
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(line_color))
            .data(chart_data),
    ];

    let x_min = chart_data.first().map(|(t, _)| *t).unwrap_or(0.0);
    let x_max = chart_data.last().map(|(t, _)| *t).unwrap_or(1.0);

    let y_labels = vec![
        Span::raw(format!("${:.2}", y_min)),
        Span::raw(format!("${:.2}", (y_min + y_max) / 2.0)),
        Span::raw(format!("${:.2}", y_max)),
    ];

    let is_focused = app.focus_pane == FocusPane::Right;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let mut title = format!("{} - {} Chart", symbol, range_label);
    if !is_focused {
        title.push_str(" [l/→:focus]");
    }

    let chart_widget = Chart::new(dataset)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .x_axis(ratatui::widgets::Axis::default().bounds([x_min, x_max]))
        .y_axis(
            ratatui::widgets::Axis::default()
                .bounds([y_min, y_max])
                .labels(y_labels),
        );

    f.render_widget(chart_widget, area);
}

fn render_news(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus_pane == FocusPane::Right;

    // Show loading state
    if app.is_loading_news {
        let paragraph = Paragraph::new("Loading news...")
            .block(Block::default().borders(Borders::ALL).title("News"));
        f.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .news_items
        .iter()
        .enumerate()
        .map(|(idx, article)| {
            let is_selected = is_focused && idx == app.selected_news_idx;

            let title_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let meta_style = if is_selected {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let content = vec![
                Line::from(vec![Span::styled(&article.title, title_style)]),
                Line::from(vec![
                    Span::styled(&article.source, meta_style),
                    Span::styled(" • ", meta_style),
                    Span::styled(&article.time, meta_style),
                ]),
                Line::from(""),
            ];

            ListItem::new(content)
        })
        .collect();

    // Title based on whether showing general or symbol news
    let news_title = match &app.news_symbol {
        Some(symbol) => format!("{} News", symbol),
        None => "Market News".to_string(),
    };

    let mut title = if items.is_empty() {
        format!("{} (no articles)", news_title)
    } else {
        format!(
            "{} ({}/{})",
            news_title,
            app.selected_news_idx + 1,
            items.len()
        )
    };

    // Show appropriate hints based on focus and news type
    if is_focused {
        if app.news_symbol.is_some() {
            title.push_str(" [j/k:scroll Enter:open g:market h:back]");
        } else {
            title.push_str(" [j/k:scroll Enter:open h:back]");
        }
    } else {
        title.push_str(" [Enter:symbol news l/→:focus]");
    }

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    let mut list_state = ListState::default();
    if is_focused && !app.news_items.is_empty() {
        list_state.select(Some(app.selected_news_idx));
    }

    f.render_stateful_widget(list, area, &mut list_state);
}

fn render_lookup(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let input_text = if app.is_searching {
        format!("Searching for '{}'...", app.search_query)
    } else {
        app.search_query.clone()
    };

    let input = Paragraph::new(input_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Search Symbol (type to search, Enter to add)")
            .border_style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(input, chunks[0]);

    let is_focused = app.focus_pane == FocusPane::Right;

    if app.is_searching {
        let loading = Paragraph::new("Searching...")
            .block(Block::default().borders(Borders::ALL).title("Results"));
        f.render_widget(loading, chunks[1]);
    } else if app.search_results.is_empty() {
        let empty_msg = if app.search_query.is_empty() {
            "Type a symbol or company name to search"
        } else {
            "No results found"
        };
        let paragraph = Paragraph::new(empty_msg)
            .block(Block::default().borders(Borders::ALL).title("Results"))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .search_results
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                let is_selected = is_focused && idx == app.selected_search_idx;

                let symbol_style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Cyan)
                };

                let name_style = if is_selected {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };

                let meta_style = if is_selected {
                    Style::default().fg(Color::DarkGray).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let name = result
                    .short_name
                    .clone()
                    .or_else(|| result.long_name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                let type_exch = format!(
                    "{} | {}",
                    result.quote_type.as_deref().unwrap_or(""),
                    result.exch_disp.as_deref().unwrap_or("")
                );

                let content = vec![
                    Line::from(vec![
                        Span::styled(&result.symbol, symbol_style),
                        Span::raw("  "),
                        Span::styled(name, name_style),
                    ]),
                    Line::from(vec![Span::styled(type_exch, meta_style)]),
                    Line::from(""),
                ];

                ListItem::new(content)
            })
            .collect();

        let title = format!("Results ({}) [Enter:add to watchlist]", items.len());
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        let mut list_state = ListState::default();
        if is_focused && !app.search_results.is_empty() {
            list_state.select(Some(app.selected_search_idx));
        }

        f.render_stateful_widget(list, chunks[1], &mut list_state);
    }
}

fn render_screeners(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let is_focused = app.focus_pane == FocusPane::Right;

    let categories = ScreenerCategory::ALL;
    let category_buttons: Vec<Span> = categories
        .iter()
        .flat_map(|cat| {
            let is_selected = *cat == app.screener_category;
            vec![
                if is_selected {
                    Span::styled(
                        format!(" {} ", cat.title()),
                        Style::default()
                            .bg(Color::Blue)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(
                        format!(" {} ", cat.title()),
                        Style::default().fg(Color::DarkGray),
                    )
                },
                Span::raw(" "),
            ]
        })
        .collect();

    let selector_border = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let category_title = format!("{} [←/→:change]", app.screener_category.description());
    let category_selector = Paragraph::new(Line::from(category_buttons)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(category_title)
            .border_style(selector_border),
    );
    f.render_widget(category_selector, chunks[0]);

    if app.is_loading_screeners {
        let loading = Paragraph::new("Loading...")
            .block(Block::default().borders(Borders::ALL).title("Screeners"));
        f.render_widget(loading, chunks[1]);
        return;
    }

    if app.screener_data.is_empty() {
        let empty = Paragraph::new("No data available. Press 'r' to refresh.")
            .block(Block::default().borders(Borders::ALL).title("Screeners"))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty, chunks[1]);
        return;
    }

    let items: Vec<ListItem> = app
        .screener_data
        .iter()
        .enumerate()
        .map(|(idx, quote)| {
            let is_selected = is_focused && idx == app.selected_screener_idx;

            let price = quote.regular_market_price.raw.unwrap_or(0.0);
            let change = quote.regular_market_change.raw.unwrap_or(0.0);
            let change_pct = quote.regular_market_change_percent.raw.unwrap_or(0.0);
            let volume = quote
                .regular_market_volume
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0);
            let market_cap = quote.market_cap.as_ref().and_then(|v| v.raw);

            let change_color = if change >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            let arrow = if change >= 0.0 { "▲" } else { "▼" };

            let symbol_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            };

            let name_style = if is_selected {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            let price_style = if is_selected {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            let change_style = if is_selected {
                Style::default().fg(change_color).bg(Color::DarkGray)
            } else {
                Style::default().fg(change_color)
            };

            let meta_style = if is_selected {
                Style::default().fg(Color::DarkGray).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let name = quote
                .display_name
                .clone()
                .or_else(|| Some(quote.short_name.clone()))
                .unwrap_or_default();
            let name_display = if name.len() > 25 {
                format!("{}...", &name[..22])
            } else {
                name
            };

            let mcap_str = market_cap.map(format_market_cap).unwrap_or_default();

            let content = vec![
                Line::from(vec![
                    Span::styled(format!("{:<6}", quote.symbol), symbol_style),
                    Span::styled(format!(" {:<25}", name_display), name_style),
                    Span::styled(format!(" ${:>8.2}", price), price_style),
                    Span::styled(format!("  {:>+6.2}% {}", change_pct, arrow), change_style),
                ]),
                Line::from(vec![Span::styled(
                    format!("      Vol: {}  MCap: {}", format_volume(volume), mcap_str),
                    meta_style,
                )]),
                Line::from(""),
            ];

            ListItem::new(content)
        })
        .collect();

    let title = format!(
        "{} ({}/{}) [Enter:add j/k:nav]",
        app.screener_category.title(),
        if app.screener_data.is_empty() {
            0
        } else {
            app.selected_screener_idx + 1
        },
        app.screener_data.len()
    );

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    let mut list_state = ListState::default();
    if is_focused && !app.screener_data.is_empty() {
        list_state.select(Some(app.selected_screener_idx));
    }

    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    use crate::alerts::AlertType;

    let available_width = area.width.saturating_sub(2) as usize; // Account for borders

    let status_text = match app.input_mode {
        InputMode::Normal => {
            let update_time = app
                .last_update
                .map(|t| t.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let focus_hint = match app.active_tab {
                Tab::Alerts => {
                    // Alerts tab uses full width, no left/right panes
                    "j/k:nav a:add d:del e:toggle c:check"
                }
                Tab::Portfolio => {
                    // Portfolio tab: left pane is portfolio list, not watchlist
                    if app.focus_pane == FocusPane::Left {
                        "j/k:nav a:add d:delete"
                    } else {
                        "Esc:back"
                    }
                }
                _ => {
                    // Other tabs have watchlist on left, details on right
                    if app.focus_pane == FocusPane::Left {
                        match app.active_tab {
                            Tab::Watchlist => "j/k:nav a:add d:del 1/2/3:scroll 0:reset",
                            _ => "j/k:nav a:add d:delete l/→:focus",
                        }
                    } else {
                        match app.active_tab {
                            Tab::Charts => "←/→:range Esc/h:back",
                            Tab::News => "j/k:scroll Enter:open Esc/h:back",
                            Tab::Watchlist => "Esc/h:back",
                            Tab::Lookup => "j/k:nav Enter:add Esc/h:back",
                            Tab::Screeners => "←/→:cat j/k:nav Enter:add Esc/h:back",
                            _ => "Esc:back",
                        }
                    }
                }
            };

            // Truncate status message if too long
            let max_msg_len = available_width.saturating_sub(60);
            let msg = if app.status_message.len() > max_msg_len && max_msg_len > 3 {
                format!("{}...", &app.status_message[..max_msg_len - 3])
            } else {
                app.status_message.clone()
            };

            let symbol_count = match app.active_tab {
                Tab::Alerts => format!("{} alerts", app.alerts.len()),
                Tab::Portfolio => format!("{} positions", app.portfolio.positions.len()),
                _ => format!("{} symbols", app.current_watchlist.symbols.len()),
            };

            format!(
                " {} | {} | Last: {} | {}",
                msg, symbol_count, update_time, focus_hint
            )
        }
        InputMode::AddSymbol => {
            format!(
                " Add symbol: {} | Enter:confirm Esc:cancel",
                app.input_buffer
            )
        }
        InputMode::AddAlert => {
            let all_types = AlertType::all();
            let alert_type = all_types[app.alert_form_type_idx];

            let symbol_field = if app.alert_form_field == 0 {
                format!("[{}▌]", app.alert_form_symbol)
            } else if app.alert_form_symbol.is_empty() {
                "[SYM]".to_string()
            } else {
                app.alert_form_symbol.clone()
            };
            let type_field = if app.alert_form_field == 1 {
                format!("<{}>", alert_type.short_display())
            } else {
                alert_type.short_display().to_string()
            };
            let threshold_field = if app.alert_form_field == 2 {
                format!("[{}▌]", app.alert_form_threshold)
            } else if app.alert_form_threshold.is_empty() {
                "[VAL]".to_string()
            } else {
                app.alert_form_threshold.clone()
            };
            format!(
                " {} {} {} | Tab:next ←/→:type Enter:create Esc:cancel",
                symbol_field, type_field, threshold_field
            )
        }
        InputMode::AddPosition => " Adding position... | Esc:cancel".to_string(),
    };

    // Final truncation to fit width (character-aware for UTF-8)
    let final_text = if status_text.chars().count() > available_width {
        let truncated: String = status_text
            .chars()
            .take(available_width.saturating_sub(1))
            .collect();
        format!("{}…", truncated)
    } else {
        status_text
    };

    let paragraph = Paragraph::new(final_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

fn render_portfolio(f: &mut Frame, app: &App, area: Rect) {
    use ratatui::widgets::{Row, Table};

    if app.input_mode == InputMode::AddPosition {
        render_add_position_form(f, app, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(5)])
        .split(area);

    // Portfolio table
    if app.portfolio.is_empty() {
        let empty_msg = Paragraph::new("No positions.\n\nPress 'a' to add a position.")
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Portfolio Positions"),
            );
        f.render_widget(empty_msg, chunks[0]);
    } else {
        let header = Row::new(vec![
            "Symbol",
            "Shares",
            "Cost Basis",
            "Current",
            "Value",
            "P/L",
            "P/L %",
        ])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        let rows: Vec<Row> = app
            .portfolio
            .positions
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let current_price = app.portfolio_prices.get(&p.symbol).copied().unwrap_or(0.0);
                let current_value = p.current_value(current_price);
                let pl = p.profit_loss(current_price);
                let pl_percent = p.profit_loss_percent(current_price);

                let style = if i == app.selected_portfolio_idx {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                Row::new(vec![
                    p.symbol.clone(),
                    format!("{:.2}", p.shares),
                    format!("${:.2}", p.cost_basis),
                    if current_price > 0.0 {
                        format!("${:.2}", current_price)
                    } else {
                        "N/A".to_string()
                    },
                    format!("${:.2}", current_value),
                    format!("${:.2}", pl),
                    format!("{:.2}%", pl_percent),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(10),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Portfolio Positions"),
        );

        f.render_widget(table, chunks[0]);
    }

    // Summary
    let total_cost = app.portfolio.total_cost();
    let total_value = app.total_portfolio_value();
    let total_pl = app.total_portfolio_profit_loss();
    let total_pl_percent = app.total_portfolio_profit_loss_percent();

    let pl_color = if total_pl >= 0.0 {
        Color::Green
    } else {
        Color::Red
    };

    let summary = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Total Cost: "),
            Span::styled(
                format!("${:.2}", total_cost),
                Style::default().fg(Color::White),
            ),
            Span::raw("  |  Total Value: "),
            Span::styled(
                format!("${:.2}", total_value),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::raw("Total P/L: "),
            Span::styled(
                format!("${:.2} ({:.2}%)", total_pl, total_pl_percent),
                Style::default().fg(pl_color).add_modifier(Modifier::BOLD),
            ),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Summary"));

    f.render_widget(summary, chunks[1]);
}

fn render_add_position_form(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let symbol_style = if app.add_form_field == 0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let symbol = Paragraph::new(app.add_form_symbol.as_str())
        .style(symbol_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Symbol (e.g., AAPL)"),
        );
    f.render_widget(symbol, chunks[0]);

    let shares_style = if app.add_form_field == 1 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let shares = Paragraph::new(app.add_form_shares.as_str())
        .style(shares_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Shares (e.g., 10)"),
        );
    f.render_widget(shares, chunks[1]);

    let cost_style = if app.add_form_field == 2 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let cost = Paragraph::new(app.add_form_cost.as_str())
        .style(cost_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Cost Basis per Share (e.g., 150.50)"),
        );
    f.render_widget(cost, chunks[2]);
}

fn format_market_cap(mcap: i64) -> String {
    match mcap {
        v if v >= 1_000_000_000_000 => format!("${:.2}T", v as f64 / 1_000_000_000_000.0),
        v if v >= 1_000_000_000 => format!("${:.2}B", v as f64 / 1_000_000_000.0),
        v if v >= 1_000_000 => format!("${:.2}M", v as f64 / 1_000_000.0),
        _ => format!("${}", mcap),
    }
}

fn format_volume(volume: i64) -> String {
    match volume {
        v if v >= 1_000_000_000 => format!("{:.2}B", v as f64 / 1_000_000_000.0),
        v if v >= 1_000_000 => format!("{:.2}M", v as f64 / 1_000_000.0),
        v if v >= 1_000 => format!("{:.2}K", v as f64 / 1_000.0),
        _ => volume.to_string(),
    }
}

/// Check if current time is during overnight trading session (8 PM - 4 AM ET)
fn is_overnight_session() -> bool {
    use chrono::{Timelike, Utc};
    use chrono_tz::America::New_York;

    let now_et = Utc::now().with_timezone(&New_York);
    let hour = now_et.hour();

    // Overnight: 8 PM (20:00) to 4 AM (04:00) ET
    !(4..20).contains(&hour)
}

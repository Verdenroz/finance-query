use super::util::{get_change_color, truncate_str};
use crate::dashboard::state::{App, SectorsViewMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub(super) fn render_sectors_tab(f: &mut Frame, app: &App, area: Rect) {
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

use chrono::Weekday;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::PeriodsMode;
use super::ResultsApp;

pub(super) fn render_periods(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    use ratatui::widgets::Row;
    use ratatui::widgets::Table;
    let r = &app.result.backtest;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let header_para = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" Breakdown by {} ", app.periods_mode.label()),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " (m: cycle yearly/monthly/day-of-week  ↑/↓: scroll)",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    f.render_widget(header_para, chunks[0]);

    match app.periods_mode {
        PeriodsMode::Monthly => {
            let mut data = r.by_month().into_iter().collect::<Vec<_>>();
            data.sort_by_key(|(k, _)| *k);

            let header = Row::new(vec!["Month", "Return %", "Win Rate", "Trades", "Max DD %"])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1);

            let visible_rows: Vec<Row> = data
                .iter()
                .skip(app.scroll)
                .take(chunks[1].height.saturating_sub(3) as usize)
                .map(|((year, month), m)| {
                    let ret = m.total_return_pct;
                    let color = if ret >= 0.0 { Color::Green } else { Color::Red };
                    Row::new(vec![
                        format!("{}-{:02}", year, month),
                        format!("{:+.2}%", ret),
                        format!("{:.1}%", m.win_rate * 100.0),
                        m.total_trades.to_string(),
                        format!("{:.2}%", m.max_drawdown_pct * 100.0),
                    ])
                    .style(Style::default().fg(color))
                })
                .collect();

            let table = Table::new(
                visible_rows,
                [
                    Constraint::Length(10),
                    Constraint::Length(12),
                    Constraint::Length(10),
                    Constraint::Length(8),
                    Constraint::Length(10),
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(format!(" Monthly ({}/{}) ", app.scroll + 1, data.len())),
            );
            f.render_widget(table, chunks[1]);
        }
        PeriodsMode::Yearly => {
            let mut data = r.by_year().into_iter().collect::<Vec<_>>();
            data.sort_by_key(|(k, _)| *k);

            let header = Row::new(vec!["Year", "Return %", "Win Rate", "Trades", "Max DD %"])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1);

            let visible_rows: Vec<Row> = data
                .iter()
                .skip(app.scroll)
                .take(chunks[1].height.saturating_sub(3) as usize)
                .map(|(year, m)| {
                    let ret = m.total_return_pct;
                    let color = if ret >= 0.0 { Color::Green } else { Color::Red };
                    Row::new(vec![
                        year.to_string(),
                        format!("{:+.2}%", ret),
                        format!("{:.1}%", m.win_rate * 100.0),
                        m.total_trades.to_string(),
                        format!("{:.2}%", m.max_drawdown_pct * 100.0),
                    ])
                    .style(Style::default().fg(color))
                })
                .collect();

            let table = Table::new(
                visible_rows,
                [
                    Constraint::Length(6),
                    Constraint::Length(12),
                    Constraint::Length(10),
                    Constraint::Length(8),
                    Constraint::Length(10),
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(format!(" Yearly ({}/{}) ", app.scroll + 1, data.len())),
            );
            f.render_widget(table, chunks[1]);
        }
        PeriodsMode::DayOfWeek => {
            // Canonical Mon–Sun ordering; Sat/Sun matter for 24/7 assets (crypto)
            const DOW_ORDER: [Weekday; 7] = [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ];

            let dow_map = r.by_day_of_week();
            let data: Vec<(Weekday, &finance_query::backtesting::PerformanceMetrics)> = DOW_ORDER
                .iter()
                .filter_map(|d| dow_map.get(d).map(|m| (*d, m)))
                .collect();

            if data.is_empty() {
                let msg = Paragraph::new("Not enough trades to compute day-of-week breakdown.")
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Day of Week "),
                    );
                f.render_widget(msg, chunks[1]);
                return;
            }

            let header = Row::new(vec!["Day", "Return %", "Win Rate", "Trades", "Avg Trade %"])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1);

            let rows: Vec<Row> = data
                .iter()
                .map(|(day, m)| {
                    let ret = m.total_return_pct;
                    let color = if ret >= 0.0 { Color::Green } else { Color::Red };
                    Row::new(vec![
                        format!("{}", day),
                        format!("{:+.2}%", ret),
                        format!("{:.1}%", m.win_rate * 100.0),
                        m.total_trades.to_string(),
                        format!("{:+.2}%", m.avg_trade_return_pct),
                    ])
                    .style(Style::default().fg(color))
                })
                .collect();

            let table = Table::new(
                rows,
                [
                    Constraint::Length(10),
                    Constraint::Length(12),
                    Constraint::Length(10),
                    Constraint::Length(8),
                    Constraint::Length(12),
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Day of Week "),
            );
            f.render_widget(table, chunks[1]);
        }
    }
}

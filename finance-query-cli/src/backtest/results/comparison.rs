use finance_query::backtesting::{BacktestComparison, OptimizeMetric};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::ResultsApp;

pub(super) fn render_comparison(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    use ratatui::widgets::Row;
    use ratatui::widgets::Table;

    if app.saved_results.len() < 2 {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                " Save at least 2 results with 'c' to compare them.",
                Style::default().fg(Color::DarkGray),
            )]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Comparison "),
        );
        f.render_widget(msg, area);
        return;
    }

    let mut cmp = BacktestComparison::new();
    for (label, result) in &app.saved_results {
        cmp = cmp.add(label.clone(), result.clone());
    }
    let report = cmp.ranked_by(app.comparison_metric);

    let metric_name = match app.comparison_metric {
        OptimizeMetric::SharpeRatio => "Sharpe",
        OptimizeMetric::SortinoRatio => "Sortino",
        OptimizeMetric::TotalReturn => "Return",
        OptimizeMetric::WinRate => "Win Rate",
        OptimizeMetric::ProfitFactor => "Profit Factor",
        OptimizeMetric::MinDrawdown => "Max DD",
        OptimizeMetric::CalmarRatio => "Calmar",
        _ => "Sharpe",
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let header_para = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" Ranked by: {} ", metric_name),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " (press m to cycle metric, c to add current run, x to clear)",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    f.render_widget(header_para, chunks[0]);

    let header = Row::new(vec![
        "Rank", "Label", "Return %", "Sharpe", "Sortino", "Max DD %", "Win Rate", "SQN",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let rows: Vec<Row> = report
        .table()
        .iter()
        .map(|row| {
            let color = if row.total_return_pct >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            Row::new(vec![
                format!("#{}", row.rank),
                row.label.clone(),
                format!("{:+.2}%", row.total_return_pct),
                format!("{:.2}", row.sharpe_ratio),
                format!("{:.2}", row.sortino_ratio),
                format!("{:.2}%", row.max_drawdown_pct * 100.0),
                format!("{:.1}%", row.win_rate * 100.0),
                format!("{:.2}", row.sqn),
            ])
            .style(Style::default().fg(color))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Min(20),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(6),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(format!(" Comparison ({} runs) ", report.len())),
    );

    f.render_widget(table, chunks[1]);
}

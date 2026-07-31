use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{BarChart, Block, Borders, Paragraph, Wrap},
};

use super::ResultsApp;
use super::format::{metric_line, percentile};

// Distribution tab: trade P&L histogram

pub(super) fn render_distribution(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let trades = &app.result.backtest.trades;

    if trades.is_empty() {
        let msg = Paragraph::new("No trades to display.").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" P&L Distribution "),
        );
        f.render_widget(msg, area);
        return;
    }

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    render_pnl_histogram(f, app, split[0]);
    render_distribution_stats(f, app, split[1]);
}

fn render_pnl_histogram(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let trades = &app.result.backtest.trades;
    let pnls: Vec<f64> = trades.iter().map(|t| t.pnl).collect();

    let min_pnl = pnls.iter().cloned().fold(f64::MAX, f64::min);
    let max_pnl = pnls.iter().cloned().fold(f64::MIN, f64::max);

    const BINS: usize = 10;
    let range = (max_pnl - min_pnl).max(1e-9);
    let bin_width = range / BINS as f64;

    let mut counts = [0u64; BINS];
    for &p in &pnls {
        let idx = ((p - min_pnl) / bin_width).floor() as usize;
        let idx = idx.min(BINS - 1);
        counts[idx] += 1;
    }

    let bar_data: Vec<(String, u64)> = counts
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let center = min_pnl + bin_width * (i as f64 + 0.5);
            let label = if center >= 0.0 {
                format!("+{:.0}", center)
            } else {
                format!("{:.0}", center)
            };
            (label, c)
        })
        .collect();

    let bar_refs: Vec<(&str, u64)> = bar_data
        .iter()
        .map(|(label, count)| (label.as_str(), *count))
        .collect();

    let max_count = counts.iter().cloned().max().unwrap_or(1).max(1);

    let chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" P&L Distribution (trade count per bucket) "),
        )
        .data(&bar_refs)
        .bar_width(((area.width.saturating_sub(4)) / BINS as u16).max(3))
        .bar_gap(1)
        .max(max_count)
        .bar_style(Style::default().fg(Color::Cyan))
        .value_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .label_style(Style::default().fg(Color::DarkGray));

    f.render_widget(chart, area);
}

fn render_distribution_stats(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let trades = &app.result.backtest.trades;
    let pnls: Vec<f64> = trades.iter().map(|t| t.pnl).collect();

    let mut sorted = pnls.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted.len();
    let median = if n.is_multiple_of(2) {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    };

    let mean = pnls.iter().sum::<f64>() / n as f64;
    let variance = if n > 1 {
        pnls.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / (n - 1) as f64
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    let p25 = percentile(&sorted, 0.25);
    let p75 = percentile(&sorted, 0.75);

    let m = &app.result.backtest.metrics;
    let lines = vec![
        Line::from(""),
        metric_line("Trades", &n.to_string()),
        metric_line("Mean P&L", &format!("${:.2}", mean)),
        metric_line("Median P&L", &format!("${:.2}", median)),
        metric_line("Std Dev", &format!("${:.2}", std_dev)),
        Line::from(""),
        metric_line("p25", &format!("${:.2}", p25)),
        metric_line("p75", &format!("${:.2}", p75)),
        Line::from(""),
        metric_line(
            "Wins",
            &format!("{} ({:.0}%)", m.winning_trades, m.win_rate * 100.0),
        ),
        metric_line("Losses", &format!("{}", m.losing_trades)),
    ];

    let stats = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" Stats "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(stats, area);
}

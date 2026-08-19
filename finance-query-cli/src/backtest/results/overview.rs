use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::ResultsApp;
use super::format::{format_duration_secs, format_ratio, metric_line};

pub(super) fn render_results_overview(
    f: &mut Frame,
    app: &ResultsApp,
    area: ratatui::layout::Rect,
) {
    let r = &app.result.backtest;
    let m = &r.metrics;
    let pnl = r.total_pnl();

    // Split vertically if benchmark data is present
    let has_benchmark = r.benchmark.is_some();
    let (metrics_area, bench_area) = if has_benchmark {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(area);
        (vert[0], Some(vert[1]))
    } else {
        (area, None)
    };

    // 3-column metrics layout
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(metrics_area);

    // Column 1 — Performance
    let perf_lines = vec![
        Line::from(""),
        metric_line("Start Capital", &format!("${:.2}", r.initial_capital)),
        metric_line("End Capital", &format!("${:.2}", r.final_equity)),
        metric_line(
            "Total P&L",
            &format!("{}{:.2}", if pnl >= 0.0 { "+" } else { "" }, pnl),
        ),
        metric_line(
            "Return",
            &format!(
                "{}{:.2}%",
                if m.total_return_pct >= 0.0 { "+" } else { "" },
                m.total_return_pct
            ),
        ),
        metric_line(
            "Ann. Return",
            &format!(
                "{}{:.2}%",
                if m.annualized_return_pct >= 0.0 {
                    "+"
                } else {
                    ""
                },
                m.annualized_return_pct
            ),
        ),
        Line::from(""),
        metric_line("Total Trades", &m.total_trades.to_string()),
        metric_line("Long Trades", &m.long_trades.to_string()),
        metric_line("Short Trades", &m.short_trades.to_string()),
        metric_line("Win Rate", &format!("{:.1}%", m.win_rate * 100.0)),
    ];

    let perf = Paragraph::new(perf_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Performance "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(perf, cols[0]);

    // Column 2 — Risk
    let max_dd_dollars = max_drawdown_dollars(r.equity_curve.iter().map(|p| p.equity));

    let risk_lines = vec![
        Line::from(""),
        metric_line("Profit Factor", &format_ratio(m.profit_factor)),
        metric_line("Max Drawdown", &format!("${:.2}", max_dd_dollars)),
        metric_line(
            "Max Drawdown %",
            &format!("{:.2}%", m.max_drawdown_pct * 100.0),
        ),
        metric_line("DD Duration", &format!("{} bars", m.max_drawdown_duration)),
        Line::from(""),
        metric_line(
            &format!("Sharpe (RF {:.1}%)", r.config.risk_free_rate * 100.0),
            &format_ratio(m.sharpe_ratio),
        ),
        metric_line(
            &format!("Sortino (RF {:.1}%)", r.config.risk_free_rate * 100.0),
            &format_ratio(m.sortino_ratio),
        ),
        metric_line("Calmar Ratio", &format_ratio(m.calmar_ratio)),
        metric_line("Omega Ratio", &format_ratio(m.omega_ratio)),
        metric_line("Serenity Ratio", &format_ratio(m.serenity_ratio)),
        Line::from(""),
        metric_line("Ulcer Index", &format!("{:.4}", m.ulcer_index)),
        metric_line("Tail Ratio", &format_ratio(m.tail_ratio)),
        metric_line("Recovery Factor", &format_ratio(m.recovery_factor)),
        Line::from(""),
        metric_line("Avg Win %", &format!("{:.2}%", m.avg_win_pct)),
        metric_line("Avg Loss %", &format!("{:.2}%", m.avg_loss_pct)),
        metric_line("Avg Trade Ret", &format!("{:.2}%", m.avg_trade_return_pct)),
        metric_line("Avg Win Dur", &format_duration_secs(m.avg_win_duration)),
        metric_line("Avg Loss Dur", &format_duration_secs(m.avg_loss_duration)),
    ];

    let risk = Paragraph::new(risk_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Risk Metrics "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(risk, cols[1]);

    // Column 3 — Activity
    let mut activity_lines = vec![
        Line::from(""),
        metric_line("Winning Trades", &m.winning_trades.to_string()),
        metric_line("Losing Trades", &m.losing_trades.to_string()),
        metric_line("Max Consec Wins", &m.max_consecutive_wins.to_string()),
        metric_line("Max Consec Loss", &m.max_consecutive_losses.to_string()),
        Line::from(""),
        metric_line(
            "Time in Market",
            &format!("{:.1}%", m.time_in_market_pct * 100.0),
        ),
        metric_line(
            "Avg Trade Dur",
            &format!("{:.1} bars", m.avg_trade_duration),
        ),
        metric_line(
            "Max Idle Period",
            &format_duration_secs(m.max_idle_period as f64),
        ),
        Line::from(""),
        metric_line("Commission Paid", &format!("${:.2}", m.total_commission)),
    ];

    if m.total_dividend_income > 0.0 {
        activity_lines.push(metric_line(
            "Dividend Income",
            &format!("${:.2}", m.total_dividend_income),
        ));
    }

    activity_lines.extend([
        Line::from(""),
        metric_line("Largest Win", &format!("${:.2}", m.largest_win)),
        metric_line("Largest Loss", &format!("${:.2}", m.largest_loss)),
        Line::from(""),
        metric_line(
            "Kelly Criterion",
            &format!("{:.2}%", m.kelly_criterion * 100.0),
        ),
        metric_line("SQN", &format!("{:.2}", m.sqn)),
        metric_line("Expectancy", &format!("${:.2}", m.expectancy)),
        Line::from(""),
        metric_line("Total Signals", &m.total_signals.to_string()),
        metric_line("Executed", &m.executed_signals.to_string()),
    ]);

    let activity = Paragraph::new(activity_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" Activity "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(activity, cols[2]);

    // Optional benchmark section
    if let (Some(bench), Some(ba)) = (&r.benchmark, bench_area) {
        let bench_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(ba);

        let left_lines = vec![
            Line::from(""),
            metric_line("Benchmark", &bench.symbol),
            metric_line(
                "Benchmark Return",
                &format!("{:.2}%", bench.benchmark_return_pct),
            ),
            metric_line(
                "Buy & Hold Ret",
                &format!("{:.2}%", bench.buy_and_hold_return_pct),
            ),
        ];

        let right_lines = vec![
            Line::from(""),
            metric_line("Alpha", &format!("{:.2}%", bench.alpha)),
            metric_line("Beta", &format!("{:.3}", bench.beta)),
            metric_line("Info Ratio", &format_ratio(bench.information_ratio)),
        ];

        let bench_left = Paragraph::new(left_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .title(" Benchmark "),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(bench_left, bench_cols[0]);

        let bench_right = Paragraph::new(right_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .title(" Alpha / Beta "),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(bench_right, bench_cols[1]);
    }
}

fn max_drawdown_dollars(equities: impl IntoIterator<Item = f64>) -> f64 {
    let mut peak = f64::NEG_INFINITY;
    let mut max_dd: f64 = 0.0;
    let mut has_any = false;

    for equity in equities {
        has_any = true;
        peak = peak.max(equity);
        max_dd = max_dd.max((peak - equity).max(0.0));
    }

    if has_any { max_dd } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_drawdown_dollars_uses_peak_to_trough() {
        let equities = vec![10_000.0, 12_000.0, 9_000.0, 11_000.0];
        assert!((max_drawdown_dollars(equities.into_iter()) - 3_000.0).abs() < 1e-9);
    }
}

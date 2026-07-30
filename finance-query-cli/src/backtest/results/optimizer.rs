use finance_query::backtesting::{BacktestResult, OptimizeMetric};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use super::ResultsApp;
use super::format::{format_ratio, metric_line, return_color};

pub(super) fn next_optimize_metric(m: OptimizeMetric) -> OptimizeMetric {
    match m {
        OptimizeMetric::SharpeRatio => OptimizeMetric::SortinoRatio,
        OptimizeMetric::SortinoRatio => OptimizeMetric::TotalReturn,
        OptimizeMetric::TotalReturn => OptimizeMetric::WinRate,
        OptimizeMetric::WinRate => OptimizeMetric::ProfitFactor,
        OptimizeMetric::ProfitFactor => OptimizeMetric::MinDrawdown,
        OptimizeMetric::MinDrawdown => OptimizeMetric::CalmarRatio,
        OptimizeMetric::CalmarRatio => OptimizeMetric::SharpeRatio,
        _ => OptimizeMetric::SharpeRatio,
    }
}

/// Returns the short label and formatted value for the selected optimize metric.
fn metric_score_display(metric: OptimizeMetric, result: &BacktestResult) -> (String, String) {
    match metric {
        OptimizeMetric::SharpeRatio => (
            "Sharpe".to_string(),
            format_ratio(result.metrics.sharpe_ratio),
        ),
        OptimizeMetric::TotalReturn => (
            "Return".to_string(),
            format!("{:+.2}%", result.metrics.total_return_pct),
        ),
        OptimizeMetric::SortinoRatio => (
            "Sortino".to_string(),
            format_ratio(result.metrics.sortino_ratio),
        ),
        OptimizeMetric::CalmarRatio => (
            "Calmar".to_string(),
            format_ratio(result.metrics.calmar_ratio),
        ),
        OptimizeMetric::ProfitFactor => (
            "Prof.Factor".to_string(),
            format_ratio(result.metrics.profit_factor),
        ),
        OptimizeMetric::WinRate => (
            "Win Rate".to_string(),
            format!("{:.1}%", result.metrics.win_rate * 100.0),
        ),
        OptimizeMetric::MinDrawdown => (
            "Drawdown".to_string(),
            format!("{:.2}%", result.metrics.max_drawdown_pct * 100.0),
        ),
        _ => ("Score".to_string(), "N/A".to_string()),
    }
}

// Optimizer tab: best params + ranked results list

pub(super) fn render_optimizer_results(
    f: &mut Frame,
    app: &ResultsApp,
    area: ratatui::layout::Rect,
) {
    let Some(ref opt) = app.result.optimization else {
        let msg = Paragraph::new("No optimization data.")
            .block(Block::default().borders(Borders::ALL).title(" Optimizer "));
        f.render_widget(msg, area);
        return;
    };

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(area);

    // Summary
    let best = &opt.best;
    let mut params_parts: Vec<String> = best
        .params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    params_parts.sort();
    let params_str = params_parts.join("  ");

    let opt_metric = app.result.opt_metric.unwrap_or(OptimizeMetric::SharpeRatio);
    let (best_metric_label, best_metric_value) = metric_score_display(opt_metric, &best.result);

    let summary_lines = vec![
        Line::from(""),
        metric_line("Strategy", &opt.strategy_name),
        metric_line("Total Combos", &opt.total_combinations.to_string()),
        metric_line("Best Params", &params_str),
        metric_line(&format!("Best {}", best_metric_label), &best_metric_value),
        metric_line(
            "Best Return",
            &format!("{:+.2}%", best.result.metrics.total_return_pct),
        ),
    ];

    let summary = Paragraph::new(summary_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Optimizer Summary "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(summary, split[0]);

    // Ranked results list
    let visible: Vec<ListItem> = opt
        .results
        .iter()
        .skip(app.scroll)
        .take(split[1].height.saturating_sub(2) as usize)
        .enumerate()
        .map(|(i, res)| {
            let rank = app.scroll + i + 1;
            let mut parts: Vec<String> = res
                .params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            parts.sort();
            let p_str = parts.join(" ");
            let ret_color = return_color(res.result.metrics.total_return_pct);

            let (col_label, col_value) = metric_score_display(opt_metric, &res.result);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" #{:<3}", rank),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(format!("{:<40}", p_str), Style::default().fg(Color::White)),
                Span::styled(
                    format!(" {}: ", col_label),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(col_value, Style::default().fg(Color::Cyan)),
                Span::styled(" Ret: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:+.2}%", res.result.metrics.total_return_pct),
                    Style::default().fg(ret_color),
                ),
            ]))
        })
        .collect();

    let list = List::new(visible).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(format!(
                " Ranked Results ({}) - ↑/↓ scroll ",
                opt.results.len()
            )),
    );
    f.render_widget(list, split[1]);
}

// Walk-forward tab: aggregate metrics + per-window IS/OOS results

pub(super) fn render_walk_forward_results(
    f: &mut Frame,
    app: &ResultsApp,
    area: ratatui::layout::Rect,
) {
    let Some(ref wf) = app.result.walk_forward else {
        let msg = Paragraph::new("No walk-forward data.").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Walk-Forward "),
        );
        f.render_widget(msg, area);
        return;
    };

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(area);

    // Aggregate summary
    let m = &wf.aggregate_metrics;
    let summary_lines = vec![
        Line::from(""),
        metric_line("Windows", &wf.windows.len().to_string()),
        metric_line(
            "OOS Consistency",
            &format!("{:.1}%", wf.consistency_ratio * 100.0),
        ),
        metric_line("Agg Return", &format!("{:+.2}%", m.total_return_pct)),
        metric_line("Agg Sharpe", &format_ratio(m.sharpe_ratio)),
        metric_line("Agg Max DD", &format!("{:.2}%", m.max_drawdown_pct * 100.0)),
    ];

    let summary = Paragraph::new(summary_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Walk-Forward Summary "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(summary, split[0]);

    // Per-window list
    let visible: Vec<ListItem> = wf
        .windows
        .iter()
        .skip(app.scroll)
        .take(split[1].height.saturating_sub(2) as usize)
        .map(|w| {
            let is_ret = w.in_sample.metrics.total_return_pct;
            let oos_ret = w.out_of_sample.metrics.total_return_pct;
            let mut parts: Vec<String> = w
                .optimized_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            parts.sort();
            let params_str = parts.join(" ");

            let oos_sharpe = w.out_of_sample.metrics.sharpe_ratio;
            let oos_dd = w.out_of_sample.metrics.max_drawdown_pct * 100.0;
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" W{:<2} ", w.window + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("IS:", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {:+.2}%", is_ret),
                    Style::default().fg(return_color(is_ret)),
                ),
                Span::styled("  OOS:", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {:+.2}%", oos_ret),
                    Style::default().fg(return_color(oos_ret)),
                ),
                Span::styled("  Sh:", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {}", format_ratio(oos_sharpe)),
                    Style::default().fg(if oos_sharpe >= 1.0 {
                        Color::Green
                    } else if oos_sharpe >= 0.0 {
                        Color::Yellow
                    } else {
                        Color::Red
                    }),
                ),
                Span::styled("  DD:", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {:.1}%", oos_dd),
                    Style::default().fg(if oos_dd <= 10.0 {
                        Color::Green
                    } else if oos_dd <= 20.0 {
                        Color::Yellow
                    } else {
                        Color::Red
                    }),
                ),
                Span::styled(
                    format!("  {}", params_str),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(visible).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(format!(" Windows ({}) - ↑/↓ scroll ", wf.windows.len())),
    );
    f.render_widget(list, split[1]);
}

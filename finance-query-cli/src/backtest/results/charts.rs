use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};

use super::ChartsView;
use super::ResultsApp;

// Charts tab: equity curve (top) + drawdown (bottom)

pub(super) fn render_charts(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let curve = &app.result.backtest.equity_curve;
    if curve.is_empty() {
        let msg = Paragraph::new("No equity curve data.")
            .block(Block::default().borders(Borders::ALL).title(" Charts "));
        f.render_widget(msg, area);
        return;
    }

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    render_equity_chart(f, app, split[0]);
    match app.charts_view {
        ChartsView::RollingSharpe => render_rolling_sharpe_chart(f, app, split[1]),
        ChartsView::RollingWinRate => render_rolling_win_rate_chart(f, app, split[1]),
        ChartsView::Drawdown => render_drawdown_chart(f, app, split[1]),
    }
}

fn render_equity_chart(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result.backtest;
    let curve = &r.equity_curve;

    let strategy_data: Vec<(f64, f64)> = curve
        .iter()
        .enumerate()
        .map(|(i, p)| (i as f64, p.equity))
        .collect();

    let n = curve.len() as f64;
    let min_equity = curve.iter().map(|p| p.equity).fold(f64::MAX, f64::min);
    let max_equity = curve.iter().map(|p| p.equity).fold(f64::MIN, f64::max);
    let y_margin = ((max_equity - min_equity) * 0.05).max(1.0);
    let y_min = (min_equity - y_margin).max(0.0);
    let y_max = max_equity + y_margin;

    let mut datasets = vec![
        Dataset::default()
            .name(format!("{} ({})", r.strategy_name, r.symbol))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&strategy_data),
    ];

    // Benchmark overlay: actual buy-and-hold equity curve from candle closes,
    // falling back to a linear approximation when candles aren't available.
    let bench_data: Vec<(f64, f64)>;
    if let Some(ref bench) = r.benchmark {
        bench_data = if let Some(ref candles) = app.result.bench_candles {
            let first_close = candles.first().map(|c| c.close).unwrap_or(1.0);
            candles
                .iter()
                .enumerate()
                .map(|(i, c)| (i as f64, r.initial_capital * (c.close / first_close)))
                .collect()
        } else {
            // Fallback: linear interpolation of total return (no intra-period detail)
            let end_equity = r.initial_capital * (1.0 + bench.benchmark_return_pct / 100.0);
            (0..curve.len())
                .map(|i| {
                    let frac = if n > 1.0 { i as f64 / (n - 1.0) } else { 1.0 };
                    (
                        i as f64,
                        r.initial_capital + (end_equity - r.initial_capital) * frac,
                    )
                })
                .collect()
        };
        let label = if app.result.bench_candles.is_some() {
            format!("{} B&H", bench.symbol)
        } else {
            format!("{} B&H (approx)", bench.symbol)
        };
        datasets.push(
            Dataset::default()
                .name(label)
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&bench_data),
        );
    }

    let x_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .bounds([0.0, (curve.len().saturating_sub(1)) as f64]);

    let y_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .labels(vec![
            Span::raw(format!("${:.0}", y_min)),
            Span::raw(format!("${:.0}", (y_min + y_max) / 2.0)),
            Span::raw(format!("${:.0}", y_max)),
        ])
        .bounds([y_min, y_max]);

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Equity Curve "),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    f.render_widget(chart, area);
}

fn render_drawdown_chart(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let curve = &app.result.backtest.equity_curve;

    // drawdown_pct stored as fraction (0-1); display as negative percentage
    let dd_data: Vec<(f64, f64)> = curve
        .iter()
        .enumerate()
        .map(|(i, p)| (i as f64, -(p.drawdown_pct * 100.0)))
        .collect();

    let max_dd = curve
        .iter()
        .map(|p| p.drawdown_pct * 100.0)
        .fold(0.0_f64, f64::max);

    let y_min = -(max_dd * 1.1).max(1.0);

    let x_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .bounds([0.0, (curve.len().saturating_sub(1)) as f64]);

    let y_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .labels(vec![
            Span::raw(format!("{:.1}%", y_min)),
            Span::raw(format!("{:.1}%", y_min / 2.0)),
            Span::raw("0.0%"),
        ])
        .bounds([y_min, 0.0]);

    let dataset = Dataset::default()
        .name("Drawdown")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Red))
        .data(&dd_data);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title(" Drawdown % "),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    f.render_widget(chart, area);
}

fn render_rolling_sharpe_chart(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result.backtest;
    const WINDOW: usize = 63;
    let rolling = r.rolling_sharpe(WINDOW);

    if rolling.is_empty() {
        let msg = Paragraph::new(format!(
            "Not enough data for rolling Sharpe (need >{} bars).",
            WINDOW
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Rolling Sharpe ({}-bar) ", WINDOW)),
        );
        f.render_widget(msg, area);
        return;
    }

    let offset = r.equity_curve.len().saturating_sub(rolling.len());
    let data: Vec<(f64, f64)> = rolling
        .iter()
        .enumerate()
        .map(|(i, &v)| ((i + offset) as f64, v))
        .collect();

    let y_min = rolling.iter().cloned().fold(f64::MAX, f64::min);
    let y_max = rolling.iter().cloned().fold(f64::MIN, f64::max);
    let margin = ((y_max - y_min) * 0.1).max(0.1);
    let y_lo = y_min - margin;
    let y_hi = y_max + margin;

    let x_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .bounds([0.0, (r.equity_curve.len().saturating_sub(1)) as f64]);

    let y_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .labels(vec![
            Span::raw(format!("{:.1}", y_lo)),
            Span::raw(format!("{:.1}", (y_lo + y_hi) / 2.0)),
            Span::raw(format!("{:.1}", y_hi)),
        ])
        .bounds([y_lo, y_hi]);

    let dataset = Dataset::default()
        .name(format!("Rolling Sharpe ({})", WINDOW))
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Magenta))
        .data(&data);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(format!(
                    " Rolling Sharpe ({}-bar) — s: cycle chart ",
                    WINDOW
                )),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    f.render_widget(chart, area);
}

fn render_rolling_win_rate_chart(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result.backtest;
    const WINDOW: usize = 20;
    let rolling = r.rolling_win_rate(WINDOW);

    if rolling.is_empty() {
        let msg = Paragraph::new(format!(
            "Not enough trades for rolling win rate (need >{} trades).",
            WINDOW
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Rolling Win Rate ({}-trade) ", WINDOW)),
        );
        f.render_widget(msg, area);
        return;
    }

    // rolling_win_rate is indexed by trade, not by bar — align to equity curve
    // length by offsetting to the right so the last value aligns with the end.
    let total_bars = r.equity_curve.len();
    let offset = total_bars.saturating_sub(rolling.len());
    let data: Vec<(f64, f64)> = rolling
        .iter()
        .enumerate()
        .map(|(i, &v)| ((i + offset) as f64, v * 100.0))
        .collect();

    let x_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .bounds([0.0, total_bars.saturating_sub(1) as f64]);

    let y_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .labels(vec![Span::raw("0%"), Span::raw("50%"), Span::raw("100%")])
        .bounds([0.0, 100.0]);

    let dataset = Dataset::default()
        .name(format!("Win Rate % ({}-trade)", WINDOW))
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Green))
        .data(&data);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(format!(
                    " Rolling Win Rate ({}-trade window) — s: cycle chart ",
                    WINDOW
                )),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    f.render_widget(chart, area);
}

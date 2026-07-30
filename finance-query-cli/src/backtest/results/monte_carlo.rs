use finance_query::backtesting::MonteCarloMethod;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::ResultsApp;
use super::format::format_ratio;

pub(super) fn next_mc_method(m: &MonteCarloMethod) -> MonteCarloMethod {
    match m {
        MonteCarloMethod::IidShuffle => MonteCarloMethod::BlockBootstrap { block_size: 10 },
        MonteCarloMethod::BlockBootstrap { .. } => MonteCarloMethod::StationaryBootstrap {
            mean_block_size: 10,
        },
        MonteCarloMethod::StationaryBootstrap { .. } => MonteCarloMethod::Parametric,
        _ => MonteCarloMethod::IidShuffle,
    }
}

fn mc_method_name(m: &MonteCarloMethod) -> &'static str {
    match m {
        MonteCarloMethod::IidShuffle => "IID Shuffle",
        MonteCarloMethod::BlockBootstrap { .. } => "Block Bootstrap (10)",
        MonteCarloMethod::StationaryBootstrap { .. } => "Stationary Bootstrap (10)",
        MonteCarloMethod::Parametric => "Parametric",
        _ => "Unknown",
    }
}

pub(super) fn render_monte_carlo(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let mc = &app.monte_carlo;

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let method_para = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" Method: {} ", mc_method_name(&app.mc_method)),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " (press v to cycle method)",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    f.render_widget(method_para, outer[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(outer[1]);

    let render_stat = |f: &mut Frame,
                       title: &str,
                       p5: f64,
                       p50: f64,
                       p95: f64,
                       is_pct: bool,
                       col: ratatui::layout::Rect| {
        let fmt = |v: f64| -> String {
            if is_pct {
                format_signed_pct(v * 100.0)
            } else {
                format_ratio(v)
            }
        };
        let p5_color = if p5 >= 0.0 { Color::Green } else { Color::Red };
        let p50_color = if p50 >= 0.0 { Color::Green } else { Color::Red };
        let p95_color = if p95 >= 0.0 { Color::Green } else { Color::Red };

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  p5  ", Style::default().fg(Color::DarkGray)),
                Span::styled(fmt(p5), Style::default().fg(p5_color)),
            ]),
            Line::from(vec![
                Span::styled("  p50 ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    fmt(p50),
                    Style::default().fg(p50_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  p95 ", Style::default().fg(Color::DarkGray)),
                Span::styled(fmt(p95), Style::default().fg(p95_color)),
            ]),
        ];

        let para = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(format!(" {} ", title)),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(para, col);
    };

    render_stat(
        f,
        "Total Return",
        mc.total_return.p5,
        mc.total_return.p50,
        mc.total_return.p95,
        true,
        cols[0],
    );
    render_stat(
        f,
        "Max Drawdown",
        -mc.max_drawdown.p95,
        -mc.max_drawdown.p50,
        -mc.max_drawdown.p5,
        true,
        cols[1],
    );
    render_stat(
        f,
        "Sharpe Ratio",
        mc.sharpe_ratio.p5,
        mc.sharpe_ratio.p50,
        mc.sharpe_ratio.p95,
        false,
        cols[2],
    );
    render_stat(
        f,
        "Profit Factor",
        mc.profit_factor.p5,
        mc.profit_factor.p50,
        mc.profit_factor.p95,
        false,
        cols[3],
    );
}

fn format_signed_pct(value: f64) -> String {
    if value.is_nan() {
        "-".to_string()
    } else if value == f64::MAX {
        "+∞%".to_string()
    } else if value.is_infinite() {
        if value.is_sign_negative() {
            "-∞%".to_string()
        } else {
            "+∞%".to_string()
        }
    } else {
        format!("{:+.2}%", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_signed_pct_handles_max_sentinel() {
        assert_eq!(format_signed_pct(f64::MAX), "+∞%");
    }
}

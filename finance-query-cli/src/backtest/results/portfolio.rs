use finance_query::backtesting::portfolio::PortfolioResult;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::ResultsApp;
use super::format::format_ratio;
use super::overview::render_results_overview;

// ── Portfolio tab ─────────────────────────────────────────────────────────────

pub(super) fn render_portfolio(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    use ratatui::widgets::{Cell, Row, Table};

    let Some(portfolio) = &app.result.portfolio else {
        render_results_overview(f, app, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    render_portfolio_metrics(f, portfolio, chunks[0]);
    render_portfolio_symbols_table(f, app, portfolio, chunks[1]);

    // Local helpers capture the outer widgets via `use` above.
    fn render_portfolio_metrics(
        f: &mut Frame,
        portfolio: &PortfolioResult,
        area: ratatui::layout::Rect,
    ) {
        let m = &portfolio.portfolio_metrics;
        let profit = portfolio.final_equity - portfolio.initial_capital;
        let profit_color = if profit >= 0.0 {
            Color::Green
        } else {
            Color::Red
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("Portfolio", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(
                    "  Initial: ${:.2}  Final: ${:.2}  ",
                    portfolio.initial_capital, portfolio.final_equity,
                )),
                Span::styled(
                    format!("{:+.2} ({:+.2}%)", profit, m.total_return_pct),
                    Style::default()
                        .fg(profit_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(format!(
                "  Annualized: {:+.2}%   Sharpe: {:.3}   Sortino: {:.3}   Calmar: {:.3}",
                m.annualized_return_pct, m.sharpe_ratio, m.sortino_ratio, m.calmar_ratio,
            )),
            Line::from(format!(
                "  Max DD: {:.2}%   Win Rate: {:.1}%   Profit Factor: {:.2}   Trades: {}",
                m.max_drawdown_pct * 100.0,
                m.win_rate * 100.0,
                m.profit_factor,
                m.total_trades,
            )),
            Line::from(format!(
                "  Expectancy: {:.3}   SQN: {:.3}   Kelly: {:.1}%   Time in Market: {:.1}%",
                m.expectancy,
                m.sqn,
                m.kelly_criterion * 100.0,
                m.time_in_market_pct * 100.0,
            )),
        ];

        let para = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Portfolio Summary"),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(para, area);
    }

    fn render_portfolio_symbols_table(
        f: &mut Frame,
        app: &ResultsApp,
        portfolio: &PortfolioResult,
        area: ratatui::layout::Rect,
    ) {
        let header = Row::new(vec![
            Cell::from("Symbol").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Return %").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Ann. %").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Sharpe").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Max DD %").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Win Rate").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Trades").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Profit Factor").style(Style::default().add_modifier(Modifier::BOLD)),
        ])
        .style(Style::default().fg(Color::Cyan))
        .height(1);

        // Sort symbols alphabetically for stable display order.
        let mut symbols: Vec<&String> = portfolio.symbols.keys().collect();
        symbols.sort();

        let row_count = symbols.len();
        let rows: Vec<Row> = symbols
            .iter()
            .enumerate()
            .map(|(i, sym)| {
                let r = &portfolio.symbols[*sym];
                let m = &r.metrics;
                let ret_color = if m.total_return_pct >= 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };
                let bg = if i % 2 == 0 {
                    Color::Reset
                } else {
                    Color::DarkGray
                };
                Row::new(vec![
                    Cell::from(sym.as_str()),
                    Cell::from(format!("{:+.2}%", m.total_return_pct))
                        .style(Style::default().fg(ret_color)),
                    Cell::from(format!("{:+.2}%", m.annualized_return_pct)),
                    Cell::from(format_ratio(m.sharpe_ratio)),
                    Cell::from(format!("{:.2}%", m.max_drawdown_pct * 100.0)),
                    Cell::from(format!("{:.1}%", m.win_rate * 100.0)),
                    Cell::from(format!("{}", m.total_trades)),
                    Cell::from(format_ratio(m.profit_factor)),
                ])
                .style(Style::default().bg(bg))
                .height(1)
            })
            .collect();

        let visible: Vec<Row> = rows
            .into_iter()
            .skip(app.scroll)
            .take(area.height.saturating_sub(3) as usize)
            .collect();

        let table = Table::new(
            visible,
            [
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(9),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(8),
                Constraint::Min(0),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Symbols ({}) — ↑↓ scroll", row_count)),
        );
        f.render_widget(table, area);
    }
}

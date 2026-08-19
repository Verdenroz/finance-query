use crate::dashboard::state::{App, InputMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub(super) fn render_portfolio(f: &mut Frame, app: &App, area: Rect) {
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

use ratatui::{
    Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use super::ResultsApp;
use super::format::{format_timestamp_with_precision, is_intraday, return_color};

pub(super) fn render_results_trades(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result.backtest;
    let intraday = is_intraday(&r.equity_curve);

    let visible_trades: Vec<ListItem> = r
        .trades
        .iter()
        .skip(app.scroll)
        .take(area.height.saturating_sub(2) as usize)
        .enumerate()
        .map(|(idx, trade)| {
            let pnl_color = return_color(trade.pnl);
            let side_str = if trade.is_long() { "LONG" } else { "SHORT" };
            let side_color = if trade.is_long() {
                Color::Green
            } else {
                Color::Red
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!(" #{:<3}", app.scroll + idx + 1),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(format!("{:<5}", side_str), Style::default().fg(side_color)),
                    Span::styled(" Entry: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("${:.2}", trade.entry_price),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(" @ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format_timestamp_with_precision(trade.entry_timestamp, intraday),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("      "),
                    Span::styled(" Exit:  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("${:.2}", trade.exit_price),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(" @ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format_timestamp_with_precision(trade.exit_timestamp, intraday),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled("P&L: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!(
                            "{}{:.2}",
                            if trade.pnl >= 0.0 { "+" } else { "" },
                            trade.pnl
                        ),
                        Style::default().fg(pnl_color),
                    ),
                ]),
            ])
        })
        .collect();

    let list = List::new(visible_trades).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(format!(
                " Trades ({}/{}) - ↑/↓ to scroll ",
                app.scroll + 1,
                r.trades.len()
            )),
    );
    f.render_widget(list, area);
}

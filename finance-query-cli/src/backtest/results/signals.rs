use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use super::ResultsApp;
use super::format::{format_timestamp_with_precision, is_intraday};

pub(super) fn render_results_signals(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result.backtest;
    let intraday = is_intraday(&r.equity_curve);

    let visible_signals: Vec<ListItem> = r
        .signals
        .iter()
        .skip(app.scroll)
        .take(area.height.saturating_sub(2) as usize)
        .enumerate()
        .map(|(idx, signal)| {
            let signal_str = signal.direction.to_string();
            let signal_color = match signal_str.as_str() {
                "LONG" => Color::Green,
                "SHORT" | "EXIT" => Color::Red,
                _ => Color::Yellow,
            };
            let executed_marker = if signal.executed { "✓" } else { "○" };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" #{:<4}", app.scroll + idx + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<6}", signal_str),
                    Style::default()
                        .fg(signal_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {} ", executed_marker),
                    Style::default().fg(if signal.executed {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::styled(" @ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("${:.2}", signal.price),
                    Style::default().fg(Color::White),
                ),
                Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format_timestamp_with_precision(signal.timestamp, intraday),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(visible_signals).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(format!(
                " Signals ({}/{}) - ↑/↓ to scroll ",
                app.scroll + 1,
                r.signals.len()
            )),
    );
    f.render_widget(list, area);
}

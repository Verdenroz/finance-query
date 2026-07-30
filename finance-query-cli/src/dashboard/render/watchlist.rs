use crate::dashboard::state::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
};

pub(super) fn render_watchlist(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        "Watchlist ({}) [a:add d:delete]",
        app.current_watchlist.name
    );
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.current_watchlist.symbols.is_empty() {
        let empty =
            Paragraph::new("Press 'a' to add symbols").style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty, inner);
        return;
    }

    let visible_rows = inner.height as usize;

    for (i, symbol) in app.current_watchlist.symbols.iter().enumerate() {
        if i >= visible_rows {
            break;
        }

        let row_area = Rect {
            x: inner.x,
            y: inner.y + i as u16,
            width: inner.width,
            height: 1,
        };

        // Split row: text on left (~28 chars), sparkline on right
        let text_width = 28u16.min(row_area.width.saturating_sub(8));
        let sparkline_width = row_area.width.saturating_sub(text_width);

        let text_area = Rect {
            x: row_area.x,
            y: row_area.y,
            width: text_width,
            height: 1,
        };
        let spark_area = Rect {
            x: row_area.x + text_width,
            y: row_area.y,
            width: sparkline_width,
            height: 1,
        };

        // Build price info spans
        let (price_info, _change_pct) = if let Some(update) = app.price_updates.get(symbol) {
            let change_color = if update.change >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            let arrow = if update.change >= 0.0 { "↑" } else { "↓" };
            (
                vec![
                    Span::styled(
                        format!("{:<6}", symbol),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" ${:>8.2} ", update.price)),
                    Span::styled(
                        format!("{:>+6.2}%{}", update.change_percent, arrow),
                        Style::default().fg(change_color),
                    ),
                ],
                Some(update.change_percent as f64),
            )
        } else if let Some(quote) = app.quotes.get(symbol) {
            let price = quote
                .regular_market_price
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0);
            let change_pct = quote
                .regular_market_change_percent
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0);

            let change_color = if change_pct >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            let arrow = if change_pct >= 0.0 { "↑" } else { "↓" };

            (
                vec![
                    Span::styled(
                        format!("{:<6}", symbol),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" ${:>8.2} ", price)),
                    Span::styled(
                        format!("{:>+6.2}%{}", change_pct, arrow),
                        Style::default().fg(change_color),
                    ),
                ],
                Some(change_pct),
            )
        } else {
            (
                vec![
                    Span::styled(format!("{:<6}", symbol), Style::default().fg(Color::Cyan)),
                    Span::styled(" Loading...", Style::default().fg(Color::DarkGray)),
                ],
                None,
            )
        };

        // Apply selection highlight
        let row_style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        // Render text
        let text = Paragraph::new(Line::from(price_info)).style(row_style);
        f.render_widget(text, text_area);

        // Render sparkline with per-bar colors (green above open, red below)
        if sparkline_width >= 4
            && let Some(spark) = app.sparklines.get(symbol)
            && !spark.closes.is_empty()
        {
            use ratatui::widgets::SparklineBar;

            let min = spark.min_close().unwrap_or(0.0);
            let max = spark.max_close().unwrap_or(100.0);
            let range = (max - min).max(0.01);

            // Reference price is the first price (open)
            let open_price = spark.closes[0];

            // Create bars with individual colors based on price vs open
            let bars: Vec<SparklineBar> = spark
                .closes
                .iter()
                .map(|&price| {
                    let value = ((price - min) / range * 100.0) as u64;
                    let color = if price >= open_price {
                        Color::Green
                    } else {
                        Color::Red
                    };
                    SparklineBar::from(value).style(Style::default().fg(color))
                })
                .collect();

            let sparkline = Sparkline::default().data(bars);

            f.render_widget(sparkline, spark_area);
        }
    }
}

use crate::dashboard::state::{App, FocusPane};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Block, Borders, Chart, Dataset, GraphType, Paragraph},
};

pub(super) fn render_chart(f: &mut Frame, app: &App, area: Rect) {
    let Some(symbol) = app.selected_symbol() else {
        let paragraph = Paragraph::new("No symbol selected")
            .block(Block::default().borders(Borders::ALL).title("Chart"));
        f.render_widget(paragraph, area);
        return;
    };

    let is_focused = app.focus_pane == FocusPane::Right;
    let range_options = ["1D", "5D", "1M", "6M", "YTD", "1Y", "5Y", "Max"];
    let current_range = range_options[app.selected_chart_range_idx];

    if is_focused {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let range_buttons: Vec<Span> = range_options
            .iter()
            .enumerate()
            .flat_map(|(idx, label)| {
                let is_selected = idx == app.selected_chart_range_idx;
                vec![
                    if is_selected {
                        Span::styled(
                            format!(" {} ", label),
                            Style::default()
                                .bg(Color::Blue)
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::styled(format!(" {} ", label), Style::default().fg(Color::DarkGray))
                    },
                    Span::raw(" "),
                ]
            })
            .collect();

        let range_selector = Paragraph::new(Line::from(range_buttons)).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Range [←/→:change]")
                .border_style(Style::default().fg(Color::Yellow)),
        );
        f.render_widget(range_selector, chunks[0]);

        render_chart_widget(f, app, chunks[1], symbol, current_range);
    } else {
        render_chart_widget(f, app, area, symbol, current_range);
    }
}

fn render_chart_widget(f: &mut Frame, app: &App, area: Rect, symbol: &str, range_label: &str) {
    let Some(ref chart_data) = app.chart_data else {
        let paragraph = Paragraph::new("Loading chart data...").block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} - {} Chart", symbol, range_label)),
        );
        f.render_widget(paragraph, area);
        return;
    };

    if chart_data.is_empty() {
        let paragraph = Paragraph::new("No chart data available").block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} - {} Chart", symbol, range_label)),
        );
        f.render_widget(paragraph, area);
        return;
    }

    let prices: Vec<f64> = chart_data.iter().map(|(_, p)| *p).collect();
    let min_price = prices.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_price = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let padding = (max_price - min_price) * 0.05;
    let y_min = (min_price - padding).max(0.0); // Prices can't be negative
    let y_max = max_price + padding;

    let first_price = prices.first().copied().unwrap_or(0.0);
    let last_price = prices.last().copied().unwrap_or(0.0);
    let line_color = if last_price >= first_price {
        Color::Green
    } else {
        Color::Red
    };

    let dataset = vec![
        Dataset::default()
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(line_color))
            .data(chart_data),
    ];

    let x_min = chart_data.first().map(|(t, _)| *t).unwrap_or(0.0);
    let x_max = chart_data.last().map(|(t, _)| *t).unwrap_or(1.0);

    let y_labels = vec![
        Span::raw(format!("${:.2}", y_min)),
        Span::raw(format!("${:.2}", (y_min + y_max) / 2.0)),
        Span::raw(format!("${:.2}", y_max)),
    ];

    let is_focused = app.focus_pane == FocusPane::Right;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let mut title = format!("{} - {} Chart", symbol, range_label);
    if !is_focused {
        title.push_str(" [l/→:focus]");
    }

    let chart_widget = Chart::new(dataset)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .x_axis(ratatui::widgets::Axis::default().bounds([x_min, x_max]))
        .y_axis(
            ratatui::widgets::Axis::default()
                .bounds([y_min, y_max])
                .labels(y_labels),
        );

    f.render_widget(chart_widget, area);
}

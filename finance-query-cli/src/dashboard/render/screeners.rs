use super::util::{format_market_cap, format_volume};
use crate::dashboard::state::{App, FocusPane, ScreenerCategory};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub(super) fn render_screeners(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let is_focused = app.focus_pane == FocusPane::Right;

    let categories = ScreenerCategory::ALL;
    let category_buttons: Vec<Span> = categories
        .iter()
        .flat_map(|cat| {
            let is_selected = *cat == app.screener_category;
            vec![
                if is_selected {
                    Span::styled(
                        format!(" {} ", cat.title()),
                        Style::default()
                            .bg(Color::Blue)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(
                        format!(" {} ", cat.title()),
                        Style::default().fg(Color::DarkGray),
                    )
                },
                Span::raw(" "),
            ]
        })
        .collect();

    let selector_border = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let category_title = format!("{} [←/→:change]", app.screener_category.description());
    let category_selector = Paragraph::new(Line::from(category_buttons)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(category_title)
            .border_style(selector_border),
    );
    f.render_widget(category_selector, chunks[0]);

    if app.is_loading_screeners {
        let loading = Paragraph::new("Loading...")
            .block(Block::default().borders(Borders::ALL).title("Screeners"));
        f.render_widget(loading, chunks[1]);
        return;
    }

    if app.screener_data.is_empty() {
        let empty = Paragraph::new("No data available. Press 'r' to refresh.")
            .block(Block::default().borders(Borders::ALL).title("Screeners"))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty, chunks[1]);
        return;
    }

    let items: Vec<ListItem> = app
        .screener_data
        .iter()
        .enumerate()
        .map(|(idx, quote)| {
            let is_selected = is_focused && idx == app.selected_screener_idx;

            let price = quote.regular_market_price.raw.unwrap_or(0.0);
            let change = quote.regular_market_change.raw.unwrap_or(0.0);
            let change_pct = quote.regular_market_change_percent.raw.unwrap_or(0.0);
            let volume = quote
                .regular_market_volume
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0);
            let market_cap = quote.market_cap.as_ref().and_then(|v| v.raw);

            let change_color = if change >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            let arrow = if change >= 0.0 { "▲" } else { "▼" };

            let symbol_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            };

            let name_style = if is_selected {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            let price_style = if is_selected {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            let change_style = if is_selected {
                Style::default().fg(change_color).bg(Color::DarkGray)
            } else {
                Style::default().fg(change_color)
            };

            let meta_style = if is_selected {
                Style::default().fg(Color::DarkGray).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let name = quote
                .display_name
                .clone()
                .or_else(|| Some(quote.short_name.clone()))
                .unwrap_or_default();
            let name_display = if name.len() > 25 {
                format!("{}...", &name[..22])
            } else {
                name
            };

            let mcap_str = market_cap.map(format_market_cap).unwrap_or_default();

            let content = vec![
                Line::from(vec![
                    Span::styled(format!("{:<6}", quote.symbol), symbol_style),
                    Span::styled(format!(" {:<25}", name_display), name_style),
                    Span::styled(format!(" ${:>8.2}", price), price_style),
                    Span::styled(format!("  {:>+6.2}% {}", change_pct, arrow), change_style),
                ]),
                Line::from(vec![Span::styled(
                    format!("      Vol: {}  MCap: {}", format_volume(volume), mcap_str),
                    meta_style,
                )]),
                Line::from(""),
            ];

            ListItem::new(content)
        })
        .collect();

    let title = format!(
        "{} ({}/{}) [Enter:add j/k:nav]",
        app.screener_category.title(),
        if app.screener_data.is_empty() {
            0
        } else {
            app.selected_screener_idx + 1
        },
        app.screener_data.len()
    );

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    let mut list_state = ListState::default();
    if is_focused && !app.screener_data.is_empty() {
        list_state.select(Some(app.selected_screener_idx));
    }

    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

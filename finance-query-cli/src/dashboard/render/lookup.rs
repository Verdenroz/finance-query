use crate::dashboard::state::{App, FocusPane};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub(super) fn render_lookup(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let input_text = if app.is_searching {
        format!("Searching for '{}'...", app.search_query)
    } else {
        app.search_query.clone()
    };

    let input = Paragraph::new(input_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Search Symbol (type to search, Enter to add)")
            .border_style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(input, chunks[0]);

    let is_focused = app.focus_pane == FocusPane::Right;

    if app.is_searching {
        let loading = Paragraph::new("Searching...")
            .block(Block::default().borders(Borders::ALL).title("Results"));
        f.render_widget(loading, chunks[1]);
    } else if app.search_results.is_empty() {
        let empty_msg = if app.search_query.is_empty() {
            "Type a symbol or company name to search"
        } else {
            "No results found"
        };
        let paragraph = Paragraph::new(empty_msg)
            .block(Block::default().borders(Borders::ALL).title("Results"))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .search_results
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                let is_selected = is_focused && idx == app.selected_search_idx;

                let symbol_style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Cyan)
                };

                let name_style = if is_selected {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };

                let meta_style = if is_selected {
                    Style::default().fg(Color::DarkGray).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let name = result
                    .short_name
                    .clone()
                    .or_else(|| result.long_name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                let type_exch = format!(
                    "{} | {}",
                    result.quote_type.as_deref().unwrap_or(""),
                    result.exch_disp.as_deref().unwrap_or("")
                );

                let content = vec![
                    Line::from(vec![
                        Span::styled(&result.symbol, symbol_style),
                        Span::raw("  "),
                        Span::styled(name, name_style),
                    ]),
                    Line::from(vec![Span::styled(type_exch, meta_style)]),
                    Line::from(""),
                ];

                ListItem::new(content)
            })
            .collect();

        let title = format!("Results ({}) [Enter:add to watchlist]", items.len());
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
        if is_focused && !app.search_results.is_empty() {
            list_state.select(Some(app.selected_search_idx));
        }

        f.render_stateful_widget(list, chunks[1], &mut list_state);
    }
}

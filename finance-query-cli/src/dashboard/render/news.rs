use crate::dashboard::state::{App, FocusPane};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub(super) fn render_news(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus_pane == FocusPane::Right;

    // Show loading state
    if app.is_loading_news {
        let paragraph = Paragraph::new("Loading news...")
            .block(Block::default().borders(Borders::ALL).title("News"));
        f.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .news_items
        .iter()
        .enumerate()
        .map(|(idx, article)| {
            let is_selected = is_focused && idx == app.selected_news_idx;

            let title_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let meta_style = if is_selected {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let content = vec![
                Line::from(vec![Span::styled(&article.title, title_style)]),
                Line::from(vec![
                    Span::styled(&article.source, meta_style),
                    Span::styled(" • ", meta_style),
                    Span::styled(&article.time, meta_style),
                ]),
                Line::from(""),
            ];

            ListItem::new(content)
        })
        .collect();

    // Title based on whether showing general or symbol news
    let news_title = match &app.news_symbol {
        Some(symbol) => format!("{} News", symbol),
        None => "Market News".to_string(),
    };

    let mut title = if items.is_empty() {
        format!("{} (no articles)", news_title)
    } else {
        format!(
            "{} ({}/{})",
            news_title,
            app.selected_news_idx + 1,
            items.len()
        )
    };

    // Show appropriate hints based on focus and news type
    if is_focused {
        if app.news_symbol.is_some() {
            title.push_str(" [j/k:scroll Enter:open g:market h:back]");
        } else {
            title.push_str(" [j/k:scroll Enter:open h:back]");
        }
    } else {
        title.push_str(" [Enter:symbol news l/→:focus]");
    }

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
    if is_focused && !app.news_items.is_empty() {
        list_state.select(Some(app.selected_news_idx));
    }

    f.render_stateful_widget(list, area, &mut list_state);
}

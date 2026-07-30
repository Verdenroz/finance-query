mod alerts;
mod chart;
mod details;
mod lookup;
mod news;
mod portfolio;
mod screeners;
mod sectors;
mod status;
mod util;
mod watchlist;

use super::state::{App, Tab};
use alerts::render_alerts_tab;
use details::render_details;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use sectors::render_sectors_tab;
use status::render_status;
use watchlist::render_watchlist;

pub fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    render_tabs(f, app, chunks[0]);

    // Alerts and Sectors tabs use full width, others use left/right split
    if app.active_tab == Tab::Alerts {
        render_alerts_tab(f, app, chunks[1]);
    } else if app.active_tab == Tab::Sectors {
        render_sectors_tab(f, app, chunks[1]);
    } else {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(chunks[1]);

        render_watchlist(f, app, content_chunks[0]);
        render_details(f, app, content_chunks[1]);
    }

    render_status(f, app, chunks[2]);
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tabs = [
        Tab::Watchlist,
        Tab::Charts,
        Tab::News,
        Tab::Lookup,
        Tab::Screeners,
        Tab::Sectors,
        Tab::Portfolio,
        Tab::Alerts,
    ];

    let tab_titles: Vec<Span> = tabs
        .iter()
        .map(|tab| {
            let style = if *tab == app.active_tab {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Span::styled(format!(" {} ", tab.title()), style)
        })
        .collect();

    let tabs_line = Line::from(tab_titles);
    let tabs_widget = Paragraph::new(tabs_line).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Tabs (Tab/Shift+Tab)"),
    );

    f.render_widget(tabs_widget, area);
}

use super::super::state::App;
use super::super::types::ComparisonType;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub(super) fn render_comparison_config(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let ind_name = app
        .building_indicator
        .as_ref()
        .map(|i| i.display_name())
        .unwrap_or_else(|| "Unknown".to_string());

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Select Comparison ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" for ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &ind_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    // Comparison options as a scrollable list
    let items: Vec<ListItem> = ComparisonType::all()
        .iter()
        .enumerate()
        .map(|(idx, comp)| {
            let selected = idx == app.comparison_idx;
            let prefix = if selected { "▶ " } else { "  " };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        prefix,
                        if selected {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        },
                    ),
                    Span::styled(
                        format!("{} ", comp.symbol()),
                        if selected {
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Cyan)
                        },
                    ),
                    Span::styled(
                        comp.name(),
                        if selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                ]),
                Line::from(vec![Span::styled(
                    format!("     {}", comp.description()),
                    Style::default().fg(Color::DarkGray),
                )]),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Comparison Type "),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut list_state = ListState::default();
    list_state.select(Some(app.comparison_idx));
    f.render_stateful_widget(list, chunks[1], &mut list_state);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑/↓", Style::default().fg(Color::White)),
        Span::styled(":navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::White)),
        Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::White)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[2]);
}

pub(super) fn render_target_config(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(area);

    let comp_name = app
        .building_comparison
        .map(|c| c.name())
        .unwrap_or("Unknown");

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Set Target Value ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                comp_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    // Target value configuration - check if we need two values
    let needs_two_values = app
        .building_comparison
        .map(|c| c.needs_range())
        .unwrap_or(false);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " Current condition: ",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(""),
    ];

    if let Some(ref ind) = app.building_indicator {
        let preview = if needs_two_values {
            format!(
                "   {:.2} < {} < {:.2}",
                app.target_value,
                ind.display_name(),
                app.target_value2
            )
        } else {
            format!(
                "   {} {} {:.2}",
                ind.display_name(),
                app.building_comparison.map(|c| c.symbol()).unwrap_or("?"),
                app.target_value
            )
        };

        lines.push(Line::from(vec![Span::styled(
            preview,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Primary value display
    let primary_selected = app.editing_target_value;
    let primary_editing = app.target_input_mode && app.editing_target_value;

    let primary_value_display = if primary_editing {
        format!("{}▌", app.target_edit_buffer)
    } else {
        format!("{:.2}", app.target_value)
    };

    lines.push(Line::from(vec![
        Span::styled(
            if primary_selected { " ▶ " } else { "   " },
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(
            "Primary value: ",
            if primary_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::styled(
            primary_value_display,
            if primary_editing {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if primary_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ),
    ]));

    if needs_two_values {
        lines.push(Line::from(""));

        let secondary_selected = !app.editing_target_value;
        let secondary_editing = app.target_input_mode && !app.editing_target_value;

        let secondary_value_display = if secondary_editing {
            format!("{}▌", app.target_edit_buffer)
        } else {
            format!("{:.2}", app.target_value2)
        };

        lines.push(Line::from(vec![
            Span::styled(
                if secondary_selected { " ▶ " } else { "   " },
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                "Secondary value: ",
                if secondary_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Span::styled(
                secondary_value_display,
                if secondary_editing {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if secondary_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            ),
        ]));
    }

    // Show typical range if available
    if let Some(ref ind) = app.building_indicator
        && let Some((low, high)) = ind.indicator.typical_range
    {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!(" Typical range: {:.0} to {:.0}", low, high),
            Style::default().fg(Color::DarkGray),
        )]));
    }

    let content = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Target "),
    );
    f.render_widget(content, chunks[1]);

    // Footer - different based on input mode
    let footer = if app.target_input_mode {
        Paragraph::new(vec![Line::from(vec![
            Span::styled(" Type number", Style::default().fg(Color::White)),
            Span::styled(" then ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::styled(":apply  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::White)),
            Span::styled(":cancel", Style::default().fg(Color::DarkGray)),
        ])])
    } else {
        let mut spans = vec![
            Span::styled(" Type number", Style::default().fg(Color::Yellow)),
            Span::styled(" or ", Style::default().fg(Color::DarkGray)),
            Span::styled("←/→", Style::default().fg(Color::White)),
            Span::styled(":±1  ", Style::default().fg(Color::DarkGray)),
            Span::styled("↑/↓", Style::default().fg(Color::White)),
            Span::styled(":±0.1  ", Style::default().fg(Color::DarkGray)),
        ];
        if needs_two_values {
            spans.push(Span::styled("Tab", Style::default().fg(Color::White)));
            spans.push(Span::styled(
                ":switch  ",
                Style::default().fg(Color::DarkGray),
            ));
        }
        spans.push(Span::styled("Enter", Style::default().fg(Color::Green)));
        spans.push(Span::styled(
            ":confirm",
            Style::default().fg(Color::DarkGray),
        ));
        Paragraph::new(vec![Line::from(spans)])
    };
    f.render_widget(footer, chunks[2]);
}

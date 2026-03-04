use super::super::indicators::IndicatorCategory;
use super::super::state::{App, ConditionPanel};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
};

pub(super) fn render_strategy_builder(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Strategy Builder ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " - Navigate with ←/→ and ↑/↓, toggle AND/OR with Enter/c, delete with d",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    // Main content - split into entry/exit sections
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Determine which panel is active
    let entry_active = app.active_condition_panel == ConditionPanel::Entry;
    let exit_active = app.active_condition_panel == ConditionPanel::Exit;

    // Entry conditions panel
    let entry_conds = &app.config.strategy.entry_conditions;
    let entry_items: Vec<ListItem> = if entry_conds.conditions.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "(no conditions - press 1 to add)",
            Style::default().fg(Color::DarkGray),
        )]))]
    } else {
        entry_conds
            .conditions
            .iter()
            .enumerate()
            .flat_map(|(i, cond)| {
                let mut items = Vec::new();

                // Add the condition line
                let is_selected = entry_active && i == app.entry_condition_idx;
                let prefix = if is_selected { "▶ " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(cond.display(), style),
                ])));

                // Add the operator line after this condition (if not the last)
                if i < entry_conds.conditions.len() - 1 {
                    let op_style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("     "),
                        Span::styled(format!("[{}]", cond.next_op.name()), op_style),
                    ])));
                }

                items
            })
            .collect()
    };

    let entry_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if entry_active {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .title(if entry_active {
            " ▶ Entry (Long) - [1] add "
        } else {
            "   Entry (Long) - [1] add "
        });

    let entry_list = List::new(entry_items)
        .block(entry_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(entry_list, main_chunks[0]);

    // Exit conditions panel
    let exit_conds = &app.config.strategy.exit_conditions;
    let exit_items: Vec<ListItem> = if exit_conds.conditions.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "(no conditions - press 2 to add)",
            Style::default().fg(Color::DarkGray),
        )]))]
    } else {
        exit_conds
            .conditions
            .iter()
            .enumerate()
            .flat_map(|(i, cond)| {
                let mut items = Vec::new();

                // Add the condition line
                let is_selected = exit_active && i == app.exit_condition_idx;
                let prefix = if is_selected { "▶ " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(cond.display(), style),
                ])));

                // Add the operator line after this condition (if not the last)
                if i < exit_conds.conditions.len() - 1 {
                    let op_style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("     "),
                        Span::styled(format!("[{}]", cond.next_op.name()), op_style),
                    ])));
                }

                items
            })
            .collect()
    };

    let exit_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if exit_active {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .title(if exit_active {
            " ▶ Exit - [2] add "
        } else {
            "   Exit - [2] add "
        });

    let exit_list = List::new(exit_items)
        .block(exit_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(exit_list, main_chunks[1]);

    // Footer with updated keybindings
    let can_run = app.can_run();
    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" ←/→/Tab", Style::default().fg(Color::White)),
            Span::styled(":switch panel  ", Style::default().fg(Color::DarkGray)),
            Span::styled("↑/↓", Style::default().fg(Color::White)),
            Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter", Style::default().fg(Color::White)),
            Span::styled(":add condition  ", Style::default().fg(Color::DarkGray)),
            Span::styled("c", Style::default().fg(Color::White)),
            Span::styled(":toggle AND/OR  ", Style::default().fg(Color::DarkGray)),
            Span::styled("d", Style::default().fg(Color::White)),
            Span::styled(":delete  ", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" 1", Style::default().fg(Color::Green)),
            Span::styled(":add entry  ", Style::default().fg(Color::DarkGray)),
            Span::styled("2", Style::default().fg(Color::Yellow)),
            Span::styled(":add exit  ", Style::default().fg(Color::DarkGray)),
            Span::styled("b", Style::default().fg(Color::White)),
            Span::styled(":back  ", Style::default().fg(Color::DarkGray)),
            if can_run {
                Span::styled("r", Style::default().fg(Color::Green))
            } else {
                Span::styled("r", Style::default().fg(Color::DarkGray))
            },
            Span::styled(":run", Style::default().fg(Color::DarkGray)),
        ]),
    ]);
    f.render_widget(footer, chunks[2]);
}

pub(super) fn render_indicator_browser(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Select Indicator ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" for ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.condition_target.name(),
                Style::default()
                    .fg(app.condition_target.color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" condition", Style::default().fg(Color::DarkGray)),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    // Category tabs with indicator counts
    let categories = IndicatorCategory::all();
    let tab_titles: Vec<Line> = categories
        .iter()
        .map(|cat| {
            let count = app.indicator_count_by_category(*cat);
            Line::from(Span::styled(
                format!("{} ({})", cat.name(), count),
                Style::default().fg(cat.color()),
            ))
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(app.category_idx)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );
    f.render_widget(tabs, chunks[1]);

    // Indicator list for current category
    let indicators = app.indicators_in_category();
    let items: Vec<ListItem> = indicators
        .iter()
        .enumerate()
        .map(|(idx, ind)| {
            let selected = idx == app.indicator_idx;

            let params_str = if ind.params.is_empty() {
                String::new()
            } else {
                let ps: Vec<String> = ind.params.iter().map(|p| p.name.to_string()).collect();
                format!(" ({})", ps.join(", "))
            };

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
                        format!("{} ", ind.name),
                        if selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                    Span::styled(params_str, Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![Span::styled(
                    format!("     {}", ind.description),
                    Style::default().fg(Color::DarkGray),
                )]),
            ])
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.current_category().color()))
            .title(format!(" {} ", app.current_category().name())),
    );

    let mut list_state = ListState::default();
    list_state.select(Some(app.indicator_idx));
    f.render_stateful_widget(list, chunks[2], &mut list_state);

    // Info panel
    let info_lines = if let Some(ind) = app.current_indicator() {
        let mut lines = vec![Line::from(vec![
            Span::styled(" Code: ", Style::default().fg(Color::DarkGray)),
            Span::styled(ind.code, Style::default().fg(Color::Cyan)),
        ])];

        if let Some((low, high)) = ind.typical_range {
            lines.push(Line::from(vec![
                Span::styled(" Range: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.0} to {:.0}", low, high),
                    Style::default().fg(Color::White),
                ),
            ]));
        }

        if !ind.params.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(" Parameters: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}", ind.params.len()),
                    Style::default().fg(Color::White),
                ),
            ]));
        }

        lines
    } else {
        vec![Line::from("")]
    };

    let info = Paragraph::new(info_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Info "),
    );
    f.render_widget(info, chunks[3]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ←/→", Style::default().fg(Color::White)),
        Span::styled(":category  ", Style::default().fg(Color::DarkGray)),
        Span::styled("↑/↓", Style::default().fg(Color::White)),
        Span::styled(":navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::White)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[4]);
}

pub(super) fn render_indicator_config(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let ind = app.building_indicator.as_ref();
    let ind_name = ind.map(|i| i.indicator.name).unwrap_or("Unknown");

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Configure Parameters ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                ind_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    // Parameters
    let param_lines: Vec<Line> = if let Some(ind) = ind {
        if ind.indicator.params.is_empty() {
            vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    " This indicator has no configurable parameters.",
                    Style::default().fg(Color::DarkGray),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    " Press Enter to continue.",
                    Style::default().fg(Color::White),
                )]),
            ]
        } else {
            let mut lines = vec![Line::from("")];
            for (idx, param) in ind.indicator.params.iter().enumerate() {
                let selected = idx == app.param_idx;
                let value = app.param_values.get(idx).copied().unwrap_or(param.default);

                let style = if selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {:<15}", param.name),
                        if selected {
                            style.fg(Color::Yellow)
                        } else {
                            style.fg(Color::White)
                        },
                    ),
                    Span::styled(
                        format!(
                            " {} ",
                            if value.fract() == 0.0 {
                                format!("{:.0}", value)
                            } else {
                                format!("{:.2}", value)
                            }
                        ),
                        style.fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("  ({:.0}-{:.0})", param.min, param.max),
                        style.fg(Color::DarkGray),
                    ),
                ]));

                if selected {
                    lines.push(Line::from(vec![Span::styled(
                        format!("                 {}", param.description),
                        style.fg(Color::DarkGray),
                    )]));
                }
            }
            lines
        }
    } else {
        vec![Line::from("")]
    };

    let params = Paragraph::new(param_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Parameters "),
    );
    f.render_widget(params, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" j/k", Style::default().fg(Color::White)),
        Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("←/→", Style::default().fg(Color::White)),
        Span::styled(":adjust value  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::White)),
        Span::styled(":next  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::White)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[2]);
}

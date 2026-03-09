use super::super::indicators::IndicatorCategory;
use super::super::state::{App, ConditionPanel};
use super::super::types::ConditionGroup;
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
            Constraint::Length(9),
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
                " - Navigate with ←/→ and ↑/↓, toggle AND/OR with c, delete with d",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    // Main content - split into entry/exit/regime sections (top row)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[1]);

    // Determine which panel is active
    let entry_active = app.active_condition_panel == ConditionPanel::Entry;
    let exit_active = app.active_condition_panel == ConditionPanel::Exit;
    let regime_active = app.active_condition_panel == ConditionPanel::Regime;

    f.render_widget(
        build_condition_panel(
            &app.config.strategy.entry_conditions,
            app.entry_condition_idx,
            entry_active,
            Color::Green,
            Color::Yellow,
            "Entry (Long)",
            '1',
            "(no conditions - press 1 to add)",
        ),
        main_chunks[0],
    );

    f.render_widget(
        build_condition_panel(
            &app.config.strategy.exit_conditions,
            app.exit_condition_idx,
            exit_active,
            Color::Yellow,
            Color::Yellow,
            "Exit",
            '2',
            "(no conditions - press 2 to add)",
        ),
        main_chunks[1],
    );

    f.render_widget(
        build_condition_panel(
            &app.config.strategy.regime_conditions,
            app.regime_condition_idx,
            regime_active,
            Color::Cyan,
            Color::Cyan,
            "Regime Filter",
            '5',
            "(no filter - press 5 to add)",
        ),
        main_chunks[2],
    );

    // Scale-in / Scale-out panels (bottom row)
    let scale_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    let scale_in_active = app.active_condition_panel == ConditionPanel::ScaleIn;
    let scale_out_active = app.active_condition_panel == ConditionPanel::ScaleOut;

    f.render_widget(
        build_scale_panel(
            &app.config.strategy.scale_in_conditions,
            app.scale_in_condition_idx,
            scale_in_active,
            app.config.strategy.scale_in_fraction,
            Color::LightBlue,
            "Scale-In",
            '6',
        ),
        scale_chunks[0],
    );

    f.render_widget(
        build_scale_panel(
            &app.config.strategy.scale_out_conditions,
            app.scale_out_condition_idx,
            scale_out_active,
            app.config.strategy.scale_out_fraction,
            Color::LightRed,
            "Scale-Out",
            '7',
        ),
        scale_chunks[1],
    );

    // Footer
    let can_run = app.can_run();
    let fraction_hint = if app.editing
        && matches!(
            app.active_condition_panel,
            ConditionPanel::ScaleIn | ConditionPanel::ScaleOut
        ) {
        format!(" Fraction: {}%█", app.edit_buffer)
    } else {
        String::new()
    };
    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" ←/→/Tab", Style::default().fg(Color::White)),
            Span::styled(":panel  ", Style::default().fg(Color::DarkGray)),
            Span::styled("↑/↓", Style::default().fg(Color::White)),
            Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
            Span::styled("c", Style::default().fg(Color::White)),
            Span::styled(":AND/OR  ", Style::default().fg(Color::DarkGray)),
            Span::styled("d", Style::default().fg(Color::White)),
            Span::styled(":delete  ", Style::default().fg(Color::DarkGray)),
            Span::styled("f", Style::default().fg(Color::LightBlue)),
            Span::styled(":fraction  ", Style::default().fg(Color::DarkGray)),
            Span::styled(fraction_hint, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled(" 1", Style::default().fg(Color::Green)),
            Span::styled(":entry  ", Style::default().fg(Color::DarkGray)),
            Span::styled("2", Style::default().fg(Color::Yellow)),
            Span::styled(":exit  ", Style::default().fg(Color::DarkGray)),
            Span::styled("5", Style::default().fg(Color::Cyan)),
            Span::styled(":regime  ", Style::default().fg(Color::DarkGray)),
            Span::styled("6", Style::default().fg(Color::LightBlue)),
            Span::styled(":scale-in  ", Style::default().fg(Color::DarkGray)),
            Span::styled("7", Style::default().fg(Color::LightRed)),
            Span::styled(":scale-out  ", Style::default().fg(Color::DarkGray)),
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
    f.render_widget(footer, chunks[3]);
}

#[allow(clippy::too_many_arguments)]
fn build_condition_panel<'a>(
    conditions: &'a ConditionGroup,
    sel_idx: usize,
    is_active: bool,
    color: Color,
    op_color: Color,
    label: &str,
    key: char,
    empty_msg: &'static str,
) -> List<'a> {
    let items: Vec<ListItem> = if conditions.conditions.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            empty_msg,
            Style::default().fg(Color::DarkGray),
        )]))]
    } else {
        conditions
            .conditions
            .iter()
            .enumerate()
            .flat_map(|(i, cond)| {
                let mut items = Vec::new();
                let is_selected = is_active && i == sel_idx;
                let prefix = if is_selected { "▶ " } else { "  " };
                let style = if is_selected {
                    Style::default().fg(color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(cond.display(), style),
                ])));
                if i < conditions.conditions.len() - 1 {
                    let op_style = if is_selected {
                        Style::default().fg(op_color).add_modifier(Modifier::BOLD)
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

    let title = if is_active {
        format!(" ▶ {} - [{}] add ", label, key)
    } else {
        format!("   {} - [{}] add ", label, key)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_active {
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .title(title);

    List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
}

fn build_scale_panel<'a>(
    conditions: &'a ConditionGroup,
    sel_idx: usize,
    is_active: bool,
    fraction: f64,
    color: Color,
    label: &str,
    key: char,
) -> List<'a> {
    let items: Vec<ListItem> = if conditions.conditions.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            format!("(no conditions - press {} to add)", key),
            Style::default().fg(Color::DarkGray),
        )]))]
    } else {
        conditions
            .conditions
            .iter()
            .enumerate()
            .flat_map(|(i, cond)| {
                let mut items = Vec::new();
                let is_selected = is_active && i == sel_idx;
                let prefix = if is_selected { "▶ " } else { "  " };
                let style = if is_selected {
                    Style::default().fg(color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(cond.display(), style),
                ])));
                if i < conditions.conditions.len() - 1 {
                    let op_style = if is_selected {
                        Style::default().fg(color).add_modifier(Modifier::BOLD)
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

    let pct = (fraction * 100.0).round() as u32;
    let title = if is_active {
        format!(" ▶ {} ({}%) - [{}] add [f] fraction ", label, pct, key)
    } else {
        format!("   {} ({}%) - [{}] add ", label, pct, key)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_active {
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .title(title);

    List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
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
    let htf_supported = ind
        .map(|i| !matches!(i.indicator.category, IndicatorCategory::PriceAction))
        .unwrap_or(true);

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

    let htf_scope_label = app
        .building_htf_interval
        .map(|interval| interval.as_str().to_string())
        .unwrap_or_else(|| "Base timeframe".to_string());

    // Parameters
    let param_lines: Vec<Line> = if let Some(ind) = ind {
        if ind.indicator.params.is_empty() {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(" Scope: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        htf_scope_label,
                        Style::default()
                            .fg(Color::LightBlue)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![Span::styled(
                    if htf_supported {
                        " Press t to cycle higher timeframe scope."
                    } else {
                        " HTF scope unavailable for price-action indicators."
                    },
                    if htf_supported {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::Yellow)
                    },
                )]),
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
            let mut lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(" Scope: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        htf_scope_label,
                        Style::default()
                            .fg(Color::LightBlue)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![Span::styled(
                    if htf_supported {
                        " Press t to cycle higher timeframe scope."
                    } else {
                        " HTF scope unavailable for price-action indicators."
                    },
                    if htf_supported {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::Yellow)
                    },
                )]),
                Line::from(""),
            ];
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
        Span::styled("t", Style::default().fg(Color::LightBlue)),
        Span::styled(
            if htf_supported {
                ":cycle HTF  "
            } else {
                ":HTF unavailable  "
            },
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled("Enter", Style::default().fg(Color::White)),
        Span::styled(":next  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::White)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[2]);
}

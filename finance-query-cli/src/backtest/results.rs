use finance_query::backtesting::BacktestResult;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
};

/// Results viewer tabs
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ResultsTab {
    #[default]
    Overview,
    Trades,
    Signals,
}

impl ResultsTab {
    pub fn all() -> &'static [Self] {
        &[Self::Overview, Self::Trades, Self::Signals]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Trades => "Trades",
            Self::Signals => "Signals",
        }
    }
}

/// Actions from the results TUI
pub enum ResultsAction {
    Continue,
    Quit,
    Retry,
    NewStrategy,
}

/// Results TUI state
pub struct ResultsApp {
    pub result: BacktestResult,
    pub tab: ResultsTab,
    pub scroll: usize,
}

impl ResultsApp {
    pub fn new(result: BacktestResult) -> Self {
        Self {
            result,
            tab: ResultsTab::default(),
            scroll: 0,
        }
    }

    pub fn next_tab(&mut self) {
        let tabs = ResultsTab::all();
        let idx = tabs.iter().position(|t| *t == self.tab).unwrap_or(0);
        self.tab = tabs[(idx + 1) % tabs.len()];
        self.scroll = 0;
    }

    pub fn prev_tab(&mut self) {
        let tabs = ResultsTab::all();
        let idx = tabs.iter().position(|t| *t == self.tab).unwrap_or(0);
        self.tab = tabs[(idx + tabs.len() - 1) % tabs.len()];
        self.scroll = 0;
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }
}

/// Run the results TUI
pub fn run_results_tui(result: BacktestResult) -> crate::error::Result<ResultsAction> {
    use crossterm::{
        ExecutableCommand,
        event::{self, Event, KeyCode, KeyEventKind},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::Terminal;
    use ratatui::backend::CrosstermBackend;
    use std::io::stdout;

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = ResultsApp::new(result);
    let action = loop {
        terminal.draw(|f| results_ui(f, &app))?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') => break ResultsAction::Quit,
                KeyCode::Char('r') => break ResultsAction::Retry,
                KeyCode::Char('n') => break ResultsAction::NewStrategy,
                KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => app.next_tab(),
                KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => app.prev_tab(),
                KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                KeyCode::Enter | KeyCode::Esc => break ResultsAction::Continue,
                _ => {}
            }
        }
    };

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(action)
}

/// Main results UI function
pub fn results_ui(f: &mut Frame, app: &ResultsApp) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    render_results_header(f, app, chunks[0]);
    render_results_tabs(f, app, chunks[1]);

    match app.tab {
        ResultsTab::Overview => render_results_overview(f, app, chunks[2]),
        ResultsTab::Trades => render_results_trades(f, app, chunks[2]),
        ResultsTab::Signals => render_results_signals(f, app, chunks[2]),
    }

    render_results_footer(f, chunks[3]);
}

fn render_results_header(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result;
    let pnl = r.total_pnl();
    let pnl_color = return_color(pnl);

    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " BACKTEST RESULTS ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(&r.symbol, Style::default().fg(Color::Yellow)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(&r.strategy_name, Style::default().fg(Color::White)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled("P&L: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}{:.2}", if pnl >= 0.0 { "+" } else { "" }, pnl),
                Style::default().fg(pnl_color).add_modifier(Modifier::BOLD),
            ),
        ]),
    ]);
    f.render_widget(title, area);
}

fn render_results_tabs(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let tab_titles: Vec<Line> = ResultsTab::all()
        .iter()
        .map(|t| Line::from(t.name()))
        .collect();

    let idx = ResultsTab::all()
        .iter()
        .position(|t| *t == app.tab)
        .unwrap_or(0);

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(idx)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );
    f.render_widget(tabs, area);
}

fn render_results_overview(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result;
    let m = &r.metrics;
    let pnl = r.total_pnl();

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left column - Performance metrics
    let perf_lines = vec![
        Line::from(""),
        metric_line("Starting Capital", &format!("${:.2}", r.initial_capital)),
        metric_line("Ending Capital", &format!("${:.2}", r.final_equity)),
        metric_line(
            "Total P&L",
            &format!("{}{:.2}", if pnl >= 0.0 { "+" } else { "" }, pnl),
        ),
        metric_line(
            "Return",
            &format!(
                "{}{:.2}%",
                if m.total_return_pct >= 0.0 { "+" } else { "" },
                m.total_return_pct
            ),
        ),
        Line::from(""),
        metric_line("Total Trades", &m.total_trades.to_string()),
        metric_line("Winning Trades", &m.winning_trades.to_string()),
        metric_line("Losing Trades", &m.losing_trades.to_string()),
        metric_line("Win Rate", &format!("{:.1}%", m.win_rate * 100.0)),
    ];

    let perf = Paragraph::new(perf_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Performance "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(perf, main_chunks[0]);

    // Right column - Risk metrics
    // Calculate max drawdown in dollars from percentage
    let max_dd_dollars = r.initial_capital * m.max_drawdown_pct;

    // Calculate avg win/loss in dollars (using percentage and approximate trade size)
    let avg_trade_value = r.initial_capital * r.config.position_size_pct;
    let _avg_win = avg_trade_value * (m.avg_win_pct / 100.0);
    let _avg_loss = avg_trade_value * (m.avg_loss_pct.abs() / 100.0);

    let risk_lines = vec![
        Line::from(""),
        metric_line("Profit Factor", &format_ratio(m.profit_factor)),
        metric_line("Max Drawdown", &format!("${:.2}", max_dd_dollars)),
        metric_line(
            "Max Drawdown %",
            &format!("{:.2}%", m.max_drawdown_pct * 100.0),
        ),
        Line::from(""),
        metric_line("Avg Win %", &format!("{:.2}%", m.avg_win_pct)),
        metric_line("Avg Loss %", &format!("{:.2}%", m.avg_loss_pct)),
        metric_line("Sharpe Ratio", &format!("{:.2}", m.sharpe_ratio)),
        Line::from(""),
        metric_line("Largest Win", &format!("${:.2}", m.largest_win)),
        metric_line("Largest Loss", &format!("${:.2}", m.largest_loss)),
    ];

    let risk = Paragraph::new(risk_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Risk Metrics "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(risk, main_chunks[1]);
}

fn render_results_trades(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result;

    let visible_trades: Vec<ListItem> = r
        .trades
        .iter()
        .skip(app.scroll)
        .take(area.height.saturating_sub(2) as usize)
        .enumerate()
        .map(|(idx, trade)| {
            let pnl_color = return_color(trade.pnl);
            let side_str = if trade.is_long() { "LONG" } else { "SHORT" };
            let side_color = if trade.is_long() {
                Color::Green
            } else {
                Color::Red
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!(" #{:<3}", app.scroll + idx + 1),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(format!("{:<5}", side_str), Style::default().fg(side_color)),
                    Span::styled(" Entry: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("${:.2}", trade.entry_price),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(" @ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format_timestamp(trade.entry_timestamp),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("      "),
                    Span::styled(" Exit:  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("${:.2}", trade.exit_price),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(" @ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format_timestamp(trade.exit_timestamp),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled("P&L: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!(
                            "{}{:.2}",
                            if trade.pnl >= 0.0 { "+" } else { "" },
                            trade.pnl
                        ),
                        Style::default().fg(pnl_color),
                    ),
                ]),
            ])
        })
        .collect();

    let list = List::new(visible_trades).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(format!(
                " Trades ({}/{}) - ↑/↓ to scroll ",
                app.scroll + 1,
                r.trades.len()
            )),
    );
    f.render_widget(list, area);
}

fn render_results_signals(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result;

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
                    format_timestamp(signal.timestamp),
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

fn render_results_footer(f: &mut Frame, area: ratatui::layout::Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ←/→", Style::default().fg(Color::White)),
        Span::styled(":tab  ", Style::default().fg(Color::DarkGray)),
        Span::styled("↑/↓", Style::default().fg(Color::White)),
        Span::styled(":scroll  ", Style::default().fg(Color::DarkGray)),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::styled(":retry  ", Style::default().fg(Color::DarkGray)),
        Span::styled("n", Style::default().fg(Color::Cyan)),
        Span::styled(":new strategy  ", Style::default().fg(Color::DarkGray)),
        Span::styled("q", Style::default().fg(Color::White)),
        Span::styled(":quit", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, area);
}

// Helper functions

fn metric_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:<18}", label),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}

fn format_timestamp(ts: i64) -> String {
    // Convert Unix timestamp to readable date
    use chrono::DateTime;
    if let Some(dt) = DateTime::from_timestamp(ts, 0) {
        dt.format("%Y-%m-%d").to_string()
    } else {
        ts.to_string()
    }
}

fn format_ratio(ratio: f64) -> String {
    if ratio.is_infinite() {
        "∞".to_string()
    } else if ratio.is_nan() {
        "-".to_string()
    } else {
        format!("{:.2}", ratio)
    }
}

fn return_color(value: f64) -> Color {
    if value > 0.0 {
        Color::Green
    } else if value < 0.0 {
        Color::Red
    } else {
        Color::DarkGray
    }
}

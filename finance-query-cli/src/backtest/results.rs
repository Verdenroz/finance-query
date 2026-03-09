use chrono::Weekday;
use finance_query::backtesting::portfolio::PortfolioResult;
use finance_query::backtesting::{
    BacktestComparison, BacktestResult, MonteCarloConfig, MonteCarloMethod, MonteCarloResult,
    OptimizationReport, OptimizeMetric, WalkForwardReport,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, BarChart, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph, Tabs,
        Wrap,
    },
};
use std::path::PathBuf;

/// The full result of a run (backtest + optional optimizer/walk-forward outputs).
pub struct RunResult {
    pub backtest: BacktestResult,
    pub optimization: Option<OptimizationReport>,
    pub walk_forward: Option<WalkForwardReport>,
    pub opt_metric: Option<OptimizeMetric>,
    /// Benchmark candles for plotting the actual buy-and-hold equity curve.
    /// When present, the Charts tab uses the real curve instead of a linear
    /// interpolation of the benchmark's total return.
    pub bench_candles: Option<Vec<finance_query::Candle>>,
    /// Portfolio result (set when portfolio mode was used). The `backtest`
    /// field holds the primary symbol's result for single-symbol tabs.
    pub portfolio: Option<PortfolioResult>,
}

impl RunResult {
    pub fn simple(result: BacktestResult) -> Self {
        Self {
            backtest: result,
            optimization: None,
            walk_forward: None,
            opt_metric: None,
            bench_candles: None,
            portfolio: None,
        }
    }
}

/// Results viewer tabs
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ResultsTab {
    #[default]
    Overview,
    Charts,
    Distribution,
    Trades,
    Signals,
    MonteCarlo,
    Periods,
    Comparison,
    Optimizer,
    WalkForward,
    /// Portfolio tab — only shown when a portfolio result is present.
    Portfolio,
}

impl ResultsTab {
    pub fn all_for(
        has_optimizer: bool,
        has_walk_forward: bool,
        has_comparison: bool,
        has_portfolio: bool,
    ) -> Vec<Self> {
        let mut tabs = vec![
            Self::Overview,
            Self::Charts,
            Self::Distribution,
            Self::Trades,
            Self::Signals,
            Self::MonteCarlo,
            Self::Periods,
        ];
        if has_comparison {
            tabs.push(Self::Comparison);
        }
        if has_optimizer {
            tabs.push(Self::Optimizer);
        }
        if has_walk_forward {
            tabs.push(Self::WalkForward);
        }
        if has_portfolio {
            tabs.push(Self::Portfolio);
        }
        tabs
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Charts => "Charts",
            Self::Distribution => "Distribution",
            Self::Trades => "Trades",
            Self::Signals => "Signals",
            Self::MonteCarlo => "Monte Carlo",
            Self::Periods => "Periods",
            Self::Comparison => "Comparison",
            Self::Optimizer => "Optimizer",
            Self::WalkForward => "Walk-Forward",
            Self::Portfolio => "Portfolio",
        }
    }
}

/// Bottom pane of the Charts tab.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartsView {
    #[default]
    Drawdown,
    RollingSharpe,
    RollingWinRate,
}

impl ChartsView {
    fn cycle(self) -> Self {
        match self {
            Self::Drawdown => Self::RollingSharpe,
            Self::RollingSharpe => Self::RollingWinRate,
            Self::RollingWinRate => Self::Drawdown,
        }
    }
}

/// Which period breakdown to display in the Periods tab.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PeriodsMode {
    #[default]
    Yearly,
    Monthly,
    DayOfWeek,
}

impl PeriodsMode {
    fn cycle(self) -> Self {
        match self {
            Self::Yearly => Self::Monthly,
            Self::Monthly => Self::DayOfWeek,
            Self::DayOfWeek => Self::Yearly,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Yearly => "Yearly",
            Self::Monthly => "Monthly",
            Self::DayOfWeek => "Day of Week",
        }
    }
}

/// Actions from the results TUI
pub enum ResultsAction {
    Quit,
    Retry,
    NewStrategy,
}

/// Results TUI state
pub struct ResultsApp {
    pub result: RunResult,
    pub monte_carlo: MonteCarloResult,
    pub tab: ResultsTab,
    pub scroll: usize,
    /// Which breakdown to show in the Periods tab.
    pub periods_mode: PeriodsMode,
    /// Which chart to show in the bottom pane of the Charts tab.
    pub charts_view: ChartsView,
    /// Saved results for comparison (label, result). Capped at 5.
    pub saved_results: Vec<(String, BacktestResult)>,
    /// Metric used to rank in the Comparison tab.
    pub comparison_metric: OptimizeMetric,
    /// Monte Carlo resampling method.
    pub mc_method: MonteCarloMethod,
    /// Whether the diagnostics banner is visible (dismissed with d).
    pub show_diagnostics: bool,
    /// Status message shown after export (Some = path exported to, None = idle)
    pub export_status: Option<Result<PathBuf, String>>,
}

impl ResultsApp {
    pub fn new(result: RunResult) -> Self {
        let monte_carlo = MonteCarloConfig::default().run(&result.backtest);
        // Default to Portfolio tab when portfolio mode was used so the user sees
        // the aggregate view immediately rather than a single-symbol overview.
        let tab = if result.portfolio.is_some() {
            ResultsTab::Portfolio
        } else {
            ResultsTab::default()
        };
        Self {
            result,
            monte_carlo,
            tab,
            scroll: 0,
            periods_mode: PeriodsMode::default(),
            charts_view: ChartsView::default(),
            saved_results: Vec::new(),
            comparison_metric: OptimizeMetric::SharpeRatio,
            mc_method: MonteCarloMethod::IidShuffle,
            show_diagnostics: true,
            export_status: None,
        }
    }

    pub fn export_csv(&mut self) {
        let path = if let Some(portfolio) = &self.result.portfolio {
            export_portfolio_csv(portfolio)
        } else {
            export_trades_csv(&self.result.backtest)
        };
        self.export_status = Some(path);
    }

    fn tabs(&self) -> Vec<ResultsTab> {
        ResultsTab::all_for(
            self.result.optimization.is_some(),
            self.result.walk_forward.is_some(),
            self.saved_results.len() >= 2,
            self.result.portfolio.is_some(),
        )
    }

    pub fn next_tab(&mut self) {
        let tabs = self.tabs();
        let idx = tabs.iter().position(|t| *t == self.tab).unwrap_or(0);
        self.tab = tabs[(idx + 1) % tabs.len()];
        self.scroll = 0;
    }

    pub fn prev_tab(&mut self) {
        let tabs = self.tabs();
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
pub fn run_results_tui(result: RunResult) -> crate::error::Result<ResultsAction> {
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
                KeyCode::Char('e') => app.export_csv(),
                KeyCode::Char('d') => app.show_diagnostics = !app.show_diagnostics,
                KeyCode::Char('c') => {
                    // Comparison is per-symbol only; skip silently in portfolio mode
                    // to avoid storing a single-symbol result that misrepresents the portfolio.
                    if app.result.portfolio.is_none() && app.saved_results.len() < 5 {
                        let n = app.saved_results.len() + 1;
                        let label = format!("Run {} — {}", n, app.result.backtest.strategy_name);
                        app.saved_results.push((label, app.result.backtest.clone()));
                    }
                }
                KeyCode::Char('x') => app.saved_results.clear(),
                KeyCode::Char('m') => {
                    // Cycle period breakdown or comparison metric depending on active tab
                    match app.tab {
                        ResultsTab::Periods => {
                            app.periods_mode = app.periods_mode.cycle();
                        }
                        ResultsTab::Comparison => {
                            app.comparison_metric = next_optimize_metric(app.comparison_metric);
                        }
                        _ => {}
                    }
                }
                KeyCode::Char('s') => app.charts_view = app.charts_view.cycle(),
                KeyCode::Char('v') => {
                    app.mc_method = next_mc_method(&app.mc_method);
                    app.monte_carlo = MonteCarloConfig::default()
                        .method(app.mc_method.clone())
                        .run(&app.result.backtest);
                }
                KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => app.next_tab(),
                KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => app.prev_tab(),
                KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                KeyCode::Enter => break ResultsAction::NewStrategy,
                KeyCode::Esc => break ResultsAction::Quit,
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

    let diagnostics = &app.result.backtest.diagnostics;
    let diag_height = if app.show_diagnostics && !diagnostics.is_empty() {
        diagnostics.len().min(3) as u16 + 2
    } else {
        0
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(diag_height),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    render_results_header(f, app, chunks[0]);
    render_results_tabs(f, app, chunks[1]);

    if diag_height > 0 {
        render_diagnostics_banner(f, app, chunks[2]);
    }

    match app.tab {
        ResultsTab::Overview => render_results_overview(f, app, chunks[3]),
        ResultsTab::Charts => render_charts(f, app, chunks[3]),
        ResultsTab::Distribution => render_distribution(f, app, chunks[3]),
        ResultsTab::Trades => render_results_trades(f, app, chunks[3]),
        ResultsTab::Signals => render_results_signals(f, app, chunks[3]),
        ResultsTab::MonteCarlo => render_monte_carlo(f, app, chunks[3]),
        ResultsTab::Periods => render_periods(f, app, chunks[3]),
        ResultsTab::Comparison => render_comparison(f, app, chunks[3]),
        ResultsTab::Optimizer => render_optimizer_results(f, app, chunks[3]),
        ResultsTab::WalkForward => render_walk_forward_results(f, app, chunks[3]),
        ResultsTab::Portfolio => render_portfolio(f, app, chunks[3]),
    }

    render_results_footer(f, app, chunks[4]);
}

fn render_diagnostics_banner(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let diags = &app.result.backtest.diagnostics;
    let lines: Vec<Line> = diags
        .iter()
        .take(3)
        .map(|msg| {
            Line::from(vec![
                Span::styled(" ⚠ ", Style::default().fg(Color::Yellow)),
                Span::styled(msg.as_str(), Style::default().fg(Color::Yellow)),
            ])
        })
        .collect();

    let banner = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .title(" ⚠ Diagnostics — press d to dismiss "),
    );
    f.render_widget(banner, area);
}

fn render_results_header(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result.backtest;
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
    let tabs_list = app.tabs();
    let tab_titles: Vec<Line> = tabs_list.iter().map(|t| Line::from(t.name())).collect();
    let idx = tabs_list.iter().position(|t| *t == app.tab).unwrap_or(0);

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
    let r = &app.result.backtest;
    let m = &r.metrics;
    let pnl = r.total_pnl();

    // Split vertically if benchmark data is present
    let has_benchmark = r.benchmark.is_some();
    let (metrics_area, bench_area) = if has_benchmark {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(area);
        (vert[0], Some(vert[1]))
    } else {
        (area, None)
    };

    // 3-column metrics layout
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(metrics_area);

    // Column 1 — Performance
    let perf_lines = vec![
        Line::from(""),
        metric_line("Start Capital", &format!("${:.2}", r.initial_capital)),
        metric_line("End Capital", &format!("${:.2}", r.final_equity)),
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
        metric_line(
            "Ann. Return",
            &format!(
                "{}{:.2}%",
                if m.annualized_return_pct >= 0.0 {
                    "+"
                } else {
                    ""
                },
                m.annualized_return_pct
            ),
        ),
        Line::from(""),
        metric_line("Total Trades", &m.total_trades.to_string()),
        metric_line("Long Trades", &m.long_trades.to_string()),
        metric_line("Short Trades", &m.short_trades.to_string()),
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
    f.render_widget(perf, cols[0]);

    // Column 2 — Risk
    let max_dd_dollars = max_drawdown_dollars(r.equity_curve.iter().map(|p| p.equity));

    let risk_lines = vec![
        Line::from(""),
        metric_line("Profit Factor", &format_ratio(m.profit_factor)),
        metric_line("Max Drawdown", &format!("${:.2}", max_dd_dollars)),
        metric_line(
            "Max Drawdown %",
            &format!("{:.2}%", m.max_drawdown_pct * 100.0),
        ),
        metric_line("DD Duration", &format!("{} bars", m.max_drawdown_duration)),
        Line::from(""),
        metric_line(
            &format!("Sharpe (RF {:.1}%)", r.config.risk_free_rate * 100.0),
            &format_ratio(m.sharpe_ratio),
        ),
        metric_line(
            &format!("Sortino (RF {:.1}%)", r.config.risk_free_rate * 100.0),
            &format_ratio(m.sortino_ratio),
        ),
        metric_line("Calmar Ratio", &format_ratio(m.calmar_ratio)),
        metric_line("Omega Ratio", &format_ratio(m.omega_ratio)),
        metric_line("Serenity Ratio", &format_ratio(m.serenity_ratio)),
        Line::from(""),
        metric_line("Ulcer Index", &format!("{:.4}", m.ulcer_index)),
        metric_line("Tail Ratio", &format_ratio(m.tail_ratio)),
        metric_line("Recovery Factor", &format_ratio(m.recovery_factor)),
        Line::from(""),
        metric_line("Avg Win %", &format!("{:.2}%", m.avg_win_pct)),
        metric_line("Avg Loss %", &format!("{:.2}%", m.avg_loss_pct)),
        metric_line("Avg Trade Ret", &format!("{:.2}%", m.avg_trade_return_pct)),
        metric_line("Avg Win Dur", &format_duration_secs(m.avg_win_duration)),
        metric_line("Avg Loss Dur", &format_duration_secs(m.avg_loss_duration)),
    ];

    let risk = Paragraph::new(risk_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Risk Metrics "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(risk, cols[1]);

    // Column 3 — Activity
    let mut activity_lines = vec![
        Line::from(""),
        metric_line("Winning Trades", &m.winning_trades.to_string()),
        metric_line("Losing Trades", &m.losing_trades.to_string()),
        metric_line("Max Consec Wins", &m.max_consecutive_wins.to_string()),
        metric_line("Max Consec Loss", &m.max_consecutive_losses.to_string()),
        Line::from(""),
        metric_line(
            "Time in Market",
            &format!("{:.1}%", m.time_in_market_pct * 100.0),
        ),
        metric_line(
            "Avg Trade Dur",
            &format!("{:.1} bars", m.avg_trade_duration),
        ),
        metric_line(
            "Max Idle Period",
            &format_duration_secs(m.max_idle_period as f64),
        ),
        Line::from(""),
        metric_line("Commission Paid", &format!("${:.2}", m.total_commission)),
    ];

    if m.total_dividend_income > 0.0 {
        activity_lines.push(metric_line(
            "Dividend Income",
            &format!("${:.2}", m.total_dividend_income),
        ));
    }

    activity_lines.extend([
        Line::from(""),
        metric_line("Largest Win", &format!("${:.2}", m.largest_win)),
        metric_line("Largest Loss", &format!("${:.2}", m.largest_loss)),
        Line::from(""),
        metric_line(
            "Kelly Criterion",
            &format!("{:.2}%", m.kelly_criterion * 100.0),
        ),
        metric_line("SQN", &format!("{:.2}", m.sqn)),
        metric_line("Expectancy", &format!("${:.2}", m.expectancy)),
        Line::from(""),
        metric_line("Total Signals", &m.total_signals.to_string()),
        metric_line("Executed", &m.executed_signals.to_string()),
    ]);

    let activity = Paragraph::new(activity_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" Activity "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(activity, cols[2]);

    // Optional benchmark section
    if let (Some(bench), Some(ba)) = (&r.benchmark, bench_area) {
        let bench_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(ba);

        let left_lines = vec![
            Line::from(""),
            metric_line("Benchmark", &bench.symbol),
            metric_line(
                "Benchmark Return",
                &format!("{:.2}%", bench.benchmark_return_pct),
            ),
            metric_line(
                "Buy & Hold Ret",
                &format!("{:.2}%", bench.buy_and_hold_return_pct),
            ),
        ];

        let right_lines = vec![
            Line::from(""),
            metric_line("Alpha", &format!("{:.2}%", bench.alpha)),
            metric_line("Beta", &format!("{:.3}", bench.beta)),
            metric_line("Info Ratio", &format_ratio(bench.information_ratio)),
        ];

        let bench_left = Paragraph::new(left_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .title(" Benchmark "),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(bench_left, bench_cols[0]);

        let bench_right = Paragraph::new(right_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .title(" Alpha / Beta "),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(bench_right, bench_cols[1]);
    }
}

fn render_results_trades(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result.backtest;
    let intraday = is_intraday(&r.equity_curve);

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
                        format_timestamp_with_precision(trade.entry_timestamp, intraday),
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
                        format_timestamp_with_precision(trade.exit_timestamp, intraday),
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
    let r = &app.result.backtest;
    let intraday = is_intraday(&r.equity_curve);

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
                    format_timestamp_with_precision(signal.timestamp, intraday),
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

fn render_results_footer(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let status_line = match &app.export_status {
        Some(Ok(path)) => Line::from(vec![
            Span::styled(" Exported: ", Style::default().fg(Color::Green)),
            Span::styled(
                path.display().to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
        Some(Err(e)) => Line::from(vec![Span::styled(
            format!(" Export failed: {}", e),
            Style::default().fg(Color::Red),
        )]),
        None => Line::from(vec![
            Span::styled(" ←/→", Style::default().fg(Color::White)),
            Span::styled(":tab  ", Style::default().fg(Color::DarkGray)),
            Span::styled("↑/↓", Style::default().fg(Color::White)),
            Span::styled(":scroll  ", Style::default().fg(Color::DarkGray)),
            Span::styled("e", Style::default().fg(Color::Green)),
            Span::styled(":export csv  ", Style::default().fg(Color::DarkGray)),
            Span::styled("r", Style::default().fg(Color::Yellow)),
            Span::styled(":retry  ", Style::default().fg(Color::DarkGray)),
            Span::styled("n", Style::default().fg(Color::Cyan)),
            Span::styled(":new strategy  ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().fg(Color::White)),
            Span::styled(":quit", Style::default().fg(Color::DarkGray)),
        ]),
    };
    f.render_widget(Paragraph::new(status_line), area);
}

fn next_mc_method(m: &MonteCarloMethod) -> MonteCarloMethod {
    match m {
        MonteCarloMethod::IidShuffle => MonteCarloMethod::BlockBootstrap { block_size: 10 },
        MonteCarloMethod::BlockBootstrap { .. } => MonteCarloMethod::StationaryBootstrap {
            mean_block_size: 10,
        },
        MonteCarloMethod::StationaryBootstrap { .. } => MonteCarloMethod::Parametric,
        _ => MonteCarloMethod::IidShuffle,
    }
}

fn mc_method_name(m: &MonteCarloMethod) -> &'static str {
    match m {
        MonteCarloMethod::IidShuffle => "IID Shuffle",
        MonteCarloMethod::BlockBootstrap { .. } => "Block Bootstrap (10)",
        MonteCarloMethod::StationaryBootstrap { .. } => "Stationary Bootstrap (10)",
        MonteCarloMethod::Parametric => "Parametric",
        _ => "Unknown",
    }
}

fn next_optimize_metric(m: OptimizeMetric) -> OptimizeMetric {
    match m {
        OptimizeMetric::SharpeRatio => OptimizeMetric::SortinoRatio,
        OptimizeMetric::SortinoRatio => OptimizeMetric::TotalReturn,
        OptimizeMetric::TotalReturn => OptimizeMetric::WinRate,
        OptimizeMetric::WinRate => OptimizeMetric::ProfitFactor,
        OptimizeMetric::ProfitFactor => OptimizeMetric::MinDrawdown,
        OptimizeMetric::MinDrawdown => OptimizeMetric::CalmarRatio,
        OptimizeMetric::CalmarRatio => OptimizeMetric::SharpeRatio,
        _ => OptimizeMetric::SharpeRatio,
    }
}

fn render_comparison(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    use ratatui::widgets::Row;
    use ratatui::widgets::Table;

    if app.saved_results.len() < 2 {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                " Save at least 2 results with 'c' to compare them.",
                Style::default().fg(Color::DarkGray),
            )]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Comparison "),
        );
        f.render_widget(msg, area);
        return;
    }

    let mut cmp = BacktestComparison::new();
    for (label, result) in &app.saved_results {
        cmp = cmp.add(label.clone(), result.clone());
    }
    let report = cmp.ranked_by(app.comparison_metric);

    let metric_name = match app.comparison_metric {
        OptimizeMetric::SharpeRatio => "Sharpe",
        OptimizeMetric::SortinoRatio => "Sortino",
        OptimizeMetric::TotalReturn => "Return",
        OptimizeMetric::WinRate => "Win Rate",
        OptimizeMetric::ProfitFactor => "Profit Factor",
        OptimizeMetric::MinDrawdown => "Max DD",
        OptimizeMetric::CalmarRatio => "Calmar",
        _ => "Sharpe",
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let header_para = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" Ranked by: {} ", metric_name),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " (press m to cycle metric, c to add current run, x to clear)",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    f.render_widget(header_para, chunks[0]);

    let header = Row::new(vec![
        "Rank", "Label", "Return %", "Sharpe", "Sortino", "Max DD %", "Win Rate", "SQN",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let rows: Vec<Row> = report
        .table()
        .iter()
        .map(|row| {
            let color = if row.total_return_pct >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            Row::new(vec![
                format!("#{}", row.rank),
                row.label.clone(),
                format!("{:+.2}%", row.total_return_pct),
                format!("{:.2}", row.sharpe_ratio),
                format!("{:.2}", row.sortino_ratio),
                format!("{:.2}%", row.max_drawdown_pct * 100.0),
                format!("{:.1}%", row.win_rate * 100.0),
                format!("{:.2}", row.sqn),
            ])
            .style(Style::default().fg(color))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Min(20),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(6),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(format!(" Comparison ({} runs) ", report.len())),
    );

    f.render_widget(table, chunks[1]);
}

fn render_periods(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    use ratatui::widgets::Row;
    use ratatui::widgets::Table;
    let r = &app.result.backtest;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let header_para = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" Breakdown by {} ", app.periods_mode.label()),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " (m: cycle yearly/monthly/day-of-week  ↑/↓: scroll)",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    f.render_widget(header_para, chunks[0]);

    match app.periods_mode {
        PeriodsMode::Monthly => {
            let mut data = r.by_month().into_iter().collect::<Vec<_>>();
            data.sort_by_key(|(k, _)| *k);

            let header = Row::new(vec!["Month", "Return %", "Win Rate", "Trades", "Max DD %"])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1);

            let visible_rows: Vec<Row> = data
                .iter()
                .skip(app.scroll)
                .take(chunks[1].height.saturating_sub(3) as usize)
                .map(|((year, month), m)| {
                    let ret = m.total_return_pct;
                    let color = if ret >= 0.0 { Color::Green } else { Color::Red };
                    Row::new(vec![
                        format!("{}-{:02}", year, month),
                        format!("{:+.2}%", ret),
                        format!("{:.1}%", m.win_rate * 100.0),
                        m.total_trades.to_string(),
                        format!("{:.2}%", m.max_drawdown_pct * 100.0),
                    ])
                    .style(Style::default().fg(color))
                })
                .collect();

            let table = Table::new(
                visible_rows,
                [
                    Constraint::Length(10),
                    Constraint::Length(12),
                    Constraint::Length(10),
                    Constraint::Length(8),
                    Constraint::Length(10),
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(format!(" Monthly ({}/{}) ", app.scroll + 1, data.len())),
            );
            f.render_widget(table, chunks[1]);
        }
        PeriodsMode::Yearly => {
            let mut data = r.by_year().into_iter().collect::<Vec<_>>();
            data.sort_by_key(|(k, _)| *k);

            let header = Row::new(vec!["Year", "Return %", "Win Rate", "Trades", "Max DD %"])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1);

            let visible_rows: Vec<Row> = data
                .iter()
                .skip(app.scroll)
                .take(chunks[1].height.saturating_sub(3) as usize)
                .map(|(year, m)| {
                    let ret = m.total_return_pct;
                    let color = if ret >= 0.0 { Color::Green } else { Color::Red };
                    Row::new(vec![
                        year.to_string(),
                        format!("{:+.2}%", ret),
                        format!("{:.1}%", m.win_rate * 100.0),
                        m.total_trades.to_string(),
                        format!("{:.2}%", m.max_drawdown_pct * 100.0),
                    ])
                    .style(Style::default().fg(color))
                })
                .collect();

            let table = Table::new(
                visible_rows,
                [
                    Constraint::Length(6),
                    Constraint::Length(12),
                    Constraint::Length(10),
                    Constraint::Length(8),
                    Constraint::Length(10),
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(format!(" Yearly ({}/{}) ", app.scroll + 1, data.len())),
            );
            f.render_widget(table, chunks[1]);
        }
        PeriodsMode::DayOfWeek => {
            // Canonical Mon–Sun ordering; Sat/Sun matter for 24/7 assets (crypto)
            const DOW_ORDER: [Weekday; 7] = [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ];

            let dow_map = r.by_day_of_week();
            let data: Vec<(Weekday, &finance_query::backtesting::PerformanceMetrics)> = DOW_ORDER
                .iter()
                .filter_map(|d| dow_map.get(d).map(|m| (*d, m)))
                .collect();

            if data.is_empty() {
                let msg = Paragraph::new("Not enough trades to compute day-of-week breakdown.")
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Day of Week "),
                    );
                f.render_widget(msg, chunks[1]);
                return;
            }

            let header = Row::new(vec!["Day", "Return %", "Win Rate", "Trades", "Avg Trade %"])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1);

            let rows: Vec<Row> = data
                .iter()
                .map(|(day, m)| {
                    let ret = m.total_return_pct;
                    let color = if ret >= 0.0 { Color::Green } else { Color::Red };
                    Row::new(vec![
                        format!("{}", day),
                        format!("{:+.2}%", ret),
                        format!("{:.1}%", m.win_rate * 100.0),
                        m.total_trades.to_string(),
                        format!("{:+.2}%", m.avg_trade_return_pct),
                    ])
                    .style(Style::default().fg(color))
                })
                .collect();

            let table = Table::new(
                rows,
                [
                    Constraint::Length(10),
                    Constraint::Length(12),
                    Constraint::Length(10),
                    Constraint::Length(8),
                    Constraint::Length(12),
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Day of Week "),
            );
            f.render_widget(table, chunks[1]);
        }
    }
}

fn render_monte_carlo(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let mc = &app.monte_carlo;

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let method_para = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" Method: {} ", mc_method_name(&app.mc_method)),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " (press v to cycle method)",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    f.render_widget(method_para, outer[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(outer[1]);

    let render_stat = |f: &mut Frame,
                       title: &str,
                       p5: f64,
                       p50: f64,
                       p95: f64,
                       is_pct: bool,
                       col: ratatui::layout::Rect| {
        let fmt = |v: f64| -> String {
            if is_pct {
                format_signed_pct(v * 100.0)
            } else {
                format_ratio(v)
            }
        };
        let p5_color = if p5 >= 0.0 { Color::Green } else { Color::Red };
        let p50_color = if p50 >= 0.0 { Color::Green } else { Color::Red };
        let p95_color = if p95 >= 0.0 { Color::Green } else { Color::Red };

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  p5  ", Style::default().fg(Color::DarkGray)),
                Span::styled(fmt(p5), Style::default().fg(p5_color)),
            ]),
            Line::from(vec![
                Span::styled("  p50 ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    fmt(p50),
                    Style::default().fg(p50_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  p95 ", Style::default().fg(Color::DarkGray)),
                Span::styled(fmt(p95), Style::default().fg(p95_color)),
            ]),
        ];

        let para = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(format!(" {} ", title)),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(para, col);
    };

    render_stat(
        f,
        "Total Return",
        mc.total_return.p5,
        mc.total_return.p50,
        mc.total_return.p95,
        true,
        cols[0],
    );
    render_stat(
        f,
        "Max Drawdown",
        -mc.max_drawdown.p95,
        -mc.max_drawdown.p50,
        -mc.max_drawdown.p5,
        true,
        cols[1],
    );
    render_stat(
        f,
        "Sharpe Ratio",
        mc.sharpe_ratio.p5,
        mc.sharpe_ratio.p50,
        mc.sharpe_ratio.p95,
        false,
        cols[2],
    );
    render_stat(
        f,
        "Profit Factor",
        mc.profit_factor.p5,
        mc.profit_factor.p50,
        mc.profit_factor.p95,
        false,
        cols[3],
    );
}

// Charts tab: equity curve (top) + drawdown (bottom)

fn render_charts(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let curve = &app.result.backtest.equity_curve;
    if curve.is_empty() {
        let msg = Paragraph::new("No equity curve data.")
            .block(Block::default().borders(Borders::ALL).title(" Charts "));
        f.render_widget(msg, area);
        return;
    }

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    render_equity_chart(f, app, split[0]);
    match app.charts_view {
        ChartsView::RollingSharpe => render_rolling_sharpe_chart(f, app, split[1]),
        ChartsView::RollingWinRate => render_rolling_win_rate_chart(f, app, split[1]),
        ChartsView::Drawdown => render_drawdown_chart(f, app, split[1]),
    }
}

fn render_equity_chart(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result.backtest;
    let curve = &r.equity_curve;

    let strategy_data: Vec<(f64, f64)> = curve
        .iter()
        .enumerate()
        .map(|(i, p)| (i as f64, p.equity))
        .collect();

    let n = curve.len() as f64;
    let min_equity = curve.iter().map(|p| p.equity).fold(f64::MAX, f64::min);
    let max_equity = curve.iter().map(|p| p.equity).fold(f64::MIN, f64::max);
    let y_margin = ((max_equity - min_equity) * 0.05).max(1.0);
    let y_min = (min_equity - y_margin).max(0.0);
    let y_max = max_equity + y_margin;

    let mut datasets = vec![
        Dataset::default()
            .name(format!("{} ({})", r.strategy_name, r.symbol))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&strategy_data),
    ];

    // Benchmark overlay: actual buy-and-hold equity curve from candle closes,
    // falling back to a linear approximation when candles aren't available.
    let bench_data: Vec<(f64, f64)>;
    if let Some(ref bench) = r.benchmark {
        bench_data = if let Some(ref candles) = app.result.bench_candles {
            let first_close = candles.first().map(|c| c.close).unwrap_or(1.0);
            candles
                .iter()
                .enumerate()
                .map(|(i, c)| (i as f64, r.initial_capital * (c.close / first_close)))
                .collect()
        } else {
            // Fallback: linear interpolation of total return (no intra-period detail)
            let end_equity = r.initial_capital * (1.0 + bench.benchmark_return_pct / 100.0);
            (0..curve.len())
                .map(|i| {
                    let frac = if n > 1.0 { i as f64 / (n - 1.0) } else { 1.0 };
                    (
                        i as f64,
                        r.initial_capital + (end_equity - r.initial_capital) * frac,
                    )
                })
                .collect()
        };
        let label = if app.result.bench_candles.is_some() {
            format!("{} B&H", bench.symbol)
        } else {
            format!("{} B&H (approx)", bench.symbol)
        };
        datasets.push(
            Dataset::default()
                .name(label)
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&bench_data),
        );
    }

    let x_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .bounds([0.0, (curve.len().saturating_sub(1)) as f64]);

    let y_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .labels(vec![
            Span::raw(format!("${:.0}", y_min)),
            Span::raw(format!("${:.0}", (y_min + y_max) / 2.0)),
            Span::raw(format!("${:.0}", y_max)),
        ])
        .bounds([y_min, y_max]);

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Equity Curve "),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    f.render_widget(chart, area);
}

fn render_drawdown_chart(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let curve = &app.result.backtest.equity_curve;

    // drawdown_pct stored as fraction (0-1); display as negative percentage
    let dd_data: Vec<(f64, f64)> = curve
        .iter()
        .enumerate()
        .map(|(i, p)| (i as f64, -(p.drawdown_pct * 100.0)))
        .collect();

    let max_dd = curve
        .iter()
        .map(|p| p.drawdown_pct * 100.0)
        .fold(0.0_f64, f64::max);

    let y_min = -(max_dd * 1.1).max(1.0);

    let x_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .bounds([0.0, (curve.len().saturating_sub(1)) as f64]);

    let y_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .labels(vec![
            Span::raw(format!("{:.1}%", y_min)),
            Span::raw(format!("{:.1}%", y_min / 2.0)),
            Span::raw("0.0%"),
        ])
        .bounds([y_min, 0.0]);

    let dataset = Dataset::default()
        .name("Drawdown")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Red))
        .data(&dd_data);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title(" Drawdown % "),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    f.render_widget(chart, area);
}

fn render_rolling_sharpe_chart(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result.backtest;
    const WINDOW: usize = 63;
    let rolling = r.rolling_sharpe(WINDOW);

    if rolling.is_empty() {
        let msg = Paragraph::new(format!(
            "Not enough data for rolling Sharpe (need >{} bars).",
            WINDOW
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Rolling Sharpe ({}-bar) ", WINDOW)),
        );
        f.render_widget(msg, area);
        return;
    }

    let offset = r.equity_curve.len().saturating_sub(rolling.len());
    let data: Vec<(f64, f64)> = rolling
        .iter()
        .enumerate()
        .map(|(i, &v)| ((i + offset) as f64, v))
        .collect();

    let y_min = rolling.iter().cloned().fold(f64::MAX, f64::min);
    let y_max = rolling.iter().cloned().fold(f64::MIN, f64::max);
    let margin = ((y_max - y_min) * 0.1).max(0.1);
    let y_lo = y_min - margin;
    let y_hi = y_max + margin;

    let x_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .bounds([0.0, (r.equity_curve.len().saturating_sub(1)) as f64]);

    let y_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .labels(vec![
            Span::raw(format!("{:.1}", y_lo)),
            Span::raw(format!("{:.1}", (y_lo + y_hi) / 2.0)),
            Span::raw(format!("{:.1}", y_hi)),
        ])
        .bounds([y_lo, y_hi]);

    let dataset = Dataset::default()
        .name(format!("Rolling Sharpe ({})", WINDOW))
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Magenta))
        .data(&data);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(format!(
                    " Rolling Sharpe ({}-bar) — s: cycle chart ",
                    WINDOW
                )),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    f.render_widget(chart, area);
}

fn render_rolling_win_rate_chart(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result.backtest;
    const WINDOW: usize = 20;
    let rolling = r.rolling_win_rate(WINDOW);

    if rolling.is_empty() {
        let msg = Paragraph::new(format!(
            "Not enough trades for rolling win rate (need >{} trades).",
            WINDOW
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Rolling Win Rate ({}-trade) ", WINDOW)),
        );
        f.render_widget(msg, area);
        return;
    }

    // rolling_win_rate is indexed by trade, not by bar — align to equity curve
    // length by offsetting to the right so the last value aligns with the end.
    let total_bars = r.equity_curve.len();
    let offset = total_bars.saturating_sub(rolling.len());
    let data: Vec<(f64, f64)> = rolling
        .iter()
        .enumerate()
        .map(|(i, &v)| ((i + offset) as f64, v * 100.0))
        .collect();

    let x_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .bounds([0.0, total_bars.saturating_sub(1) as f64]);

    let y_axis = Axis::default()
        .style(Style::default().fg(Color::DarkGray))
        .labels(vec![Span::raw("0%"), Span::raw("50%"), Span::raw("100%")])
        .bounds([0.0, 100.0]);

    let dataset = Dataset::default()
        .name(format!("Win Rate % ({}-trade)", WINDOW))
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Green))
        .data(&data);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(format!(
                    " Rolling Win Rate ({}-trade window) — s: cycle chart ",
                    WINDOW
                )),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    f.render_widget(chart, area);
}

// Distribution tab: trade P&L histogram

fn render_distribution(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let trades = &app.result.backtest.trades;

    if trades.is_empty() {
        let msg = Paragraph::new("No trades to display.").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" P&L Distribution "),
        );
        f.render_widget(msg, area);
        return;
    }

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    render_pnl_histogram(f, app, split[0]);
    render_distribution_stats(f, app, split[1]);
}

fn render_pnl_histogram(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let trades = &app.result.backtest.trades;
    let pnls: Vec<f64> = trades.iter().map(|t| t.pnl).collect();

    let min_pnl = pnls.iter().cloned().fold(f64::MAX, f64::min);
    let max_pnl = pnls.iter().cloned().fold(f64::MIN, f64::max);

    const BINS: usize = 10;
    let range = (max_pnl - min_pnl).max(1e-9);
    let bin_width = range / BINS as f64;

    let mut counts = [0u64; BINS];
    for &p in &pnls {
        let idx = ((p - min_pnl) / bin_width).floor() as usize;
        let idx = idx.min(BINS - 1);
        counts[idx] += 1;
    }

    let bar_data: Vec<(String, u64)> = counts
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let center = min_pnl + bin_width * (i as f64 + 0.5);
            let label = if center >= 0.0 {
                format!("+{:.0}", center)
            } else {
                format!("{:.0}", center)
            };
            (label, c)
        })
        .collect();

    let bar_refs: Vec<(&str, u64)> = bar_data
        .iter()
        .map(|(label, count)| (label.as_str(), *count))
        .collect();

    let max_count = counts.iter().cloned().max().unwrap_or(1).max(1);

    let chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" P&L Distribution (trade count per bucket) "),
        )
        .data(&bar_refs)
        .bar_width(((area.width.saturating_sub(4)) / BINS as u16).max(3))
        .bar_gap(1)
        .max(max_count)
        .bar_style(Style::default().fg(Color::Cyan))
        .value_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .label_style(Style::default().fg(Color::DarkGray));

    f.render_widget(chart, area);
}

fn render_distribution_stats(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let trades = &app.result.backtest.trades;
    let pnls: Vec<f64> = trades.iter().map(|t| t.pnl).collect();

    let mut sorted = pnls.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted.len();
    let median = if n.is_multiple_of(2) {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    };

    let mean = pnls.iter().sum::<f64>() / n as f64;
    let variance = if n > 1 {
        pnls.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / (n - 1) as f64
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    let p25 = percentile(&sorted, 0.25);
    let p75 = percentile(&sorted, 0.75);

    let m = &app.result.backtest.metrics;
    let lines = vec![
        Line::from(""),
        metric_line("Trades", &n.to_string()),
        metric_line("Mean P&L", &format!("${:.2}", mean)),
        metric_line("Median P&L", &format!("${:.2}", median)),
        metric_line("Std Dev", &format!("${:.2}", std_dev)),
        Line::from(""),
        metric_line("p25", &format!("${:.2}", p25)),
        metric_line("p75", &format!("${:.2}", p75)),
        Line::from(""),
        metric_line(
            "Wins",
            &format!("{} ({:.0}%)", m.winning_trades, m.win_rate * 100.0),
        ),
        metric_line("Losses", &format!("{}", m.losing_trades)),
    ];

    let stats = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" Stats "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(stats, area);
}

// Optimizer tab: best params + ranked results list

fn render_optimizer_results(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let Some(ref opt) = app.result.optimization else {
        let msg = Paragraph::new("No optimization data.")
            .block(Block::default().borders(Borders::ALL).title(" Optimizer "));
        f.render_widget(msg, area);
        return;
    };

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(area);

    // Summary
    let best = &opt.best;
    let mut params_parts: Vec<String> = best
        .params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    params_parts.sort();
    let params_str = params_parts.join("  ");

    let opt_metric = app.result.opt_metric.unwrap_or(OptimizeMetric::SharpeRatio);
    let (best_metric_label, best_metric_value) = metric_score_display(opt_metric, &best.result);

    let summary_lines = vec![
        Line::from(""),
        metric_line("Strategy", &opt.strategy_name),
        metric_line("Total Combos", &opt.total_combinations.to_string()),
        metric_line("Best Params", &params_str),
        metric_line(&format!("Best {}", best_metric_label), &best_metric_value),
        metric_line(
            "Best Return",
            &format!("{:+.2}%", best.result.metrics.total_return_pct),
        ),
    ];

    let summary = Paragraph::new(summary_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Optimizer Summary "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(summary, split[0]);

    // Ranked results list
    let visible: Vec<ListItem> = opt
        .results
        .iter()
        .skip(app.scroll)
        .take(split[1].height.saturating_sub(2) as usize)
        .enumerate()
        .map(|(i, res)| {
            let rank = app.scroll + i + 1;
            let mut parts: Vec<String> = res
                .params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            parts.sort();
            let p_str = parts.join(" ");
            let ret_color = return_color(res.result.metrics.total_return_pct);

            let (col_label, col_value) = metric_score_display(opt_metric, &res.result);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" #{:<3}", rank),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(format!("{:<40}", p_str), Style::default().fg(Color::White)),
                Span::styled(
                    format!(" {}: ", col_label),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(col_value, Style::default().fg(Color::Cyan)),
                Span::styled(" Ret: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:+.2}%", res.result.metrics.total_return_pct),
                    Style::default().fg(ret_color),
                ),
            ]))
        })
        .collect();

    let list = List::new(visible).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(format!(
                " Ranked Results ({}) - ↑/↓ scroll ",
                opt.results.len()
            )),
    );
    f.render_widget(list, split[1]);
}

// Walk-forward tab: aggregate metrics + per-window IS/OOS results

fn render_walk_forward_results(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let Some(ref wf) = app.result.walk_forward else {
        let msg = Paragraph::new("No walk-forward data.").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Walk-Forward "),
        );
        f.render_widget(msg, area);
        return;
    };

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(area);

    // Aggregate summary
    let m = &wf.aggregate_metrics;
    let summary_lines = vec![
        Line::from(""),
        metric_line("Windows", &wf.windows.len().to_string()),
        metric_line(
            "OOS Consistency",
            &format!("{:.1}%", wf.consistency_ratio * 100.0),
        ),
        metric_line("Agg Return", &format!("{:+.2}%", m.total_return_pct)),
        metric_line("Agg Sharpe", &format_ratio(m.sharpe_ratio)),
        metric_line("Agg Max DD", &format!("{:.2}%", m.max_drawdown_pct * 100.0)),
    ];

    let summary = Paragraph::new(summary_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Walk-Forward Summary "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(summary, split[0]);

    // Per-window list
    let visible: Vec<ListItem> = wf
        .windows
        .iter()
        .skip(app.scroll)
        .take(split[1].height.saturating_sub(2) as usize)
        .map(|w| {
            let is_ret = w.in_sample.metrics.total_return_pct;
            let oos_ret = w.out_of_sample.metrics.total_return_pct;
            let mut parts: Vec<String> = w
                .optimized_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            parts.sort();
            let params_str = parts.join(" ");

            let oos_sharpe = w.out_of_sample.metrics.sharpe_ratio;
            let oos_dd = w.out_of_sample.metrics.max_drawdown_pct * 100.0;
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" W{:<2} ", w.window + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("IS:", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {:+.2}%", is_ret),
                    Style::default().fg(return_color(is_ret)),
                ),
                Span::styled("  OOS:", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {:+.2}%", oos_ret),
                    Style::default().fg(return_color(oos_ret)),
                ),
                Span::styled("  Sh:", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {}", format_ratio(oos_sharpe)),
                    Style::default().fg(if oos_sharpe >= 1.0 {
                        Color::Green
                    } else if oos_sharpe >= 0.0 {
                        Color::Yellow
                    } else {
                        Color::Red
                    }),
                ),
                Span::styled("  DD:", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {:.1}%", oos_dd),
                    Style::default().fg(if oos_dd <= 10.0 {
                        Color::Green
                    } else if oos_dd <= 20.0 {
                        Color::Yellow
                    } else {
                        Color::Red
                    }),
                ),
                Span::styled(
                    format!("  {}", params_str),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(visible).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(format!(" Windows ({}) - ↑/↓ scroll ", wf.windows.len())),
    );
    f.render_widget(list, split[1]);
}

// CSV export

fn export_trades_csv(result: &BacktestResult) -> Result<PathBuf, String> {
    use std::io::Write;

    let filename = format!(
        "backtest_{}_{}.csv",
        result.symbol.to_lowercase(),
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let export_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fq")
        .join("exports");
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;
    let path = export_dir.join(&filename);

    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;

    writeln!(file, "side,entry_date,exit_date,entry_price,exit_price,quantity,pnl,return_pct,commission,dividend_income")
        .map_err(|e| e.to_string())?;

    for trade in &result.trades {
        let side = if trade.is_long() { "LONG" } else { "SHORT" };
        writeln!(
            file,
            "{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
            side,
            format_timestamp(trade.entry_timestamp),
            format_timestamp(trade.exit_timestamp),
            trade.entry_price,
            trade.exit_price,
            trade.quantity,
            trade.pnl,
            trade.return_pct,
            trade.commission,
            trade.dividend_income,
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(path)
}

fn export_portfolio_csv(portfolio: &PortfolioResult) -> Result<PathBuf, String> {
    use std::io::Write;

    let filename = format!(
        "portfolio_backtest_{}.csv",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let export_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fq")
        .join("exports");
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;
    let path = export_dir.join(&filename);

    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;

    writeln!(
        file,
        "symbol,side,entry_date,exit_date,entry_price,exit_price,quantity,pnl,return_pct,commission,dividend_income"
    )
    .map_err(|e| e.to_string())?;

    let mut symbols: Vec<&str> = portfolio.symbols.keys().map(|s| s.as_str()).collect();
    symbols.sort();

    for sym in symbols {
        if let Some(result) = portfolio.symbols.get(sym) {
            for trade in &result.trades {
                let side = if trade.is_long() { "LONG" } else { "SHORT" };
                writeln!(
                    file,
                    "{},{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
                    sym,
                    side,
                    format_timestamp(trade.entry_timestamp),
                    format_timestamp(trade.exit_timestamp),
                    trade.entry_price,
                    trade.exit_price,
                    trade.quantity,
                    trade.pnl,
                    trade.return_pct,
                    trade.commission,
                    trade.dividend_income,
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(path)
}

// Helper functions

/// Linear interpolation percentile on a pre-sorted slice.
/// `p` is in [0.0, 1.0]. Uses the "inclusive" (C=1) method.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    debug_assert!(
        (0.0..=1.0).contains(&p),
        "percentile p must be in [0, 1], got {p}"
    );
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    if n == 1 {
        return sorted[0];
    }
    let rank = p * (n - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = lo + 1;
    let frac = rank - lo as f64;
    if hi >= n {
        sorted[n - 1]
    } else {
        sorted[lo] + frac * (sorted[hi] - sorted[lo])
    }
}

fn metric_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:<18}", label),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}

/// Returns `true` when consecutive equity curve bars are less than one trading day apart,
/// which indicates an intraday backtest (1m, 5m, 15m, 30m, 1h intervals).
fn is_intraday(equity_curve: &[finance_query::backtesting::EquityPoint]) -> bool {
    const ONE_DAY_SECS: i64 = 86_400;
    equity_curve
        .windows(2)
        .any(|w| (w[1].timestamp - w[0].timestamp).abs() < ONE_DAY_SECS)
}

fn format_timestamp(ts: i64) -> String {
    use chrono::DateTime;
    if let Some(dt) = DateTime::from_timestamp(ts, 0) {
        dt.format("%Y-%m-%d").to_string()
    } else {
        ts.to_string()
    }
}

fn format_timestamp_with_precision(ts: i64, intraday: bool) -> String {
    use chrono::DateTime;
    if let Some(dt) = DateTime::from_timestamp(ts, 0) {
        if intraday {
            dt.format("%Y-%m-%d %H:%M").to_string()
        } else {
            dt.format("%Y-%m-%d").to_string()
        }
    } else {
        ts.to_string()
    }
}

fn format_duration_secs(secs: f64) -> String {
    if secs <= 0.0 {
        return "0s".to_string();
    }
    let days = secs / 86400.0;
    if days >= 1.0 {
        format!("{:.1}d", days)
    } else {
        let hours = secs / 3600.0;
        if hours >= 1.0 {
            format!("{:.1}h", hours)
        } else {
            format!("{:.0}m", secs / 60.0)
        }
    }
}

fn format_ratio(ratio: f64) -> String {
    if ratio.is_nan() {
        "-".to_string()
    } else if ratio == f64::MAX {
        "∞".to_string()
    } else if ratio.is_infinite() {
        if ratio.is_sign_negative() {
            "-∞".to_string()
        } else {
            "∞".to_string()
        }
    } else {
        format!("{:.2}", ratio)
    }
}

fn format_signed_pct(value: f64) -> String {
    if value.is_nan() {
        "-".to_string()
    } else if value == f64::MAX {
        "+∞%".to_string()
    } else if value.is_infinite() {
        if value.is_sign_negative() {
            "-∞%".to_string()
        } else {
            "+∞%".to_string()
        }
    } else {
        format!("{:+.2}%", value)
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

/// Returns the short label and formatted value for the selected optimize metric.
fn metric_score_display(metric: OptimizeMetric, result: &BacktestResult) -> (String, String) {
    match metric {
        OptimizeMetric::SharpeRatio => (
            "Sharpe".to_string(),
            format_ratio(result.metrics.sharpe_ratio),
        ),
        OptimizeMetric::TotalReturn => (
            "Return".to_string(),
            format!("{:+.2}%", result.metrics.total_return_pct),
        ),
        OptimizeMetric::SortinoRatio => (
            "Sortino".to_string(),
            format_ratio(result.metrics.sortino_ratio),
        ),
        OptimizeMetric::CalmarRatio => (
            "Calmar".to_string(),
            format_ratio(result.metrics.calmar_ratio),
        ),
        OptimizeMetric::ProfitFactor => (
            "Prof.Factor".to_string(),
            format_ratio(result.metrics.profit_factor),
        ),
        OptimizeMetric::WinRate => (
            "Win Rate".to_string(),
            format!("{:.1}%", result.metrics.win_rate * 100.0),
        ),
        OptimizeMetric::MinDrawdown => (
            "Drawdown".to_string(),
            format!("{:.2}%", result.metrics.max_drawdown_pct * 100.0),
        ),
        _ => ("Score".to_string(), "N/A".to_string()),
    }
}

fn max_drawdown_dollars(equities: impl IntoIterator<Item = f64>) -> f64 {
    let mut peak = f64::NEG_INFINITY;
    let mut max_dd: f64 = 0.0;
    let mut has_any = false;

    for equity in equities {
        has_any = true;
        peak = peak.max(equity);
        max_dd = max_dd.max((peak - equity).max(0.0));
    }

    if has_any { max_dd } else { 0.0 }
}

// ── Portfolio tab ─────────────────────────────────────────────────────────────

fn render_portfolio(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    use ratatui::widgets::{Cell, Row, Table};

    let Some(portfolio) = &app.result.portfolio else {
        render_results_overview(f, app, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    render_portfolio_metrics(f, portfolio, chunks[0]);
    render_portfolio_symbols_table(f, app, portfolio, chunks[1]);

    // Local helpers capture the outer widgets via `use` above.
    fn render_portfolio_metrics(
        f: &mut Frame,
        portfolio: &PortfolioResult,
        area: ratatui::layout::Rect,
    ) {
        let m = &portfolio.portfolio_metrics;
        let profit = portfolio.final_equity - portfolio.initial_capital;
        let profit_color = if profit >= 0.0 {
            Color::Green
        } else {
            Color::Red
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("Portfolio", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(
                    "  Initial: ${:.2}  Final: ${:.2}  ",
                    portfolio.initial_capital, portfolio.final_equity,
                )),
                Span::styled(
                    format!("{:+.2} ({:+.2}%)", profit, m.total_return_pct),
                    Style::default()
                        .fg(profit_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(format!(
                "  Annualized: {:+.2}%   Sharpe: {:.3}   Sortino: {:.3}   Calmar: {:.3}",
                m.annualized_return_pct, m.sharpe_ratio, m.sortino_ratio, m.calmar_ratio,
            )),
            Line::from(format!(
                "  Max DD: {:.2}%   Win Rate: {:.1}%   Profit Factor: {:.2}   Trades: {}",
                m.max_drawdown_pct * 100.0,
                m.win_rate * 100.0,
                m.profit_factor,
                m.total_trades,
            )),
            Line::from(format!(
                "  Expectancy: {:.3}   SQN: {:.3}   Kelly: {:.1}%   Time in Market: {:.1}%",
                m.expectancy,
                m.sqn,
                m.kelly_criterion * 100.0,
                m.time_in_market_pct * 100.0,
            )),
        ];

        let para = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Portfolio Summary"),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(para, area);
    }

    fn render_portfolio_symbols_table(
        f: &mut Frame,
        app: &ResultsApp,
        portfolio: &PortfolioResult,
        area: ratatui::layout::Rect,
    ) {
        let header = Row::new(vec![
            Cell::from("Symbol").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Return %").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Ann. %").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Sharpe").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Max DD %").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Win Rate").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Trades").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Profit Factor").style(Style::default().add_modifier(Modifier::BOLD)),
        ])
        .style(Style::default().fg(Color::Cyan))
        .height(1);

        // Sort symbols alphabetically for stable display order.
        let mut symbols: Vec<&String> = portfolio.symbols.keys().collect();
        symbols.sort();

        let row_count = symbols.len();
        let rows: Vec<Row> = symbols
            .iter()
            .enumerate()
            .map(|(i, sym)| {
                let r = &portfolio.symbols[*sym];
                let m = &r.metrics;
                let ret_color = if m.total_return_pct >= 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };
                let bg = if i % 2 == 0 {
                    Color::Reset
                } else {
                    Color::DarkGray
                };
                Row::new(vec![
                    Cell::from(sym.as_str()),
                    Cell::from(format!("{:+.2}%", m.total_return_pct))
                        .style(Style::default().fg(ret_color)),
                    Cell::from(format!("{:+.2}%", m.annualized_return_pct)),
                    Cell::from(format_ratio(m.sharpe_ratio)),
                    Cell::from(format!("{:.2}%", m.max_drawdown_pct * 100.0)),
                    Cell::from(format!("{:.1}%", m.win_rate * 100.0)),
                    Cell::from(format!("{}", m.total_trades)),
                    Cell::from(format_ratio(m.profit_factor)),
                ])
                .style(Style::default().bg(bg))
                .height(1)
            })
            .collect();

        let visible: Vec<Row> = rows
            .into_iter()
            .skip(app.scroll)
            .take(area.height.saturating_sub(3) as usize)
            .collect();

        let table = Table::new(
            visible,
            [
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(9),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(8),
                Constraint::Min(0),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Symbols ({}) — ↑↓ scroll", row_count)),
        );
        f.render_widget(table, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_ratio_handles_max_sentinel() {
        assert_eq!(format_ratio(f64::MAX), "∞");
    }

    #[test]
    fn format_signed_pct_handles_max_sentinel() {
        assert_eq!(format_signed_pct(f64::MAX), "+∞%");
    }

    #[test]
    fn max_drawdown_dollars_uses_peak_to_trough() {
        let equities = vec![10_000.0, 12_000.0, 9_000.0, 11_000.0];
        assert!((max_drawdown_dollars(equities.into_iter()) - 3_000.0).abs() < 1e-9);
    }
}

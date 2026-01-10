use crate::alerts::{Alert, AlertStore};
use crate::dashboard::storage::{DashboardStorage, Watchlist};
use crate::portfolio::{Portfolio, PortfolioStorage};
use finance_query::{
    Quote, ScreenerQuote, ScreenerType, Sector, SectorType, Spark, streaming::PriceUpdate,
};
use std::collections::HashMap;
use tokio::time::{Duration, Interval as TokioInterval, interval};

pub const REFRESH_INTERVAL_SECS: u64 = 5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Watchlist,
    Charts,
    News,
    Lookup,
    Screeners,
    Sectors,
    Portfolio,
    Alerts,
}

impl Tab {
    pub fn next(&self) -> Self {
        match self {
            Tab::Watchlist => Tab::Charts,
            Tab::Charts => Tab::News,
            Tab::News => Tab::Lookup,
            Tab::Lookup => Tab::Screeners,
            Tab::Screeners => Tab::Sectors,
            Tab::Sectors => Tab::Portfolio,
            Tab::Portfolio => Tab::Alerts,
            Tab::Alerts => Tab::Watchlist,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Tab::Watchlist => Tab::Alerts,
            Tab::Charts => Tab::Watchlist,
            Tab::News => Tab::Charts,
            Tab::Lookup => Tab::News,
            Tab::Screeners => Tab::Lookup,
            Tab::Sectors => Tab::Screeners,
            Tab::Portfolio => Tab::Sectors,
            Tab::Alerts => Tab::Portfolio,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Tab::Watchlist => "Watchlist",
            Tab::Charts => "Charts",
            Tab::News => "News",
            Tab::Lookup => "Lookup",
            Tab::Screeners => "Screeners",
            Tab::Sectors => "Sectors",
            Tab::Portfolio => "Portfolio",
            Tab::Alerts => "Alerts",
        }
    }
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    AddSymbol,
    AddPosition,
    AddAlert,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScreenerCategory {
    Gainers,
    Losers,
    MostActive,
    MostShorted,
    SmallCapGainers,
    GrowthTech,
    UndervaluedGrowth,
    UndervaluedLargeCap,
    AggressiveSmallCaps,
}

impl ScreenerCategory {
    pub const ALL: [ScreenerCategory; 9] = [
        ScreenerCategory::Gainers,
        ScreenerCategory::Losers,
        ScreenerCategory::MostActive,
        ScreenerCategory::MostShorted,
        ScreenerCategory::SmallCapGainers,
        ScreenerCategory::GrowthTech,
        ScreenerCategory::UndervaluedGrowth,
        ScreenerCategory::UndervaluedLargeCap,
        ScreenerCategory::AggressiveSmallCaps,
    ];

    pub fn next(&self) -> Self {
        let idx = Self::ALL.iter().position(|c| c == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn prev(&self) -> Self {
        let idx = Self::ALL.iter().position(|c| c == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn title(&self) -> &str {
        match self {
            ScreenerCategory::Gainers => "Gainers",
            ScreenerCategory::Losers => "Losers",
            ScreenerCategory::MostActive => "Active",
            ScreenerCategory::MostShorted => "Shorted",
            ScreenerCategory::SmallCapGainers => "SmallCap↑",
            ScreenerCategory::GrowthTech => "Tech",
            ScreenerCategory::UndervaluedGrowth => "Value",
            ScreenerCategory::UndervaluedLargeCap => "LargeCap",
            ScreenerCategory::AggressiveSmallCaps => "Aggr.Small",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            ScreenerCategory::Gainers => "Top gaining stocks today",
            ScreenerCategory::Losers => "Top losing stocks today",
            ScreenerCategory::MostActive => "Most actively traded by volume",
            ScreenerCategory::MostShorted => "Highest short interest",
            ScreenerCategory::SmallCapGainers => "Small cap stocks gaining today",
            ScreenerCategory::GrowthTech => "Tech stocks with 25%+ revenue growth",
            ScreenerCategory::UndervaluedGrowth => "Low P/E, high EPS growth",
            ScreenerCategory::UndervaluedLargeCap => "Large caps with low P/E",
            ScreenerCategory::AggressiveSmallCaps => "Small caps with high EPS growth",
        }
    }

    pub fn screener_type(&self) -> ScreenerType {
        match self {
            ScreenerCategory::Gainers => ScreenerType::DayGainers,
            ScreenerCategory::Losers => ScreenerType::DayLosers,
            ScreenerCategory::MostActive => ScreenerType::MostActives,
            ScreenerCategory::MostShorted => ScreenerType::MostShortedStocks,
            ScreenerCategory::SmallCapGainers => ScreenerType::SmallCapGainers,
            ScreenerCategory::GrowthTech => ScreenerType::GrowthTechnologyStocks,
            ScreenerCategory::UndervaluedGrowth => ScreenerType::UndervaluedGrowthStocks,
            ScreenerCategory::UndervaluedLargeCap => ScreenerType::UndervaluedLargeCaps,
            ScreenerCategory::AggressiveSmallCaps => ScreenerType::AggressiveSmallCaps,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusPane {
    Left,
    Right,
}

pub struct App {
    pub storage: DashboardStorage,
    pub current_watchlist: Watchlist,
    pub selected_index: usize,
    pub active_tab: Tab,
    pub focus_pane: FocusPane,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub price_updates: HashMap<String, PriceUpdate>,
    pub quotes: HashMap<String, Quote>,
    pub chart_data: Option<Vec<(f64, f64)>>,
    pub news_items: Vec<finance_query::News>,
    pub selected_chart_range_idx: usize,
    pub selected_news_idx: usize,
    pub search_query: String,
    pub search_results: Vec<finance_query::SearchQuote>,
    pub selected_search_idx: usize,
    pub is_searching: bool,
    pub screener_category: ScreenerCategory,
    pub screener_data: Vec<ScreenerQuote>,
    pub selected_screener_idx: usize,
    pub is_loading_screeners: bool,
    pub alert_store: AlertStore,
    pub alerts: Vec<Alert>,
    pub status_message: String,
    pub last_update: Option<chrono::DateTime<chrono::Local>>,
    pub refresh_interval: TokioInterval,
    pub should_quit: bool,
    // Portfolio fields
    pub portfolio_storage: PortfolioStorage,
    pub portfolio: Portfolio,
    pub portfolio_prices: HashMap<String, f64>,
    pub selected_portfolio_idx: usize,
    pub add_form_field: usize,
    pub add_form_symbol: String,
    pub add_form_shares: String,
    pub add_form_cost: String,
    // Alert form fields
    pub selected_alert_idx: usize,
    pub alert_form_symbol: String,
    pub alert_form_type_idx: usize, // Index into AlertType::all()
    pub alert_form_threshold: String,
    pub alert_form_field: usize, // 0=symbol, 1=type, 2=threshold
    // Sectors tab fields
    pub sectors_data: HashMap<SectorType, Sector>,
    pub sectors_selected_idx: usize,
    pub sectors_selected_industry: usize,
    pub sectors_view_mode: SectorsViewMode,
    pub is_loading_sectors: bool,
    // News tab fields
    pub news_symbol: Option<String>, // None = general news, Some(symbol) = symbol-specific news
    pub is_loading_news: bool,
    // Detailed quote loading (for Growth & Ownership data)
    pub is_loading_detailed_quote: bool,
    pub loading_detailed_symbol: Option<String>,
    // Scroll positions for detail panels (Trading, Fundamentals, Growth & Ownership)
    pub detail_scroll: [u16; 3],
    // Sparkline data for watchlist symbols
    pub sparklines: HashMap<String, Spark>,
}

/// Sectors view mode - sectors overview or drill-down to industries
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SectorsViewMode {
    #[default]
    Overview,
    Industries,
}

impl App {
    pub async fn new() -> anyhow::Result<Self> {
        let storage = DashboardStorage::new()?;
        let watchlists = storage.get_watchlists()?;
        let current_watchlist = watchlists
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No watchlists found"))?;

        let refresh_interval = interval(Duration::from_secs(REFRESH_INTERVAL_SECS));

        let alert_store = AlertStore::new()?;
        let alerts = alert_store.get_alerts()?;

        let portfolio_storage = PortfolioStorage::new()?;
        let portfolio = portfolio_storage.load()?;

        Ok(Self {
            storage,
            current_watchlist,
            selected_index: 0,
            active_tab: Tab::Watchlist,
            focus_pane: FocusPane::Left,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            price_updates: HashMap::new(),
            quotes: HashMap::new(),
            chart_data: None,
            news_items: Vec::new(),
            selected_chart_range_idx: 2,
            selected_news_idx: 0,
            search_query: String::new(),
            search_results: Vec::new(),
            selected_search_idx: 0,
            is_searching: false,
            screener_category: ScreenerCategory::Gainers,
            screener_data: Vec::new(),
            selected_screener_idx: 0,
            is_loading_screeners: false,
            alert_store,
            alerts,
            status_message: "Welcome to Finance Query Dashboard".to_string(),
            last_update: None,
            refresh_interval,
            should_quit: false,
            portfolio_storage,
            portfolio,
            portfolio_prices: HashMap::new(),
            selected_portfolio_idx: 0,
            add_form_field: 0,
            add_form_symbol: String::new(),
            add_form_shares: String::new(),
            add_form_cost: String::new(),
            selected_alert_idx: 0,
            alert_form_symbol: String::new(),
            alert_form_type_idx: 0,
            alert_form_threshold: String::new(),
            alert_form_field: 0,
            sectors_data: HashMap::new(),
            sectors_selected_idx: 0,
            sectors_selected_industry: 0,
            sectors_view_mode: SectorsViewMode::default(),
            is_loading_sectors: false,
            news_symbol: None,
            is_loading_news: false,
            is_loading_detailed_quote: false,
            loading_detailed_symbol: None,
            detail_scroll: [0, 0, 0],
            sparklines: HashMap::new(),
        })
    }

    pub fn selected_symbol(&self) -> Option<&String> {
        self.current_watchlist.symbols.get(self.selected_index)
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.current_watchlist.symbols.len();
        if len == 0 {
            self.selected_index = 0;
            return;
        }

        let new_index = (self.selected_index as isize + delta).rem_euclid(len as isize);
        self.selected_index = new_index as usize;
        // Reset scroll positions when changing symbols
        self.detail_scroll = [0, 0, 0];
    }

    pub fn check_alerts(&mut self) {
        use crate::alerts::send_alert_notification;

        let triggered: Vec<(i64, String, Option<f64>)> = self
            .alerts
            .iter()
            .filter(|alert| {
                if !alert.enabled {
                    return false;
                }

                // Skip alerts that have already been triggered (avoid repeated notifications)
                if alert.last_triggered.is_some() {
                    return false;
                }

                if let Some(quote) = self.quotes.get(&alert.symbol) {
                    alert.check(quote)
                } else {
                    false
                }
            })
            .map(|alert| {
                let current_value = self
                    .quotes
                    .get(&alert.symbol)
                    .and_then(|q| alert.get_current_value(q));
                (alert.id, alert.symbol.clone(), current_value)
            })
            .collect();

        for (id, symbol, current_value) in &triggered {
            if let Some(alert) = self.alerts.iter().find(|a| a.id == *id) {
                let _ = self.alert_store.mark_triggered(*id);

                // Send desktop notification
                send_alert_notification(alert, *current_value);

                self.status_message = format!(
                    "ALERT: {} {} {}",
                    symbol,
                    alert.alert_type.display(),
                    alert.alert_type.format_threshold(alert.threshold)
                );
            }
        }

        // Reload alerts to get updated trigger status
        if let Ok(updated_alerts) = self.alert_store.get_alerts() {
            self.alerts = updated_alerts;
        }
    }

    // Portfolio helper methods
    pub fn add_portfolio_position(&mut self) {
        use crate::portfolio::Position;

        let shares = self.add_form_shares.parse::<f64>();
        let cost = self.add_form_cost.parse::<f64>();

        if self.add_form_symbol.is_empty() {
            self.status_message = "Error: Symbol cannot be empty".to_string();
            return;
        }

        match (shares, cost) {
            (Ok(shares), Ok(cost)) if shares > 0.0 && cost > 0.0 => {
                let position = Position::new(self.add_form_symbol.clone(), shares, cost);
                self.portfolio.add_position(position);

                if let Err(e) = self.portfolio_storage.save(&self.portfolio) {
                    self.status_message = format!("Error saving: {}", e);
                } else {
                    self.status_message = format!("Added {} to portfolio", self.add_form_symbol);
                    self.add_form_symbol.clear();
                    self.add_form_shares.clear();
                    self.add_form_cost.clear();
                    self.add_form_field = 0;
                    self.input_mode = InputMode::Normal;
                }
            }
            _ => {
                self.status_message = "Error: Invalid shares or cost".to_string();
            }
        }
    }

    pub fn delete_portfolio_position(&mut self) {
        if self.portfolio.is_empty() {
            return;
        }

        if self.selected_portfolio_idx < self.portfolio.positions.len() {
            let symbol = self.portfolio.positions[self.selected_portfolio_idx]
                .symbol
                .clone();
            self.portfolio.remove_position(&symbol);

            if let Err(e) = self.portfolio_storage.save(&self.portfolio) {
                self.status_message = format!("Error saving: {}", e);
            } else {
                self.status_message = format!("Deleted {}", symbol);
                if self.selected_portfolio_idx > 0 {
                    self.selected_portfolio_idx -= 1;
                }
            }
        }
    }

    pub fn total_portfolio_value(&self) -> f64 {
        self.portfolio
            .positions
            .iter()
            .map(|p| {
                let current_price = self.portfolio_prices.get(&p.symbol).copied().unwrap_or(0.0);
                p.current_value(current_price)
            })
            .sum()
    }

    pub fn total_portfolio_profit_loss(&self) -> f64 {
        let total_cost = self.portfolio.total_cost();
        let total_value = self.total_portfolio_value();
        total_value - total_cost
    }

    pub fn total_portfolio_profit_loss_percent(&self) -> f64 {
        let total_cost = self.portfolio.total_cost();
        if total_cost == 0.0 {
            return 0.0;
        }
        (self.total_portfolio_profit_loss() / total_cost) * 100.0
    }

    /// Fetch sparkline data for all watchlist symbols
    pub async fn fetch_sparklines(&mut self) -> anyhow::Result<()> {
        use finance_query::{Interval, Tickers, TimeRange};

        if self.current_watchlist.symbols.is_empty() {
            return Ok(());
        }

        let symbols: Vec<&str> = self
            .current_watchlist
            .symbols
            .iter()
            .map(|s| s.as_str())
            .collect();

        let tickers = Tickers::new(symbols).await?;
        let result = tickers
            .spark(Interval::FiveMinutes, TimeRange::OneDay)
            .await?;

        for (symbol, spark) in result.sparks {
            self.sparklines.insert(symbol, spark);
        }

        Ok(())
    }
}

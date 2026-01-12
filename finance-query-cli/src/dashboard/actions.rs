use super::state::{App, SectorsViewMode, Tab};
use anyhow::Result;
use finance_query::{Interval, SectorType, Ticker, Tickers, TimeRange, finance};

impl App {
    pub async fn refresh_details(&mut self) -> Result<()> {
        let Some(symbol) = self.selected_symbol().cloned() else {
            self.chart_data = None;
            return Ok(());
        };

        // Request detailed quote fetch for the selected symbol (includes growth & ownership data)
        // This is handled asynchronously in the event loop to avoid blocking the UI
        if self.active_tab == Tab::Watchlist {
            // Only start fetch if we're not already loading this symbol
            let should_fetch = self.loading_detailed_symbol.as_ref() != Some(&symbol);
            if should_fetch && !self.is_loading_detailed_quote {
                self.is_loading_detailed_quote = true;
                self.loading_detailed_symbol = Some(symbol.clone());
            }
        }

        if self.active_tab == Tab::Charts {
            // Use the selected range with appropriate interval
            let range_options: [(&str, TimeRange, Interval); 8] = [
                ("1D", TimeRange::OneDay, Interval::FiveMinutes),
                ("5D", TimeRange::FiveDays, Interval::FifteenMinutes),
                ("1M", TimeRange::OneMonth, Interval::OneDay),
                ("6M", TimeRange::SixMonths, Interval::OneDay),
                ("YTD", TimeRange::YearToDate, Interval::OneDay),
                ("1Y", TimeRange::OneYear, Interval::OneDay),
                ("5Y", TimeRange::FiveYears, Interval::OneWeek),
                ("Max", TimeRange::Max, Interval::OneWeek),
            ];
            let (label, range, interval) = range_options[self.selected_chart_range_idx];

            self.status_message = format!("Fetching chart for {} ({})...", symbol, label);
            match Ticker::new(&symbol).await {
                Ok(ticker) => match ticker.chart(interval, range).await {
                    Ok(chart) => {
                        let data: Vec<(f64, f64)> = chart
                            .candles
                            .iter()
                            .map(|c| (c.timestamp as f64, c.close))
                            .collect();
                        let count = data.len();
                        self.chart_data = Some(data);
                        self.status_message =
                            format!("Chart loaded for {} ({}, {} points)", symbol, label, count);
                    }
                    Err(e) => {
                        self.chart_data = None;
                        self.status_message = format!("Chart error for {}: {}", symbol, e);
                    }
                },
                Err(e) => {
                    self.chart_data = None;
                    self.status_message = format!("Failed to create ticker for {}: {}", symbol, e);
                }
            }
        }

        Ok(())
    }

    pub async fn add_symbol(&mut self) -> Result<()> {
        let symbol = self.input_buffer.trim().to_uppercase();
        if symbol.is_empty() {
            return Ok(());
        }

        self.storage
            .add_symbol_to_watchlist(self.current_watchlist.id, &symbol)?;
        self.current_watchlist = self
            .storage
            .get_watchlist(self.current_watchlist.id)?
            .ok_or_else(|| anyhow::anyhow!("Watchlist not found"))?;

        self.status_message = format!("Fetching quote for {}...", symbol);
        match Ticker::new(&symbol).await {
            Ok(ticker) => match ticker.quote(false).await {
                Ok(quote) => {
                    self.quotes.insert(symbol.clone(), quote);
                    self.status_message = format!("Added {} to watchlist", symbol);
                }
                Err(e) => {
                    self.status_message = format!("Added {} (quote error: {})", symbol, e);
                }
            },
            Err(e) => {
                self.status_message = format!("Added {} (ticker error: {})", symbol, e);
            }
        }

        self.input_buffer.clear();
        Ok(())
    }

    pub async fn perform_search(&mut self) -> Result<()> {
        if self.search_query.trim().is_empty() {
            self.search_results.clear();
            self.selected_search_idx = 0;
            return Ok(());
        }

        self.is_searching = true;
        self.status_message = format!("Searching for '{}'...", self.search_query);

        let options = finance_query::SearchOptions::new()
            .quotes_count(20)
            .enable_logo_url(true);

        match finance_query::finance::search(&self.search_query, &options).await {
            Ok(results) => {
                self.search_results = results.quotes.0;
                self.selected_search_idx = 0;
                self.status_message = format!("Found {} results", self.search_results.len());
            }
            Err(e) => {
                self.search_results.clear();
                self.selected_search_idx = 0;
                self.status_message = format!("Search error: {}", e);
            }
        }

        self.is_searching = false;
        Ok(())
    }

    pub async fn add_selected_search_result(&mut self) -> Result<()> {
        if self.selected_search_idx >= self.search_results.len() {
            return Ok(());
        }

        let result = &self.search_results[self.selected_search_idx];
        let symbol = result.symbol.clone();

        self.storage
            .add_symbol_to_watchlist(self.current_watchlist.id, &symbol)?;
        self.current_watchlist = self
            .storage
            .get_watchlist(self.current_watchlist.id)?
            .ok_or_else(|| anyhow::anyhow!("Watchlist not found"))?;

        self.status_message = format!("Added {} to watchlist", symbol);

        match Ticker::new(&symbol).await {
            Ok(ticker) => match ticker.quote(false).await {
                Ok(quote) => {
                    self.quotes.insert(symbol.clone(), quote);
                    self.status_message = format!("Added {} to watchlist", symbol);
                }
                Err(e) => {
                    self.status_message = format!("Added {} (quote error: {})", symbol, e);
                }
            },
            Err(e) => {
                self.status_message = format!("Added {} (ticker error: {})", symbol, e);
            }
        }

        Ok(())
    }

    pub async fn fetch_screeners(&mut self) -> Result<()> {
        self.is_loading_screeners = true;
        self.status_message = format!("Fetching {}...", self.screener_category.title());

        match finance::screener(self.screener_category.screener_type(), 25).await {
            Ok(results) => {
                self.screener_data = results.quotes;
                self.selected_screener_idx = 0;
                self.status_message = format!(
                    "{}: {} stocks loaded",
                    self.screener_category.title(),
                    self.screener_data.len()
                );
            }
            Err(e) => {
                self.screener_data.clear();
                self.selected_screener_idx = 0;
                self.status_message =
                    format!("Error loading {}: {}", self.screener_category.title(), e);
            }
        }

        self.is_loading_screeners = false;
        Ok(())
    }

    pub async fn add_screener_to_watchlist(&mut self) -> Result<()> {
        if self.selected_screener_idx >= self.screener_data.len() {
            return Ok(());
        }

        let item = &self.screener_data[self.selected_screener_idx];
        let symbol = item.symbol.clone();

        self.storage
            .add_symbol_to_watchlist(self.current_watchlist.id, &symbol)?;
        self.current_watchlist = self
            .storage
            .get_watchlist(self.current_watchlist.id)?
            .ok_or_else(|| anyhow::anyhow!("Watchlist not found"))?;

        self.status_message = format!("Added {} to watchlist", symbol);

        match Ticker::new(&symbol).await {
            Ok(ticker) => match ticker.quote(false).await {
                Ok(quote) => {
                    self.quotes.insert(symbol.clone(), quote);
                    self.status_message = format!("Added {} to watchlist", symbol);
                }
                Err(e) => {
                    self.status_message = format!("Added {} (quote error: {})", symbol, e);
                }
            },
            Err(e) => {
                self.status_message = format!("Added {} (ticker error: {})", symbol, e);
            }
        }

        Ok(())
    }

    pub async fn fetch_all_quotes(&mut self) -> Result<()> {
        if self.current_watchlist.symbols.is_empty() {
            return Ok(());
        }

        self.status_message = "Fetching quotes...".to_string();

        match Tickers::new(&self.current_watchlist.symbols).await {
            Ok(tickers) => match tickers.quotes(false).await {
                Ok(response) => {
                    for (symbol, quote) in response.quotes {
                        self.quotes.insert(symbol, quote);
                    }
                    self.last_update = Some(chrono::Local::now());
                    self.status_message = format!("Quotes loaded ({} symbols)", self.quotes.len());
                }
                Err(e) => {
                    self.status_message = format!("Error fetching quotes: {}", e);
                }
            },
            Err(e) => {
                self.status_message = format!("Error creating tickers: {}", e);
            }
        }

        Ok(())
    }

    pub async fn remove_selected_symbol(&mut self) -> Result<()> {
        let Some(symbol) = self.selected_symbol().cloned() else {
            return Ok(());
        };

        self.storage
            .remove_symbol_from_watchlist(self.current_watchlist.id, &symbol)?;
        self.current_watchlist = self
            .storage
            .get_watchlist(self.current_watchlist.id)?
            .ok_or_else(|| anyhow::anyhow!("Watchlist not found"))?;

        if self.selected_index >= self.current_watchlist.symbols.len() && self.selected_index > 0 {
            self.selected_index -= 1;
        }

        self.status_message = format!("Removed {} from watchlist", symbol);
        Ok(())
    }

    pub async fn refresh_portfolio_prices(&mut self) -> Result<()> {
        if self.portfolio.is_empty() {
            return Ok(());
        }

        let symbols = self.portfolio.symbols();
        match Tickers::new(&symbols).await {
            Ok(tickers) => match tickers.quotes(false).await {
                Ok(response) => {
                    self.portfolio_prices.clear();
                    for (symbol, quote) in &response.quotes {
                        if let Some(price) = quote.regular_market_price.as_ref().and_then(|v| v.raw)
                        {
                            self.portfolio_prices.insert(symbol.clone(), price);
                        }
                    }
                    self.last_update = Some(chrono::Local::now());
                }
                Err(e) => {
                    self.status_message = format!("Error fetching portfolio prices: {}", e);
                }
            },
            Err(e) => {
                self.status_message = format!("Error creating tickers: {}", e);
            }
        }
        Ok(())
    }

    pub async fn fetch_sectors_data(&mut self) -> Result<()> {
        if self.is_loading_sectors {
            return Ok(());
        }

        self.is_loading_sectors = true;
        self.status_message = "Loading sector data...".to_string();

        let sectors = SectorType::all();
        let mut loaded_count = 0;

        for sector_type in sectors {
            match finance::sector(*sector_type).await {
                Ok(sector) => {
                    self.sectors_data.insert(*sector_type, sector);
                    loaded_count += 1;
                }
                Err(e) => {
                    self.status_message =
                        format!("Error loading {}: {}", sector_type.display_name(), e);
                }
            }
        }

        self.is_loading_sectors = false;
        self.status_message = format!("Loaded {} sectors", loaded_count);
        self.last_update = Some(chrono::Local::now());
        Ok(())
    }

    pub fn get_sorted_sectors(&self) -> Vec<(SectorType, &finance_query::Sector)> {
        let mut sectors: Vec<_> = self.sectors_data.iter().map(|(k, v)| (*k, v)).collect();

        // Sort by day change percent (descending - best performers first)
        sectors.sort_by(|a, b| {
            let a_change =
                a.1.performance
                    .as_ref()
                    .and_then(|p| p.day_change_percent.as_ref())
                    .and_then(|v| v.raw)
                    .unwrap_or(0.0);
            let b_change =
                b.1.performance
                    .as_ref()
                    .and_then(|p| p.day_change_percent.as_ref())
                    .and_then(|v| v.raw)
                    .unwrap_or(0.0);
            b_change
                .partial_cmp(&a_change)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        sectors
    }

    pub fn selected_sector(&self) -> Option<(SectorType, &finance_query::Sector)> {
        let sectors = self.get_sorted_sectors();
        sectors.get(self.sectors_selected_idx).copied()
    }

    pub fn toggle_sectors_view(&mut self) {
        self.sectors_view_mode = match self.sectors_view_mode {
            SectorsViewMode::Overview => SectorsViewMode::Industries,
            SectorsViewMode::Industries => SectorsViewMode::Overview,
        };
        self.sectors_selected_industry = 0;
    }

    /// Fetch general market news
    pub async fn fetch_general_news(&mut self) -> Result<()> {
        self.status_message = "Fetching market news...".to_string();
        self.is_loading_news = true;

        match finance::news().await {
            Ok(news) => {
                self.news_items = news.into_iter().take(20).collect();
                self.selected_news_idx = 0;
                self.news_symbol = None;
                self.status_message =
                    format!("Market news loaded ({} items)", self.news_items.len());
            }
            Err(e) => {
                self.news_items.clear();
                self.selected_news_idx = 0;
                self.news_symbol = None;
                self.status_message = format!("News error: {}", e);
            }
        }

        self.is_loading_news = false;
        Ok(())
    }

    /// Fetch news for a specific symbol
    pub async fn fetch_symbol_news(&mut self, symbol: &str) -> Result<()> {
        self.status_message = format!("Fetching news for {}...", symbol);
        self.is_loading_news = true;

        match Ticker::new(symbol).await {
            Ok(ticker) => match ticker.news().await {
                Ok(news) => {
                    self.news_items = news.into_iter().take(20).collect();
                    self.selected_news_idx = 0;
                    self.news_symbol = Some(symbol.to_string());
                    self.status_message = format!(
                        "News loaded for {} ({} items)",
                        symbol,
                        self.news_items.len()
                    );
                }
                Err(e) => {
                    self.news_items.clear();
                    self.selected_news_idx = 0;
                    self.status_message = format!("News error for {}: {}", symbol, e);
                }
            },
            Err(e) => {
                self.news_items.clear();
                self.selected_news_idx = 0;
                self.status_message = format!("Failed to create ticker for {}: {}", symbol, e);
            }
        }

        self.is_loading_news = false;
        Ok(())
    }
}

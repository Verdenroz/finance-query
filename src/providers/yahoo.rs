//! Yahoo Finance provider implementation.
//!
//! Wraps YahooClient and delegates conversion to adapter functions
//! to keep this file focused on routing and lifecycle.

use super::{
    CalendarProvider, Capability, ChartProvider, CommoditiesProvider, CorporateProvider,
    DiscoveryProvider, FundamentalsProvider, IndicesProvider, MarketProvider, OptionsProvider,
    ProviderAdapter, ProviderCore, QuoteProvider,
};
use crate::adapters::yahoo::client::{ClientConfig, YahooClient};
use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use std::sync::Arc;

/// Yahoo's capability set, declared rather than derived: `Provider::Yahoo`
/// can't construct a `YahooProvider` for derivation without a live auth
/// handshake. Keep in lockstep with the `as_*` overrides below — pinned by
/// `yahoo_caps_const_matches_declared_capabilities` in `super::tests`.
pub(crate) const CAPS: Capability = Capability::QUOTE
    .union(Capability::CHART)
    .union(Capability::FUNDAMENTALS)
    .union(Capability::CORPORATE)
    .union(Capability::OPTIONS)
    .union(Capability::MARKET)
    .union(Capability::INDICES)
    .union(Capability::COMMODITIES)
    .union(Capability::DISCOVERY)
    .union(Capability::CALENDAR);

pub(crate) struct YahooProvider {
    client: Arc<YahooClient>,
}

impl YahooProvider {
    pub(crate) async fn new(config: &ClientConfig) -> Result<Self> {
        Ok(Self {
            client: crate::adapters::yahoo::session::get_or_auth(config).await?,
        })
    }

    /// Wrap an existing authenticated client — no new auth handshake.
    pub(crate) fn from_client(client: Arc<YahooClient>) -> Self {
        Self { client }
    }

    pub(crate) fn client_arc(&self) -> Arc<YahooClient> {
        Arc::clone(&self.client)
    }
}

impl ProviderCore for YahooProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Yahoo
    }
}

#[async_trait::async_trait]
impl QuoteProvider for YahooProvider {
    async fn fetch_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::quote::QuoteSummaryResponse> {
        crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await
    }

    async fn fetch_quotes_batch(
        &self,
        symbols: &[&str],
    ) -> Result<Vec<(String, crate::models::quote::QuoteSummaryResponse)>> {
        crate::adapters::yahoo::quote::quotes::fetch_quotes_batch(&self.client, symbols).await
    }
}

#[async_trait::async_trait]
impl ChartProvider for YahooProvider {
    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: Interval,
        range: TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        crate::adapters::yahoo::chart::fetch_chart(&self.client, symbol, interval, range).await
    }

    async fn fetch_chart_range(
        &self,
        symbol: &str,
        interval: Interval,
        start: i64,
        end: i64,
    ) -> Result<crate::models::chart::Chart> {
        crate::adapters::yahoo::chart::fetch_chart_with_dates(
            &self.client,
            symbol,
            interval,
            start,
            end,
        )
        .await
    }

    async fn fetch_spark(
        &self,
        symbols: &[&str],
        interval: Interval,
        range: TimeRange,
    ) -> Result<Vec<(String, crate::models::chart::spark::Spark)>> {
        use crate::models::chart::spark::Spark;
        use crate::models::chart::spark::response::SparkResponse;

        let json =
            crate::adapters::yahoo::quote::spark::fetch(&self.client, symbols, interval, range)
                .await?;
        let spark_response = SparkResponse::from_json(json)?;

        let mut out = Vec::new();
        if let Some(results) = spark_response.spark.result {
            for result in &results {
                if let Some(spark) = Spark::from_response(
                    result,
                    Some(interval.as_str().to_string()),
                    Some(range.as_str().to_string()),
                ) {
                    out.push((result.symbol.clone(), spark));
                }
            }
        }
        Ok(out)
    }
}

/// Convert a Yahoo epoch timestamp to a `YYYY-MM-DD` date string.
fn epoch_to_date(ts: i64) -> Option<String> {
    chrono::DateTime::from_timestamp(ts, 0).map(|dt| dt.format("%Y-%m-%d").to_string())
}

#[async_trait::async_trait]
impl FundamentalsProvider for YahooProvider {
    /// Derived from `defaultKeyStatistics` rather than a dedicated endpoint:
    /// Yahoo reports the current and prior-month settlement snapshots (with
    /// `shortRatio` as days-to-cover), so the default keyless route serves
    /// short interest without an API key. Route to Polygon for deeper history.
    async fn fetch_short_interest(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::fundamentals::ShortInterest>> {
        let resp =
            crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await?;
        let stats = resp.default_key_statistics.ok_or_else(|| {
            crate::error::FinanceError::ResponseStructureError {
                field: "defaultKeyStatistics".into(),
                context: format!("no key statistics returned for {symbol}"),
            }
        })?;

        let mut out = Vec::new();
        if let Some(shares) = stats.shares_short.as_ref().and_then(|v| v.raw) {
            out.push(crate::models::fundamentals::ShortInterest {
                settlement_date: stats
                    .date_short_interest
                    .as_ref()
                    .and_then(|v| v.raw)
                    .and_then(epoch_to_date),
                short_interest: Some(shares as f64),
                avg_daily_volume: None,
                days_to_cover: stats.short_ratio.as_ref().and_then(|v| v.raw),
            });
        }
        if let Some(shares) = stats.shares_short_prior_month.as_ref().and_then(|v| v.raw) {
            out.push(crate::models::fundamentals::ShortInterest {
                settlement_date: stats
                    .shares_short_previous_month_date
                    .as_ref()
                    .and_then(|v| v.raw)
                    .and_then(epoch_to_date),
                short_interest: Some(shares as f64),
                avg_daily_volume: None,
                days_to_cover: None,
            });
        }
        Ok(out)
    }

    /// Derived from `defaultKeyStatistics` (`floatShares` /
    /// `sharesOutstanding`) rather than a dedicated endpoint.
    async fn fetch_share_float(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::ShareFloat> {
        let resp =
            crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await?;
        let stats = resp.default_key_statistics.ok_or_else(|| {
            crate::error::FinanceError::ResponseStructureError {
                field: "defaultKeyStatistics".into(),
                context: format!("no key statistics returned for {symbol}"),
            }
        })?;
        Ok(crate::models::fundamentals::ShareFloat {
            symbol: Some(symbol.to_string()),
            float_percent: None,
            float_shares: stats
                .float_shares
                .as_ref()
                .and_then(|v| v.raw)
                .map(|r| r as f64),
            outstanding_shares: stats
                .shares_outstanding
                .as_ref()
                .and_then(|v| v.raw)
                .map(|r| r as f64),
            date: None,
        })
    }

    async fn fetch_financials(
        &self,
        symbol: &str,
        stmt_type: crate::StatementType,
        frequency: crate::Frequency,
    ) -> Result<crate::models::fundamentals::FinancialStatement> {
        let mut stmt =
            crate::adapters::yahoo::fundamentals::fetch(&self.client, symbol, stmt_type, frequency)
                .await?;
        stmt.provider_id = Some(super::Provider::Yahoo);
        Ok(stmt)
    }
}

#[async_trait::async_trait]
impl CorporateProvider for YahooProvider {
    async fn fetch_news(&self, symbol: &str) -> Result<Vec<crate::models::corporate::news::News>> {
        crate::adapters::yahoo::corporate::news::fetch_news(symbol).await
    }

    async fn fetch_events(
        &self,
        symbol: &str,
    ) -> Result<crate::models::chart::events::ChartEvents> {
        crate::adapters::yahoo::chart::fetch_events(&self.client, symbol).await
    }

    async fn fetch_similar_symbols(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::recommendation::SimilarSymbol>> {
        crate::adapters::yahoo::corporate::recommendations::fetch(&self.client, symbol, limit).await
    }

    async fn fetch_earnings_transcript(
        &self,
        symbol: &str,
        quarter: Option<&str>,
        year: Option<i32>,
    ) -> Result<crate::models::corporate::earnings_transcript::EarningsTranscript> {
        let transcript = crate::adapters::yahoo::corporate::transcripts::fetch_for_symbol(
            &self.client,
            symbol,
            quarter,
            year,
        )
        .await?;
        Ok(
            crate::models::corporate::earnings_transcript::EarningsTranscript {
                symbol: Some(symbol.to_string()),
                quarter: Some(transcript.quarter().to_string()),
                year: Some(transcript.year()),
                date: None,
                text: transcript.text().to_string(),
            },
        )
    }
}

#[async_trait::async_trait]
impl MarketProvider for YahooProvider {
    /// Derived from Yahoo's predefined screeners (`day_gainers`,
    /// `day_losers`, `most_actives`) rather than a dedicated movers endpoint,
    /// so movers work on the default keyless route.
    async fn fetch_market_movers(
        &self,
        direction: crate::models::market::performance::MoverDirection,
    ) -> Result<Vec<crate::models::market::performance::MoverQuote>> {
        use crate::constants::screeners::Screener;
        use crate::models::market::performance::MoverDirection;

        let screener = match direction {
            MoverDirection::Gainers => Screener::DayGainers,
            MoverDirection::Losers => Screener::DayLosers,
            MoverDirection::MostActive => Screener::MostActives,
        };
        let results =
            crate::adapters::yahoo::discovery::screeners::fetch(&self.client, screener, 25).await?;
        Ok(screener_quotes_to_movers(results))
    }
}

/// Map screener rows to canonical mover quotes.
fn screener_quotes_to_movers(
    results: crate::models::discovery::screeners::ScreenerResults,
) -> Vec<crate::models::market::performance::MoverQuote> {
    results
        .quotes
        .into_iter()
        .map(|q| crate::models::market::performance::MoverQuote {
            name: Some(q.short_name.clone())
                .filter(|s| !s.is_empty())
                .or(q.long_name),
            symbol: q.symbol,
            price: q.regular_market_price.raw,
            change: q.regular_market_change.raw,
            change_percent: q.regular_market_change_percent.raw,
            exchange: None,
        })
        .collect()
}

#[async_trait::async_trait]
impl IndicesProvider for YahooProvider {
    /// Derived from the generic quote endpoint rather than a dedicated index
    /// endpoint: Yahoo resolves index symbols (`^GSPC`) the same way it
    /// resolves equities, so the default keyless route serves index quotes.
    async fn fetch_indices_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::indices::IndexQuote> {
        let resp =
            crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await?;
        Ok(price_to_index_quote(symbol, resp.price.as_ref()))
    }
}

fn price_to_index_quote(
    symbol: &str,
    price: Option<&crate::models::quote::price::Price>,
) -> crate::models::indices::IndexQuote {
    crate::models::indices::IndexQuote {
        symbol: symbol.to_string(),
        name: price.and_then(|p| p.short_name.clone().or_else(|| p.long_name.clone())),
        price: price.and_then(|p| p.current_price()),
        change: price.and_then(|p| p.day_change()),
        change_percent: price.and_then(|p| p.day_change_percent()),
        timestamp: price.and_then(|p| p.regular_market_time),
    }
}

#[async_trait::async_trait]
impl CommoditiesProvider for YahooProvider {
    /// Derived from the generic quote endpoint: Yahoo resolves commodity
    /// futures symbols (`GC=F`) the same way it resolves equities. `unit`
    /// stays unset, matching FMP's own commodity mapping.
    async fn fetch_commodities_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::commodities::CommodityQuote> {
        let resp =
            crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await?;
        Ok(price_to_commodity_quote(symbol, resp.price.as_ref()))
    }
}

fn price_to_commodity_quote(
    symbol: &str,
    price: Option<&crate::models::quote::price::Price>,
) -> crate::models::commodities::CommodityQuote {
    crate::models::commodities::CommodityQuote {
        symbol: symbol.to_string(),
        name: price.and_then(|p| p.short_name.clone().or_else(|| p.long_name.clone())),
        unit: None,
        price: price.and_then(|p| p.current_price()),
        change: price.and_then(|p| p.day_change()),
        change_percent: price.and_then(|p| p.day_change_percent()),
        timestamp: price.and_then(|p| p.regular_market_time),
    }
}

#[async_trait::async_trait]
impl DiscoveryProvider for YahooProvider {
    /// Derived from Yahoo's fuzzy quote search rather than a dedicated
    /// discovery endpoint — the same call `finance::search` makes, mapped
    /// onto the provider-neutral `SymbolMatch` shape.
    async fn fetch_symbol_search(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>> {
        let options =
            crate::adapters::yahoo::discovery::search::SearchOptions::new().quotes_count(limit);
        let results = self.client.search(query, &options).await?;
        Ok(search_quotes_to_symbol_matches(results.quotes))
    }

    /// Derived from Yahoo's custom equity screener. Sector, industry, beta,
    /// country, and actively-trading status stay unset: Yahoo's screener
    /// response carries none of them as a result column.
    async fn fetch_screener(
        &self,
        filters: &crate::models::discovery::reference::ScreenerFilters,
    ) -> Result<Vec<crate::models::discovery::reference::ScreenerMatch>> {
        use crate::models::discovery::screeners::{
            EquityField, EquityScreenerQuery, ScreenerFieldExt,
        };

        let mut query = EquityScreenerQuery::new();
        query = add_range_condition(
            query,
            EquityField::IntradayMarketCap,
            filters.market_cap_min,
            filters.market_cap_max,
        );
        query = add_range_condition(
            query,
            EquityField::IntradayPrice,
            filters.price_min,
            filters.price_max,
        );
        query = add_range_condition(query, EquityField::Beta, filters.beta_min, filters.beta_max);
        if let Some(min) = filters.volume_min {
            query = add_range_condition(query, EquityField::DayVolume, Some(min), None);
        }
        if let Some(sector) = &filters.sector {
            query = query.add_condition(EquityField::Sector.eq_str(sector.clone()));
        }
        if let Some(industry) = &filters.industry {
            query = query.add_condition(EquityField::Industry.eq_str(industry.clone()));
        }
        if let Some(exchange) = &filters.exchange {
            query = query.add_condition(EquityField::Exchange.eq_str(exchange.clone()));
        }
        if let Some(limit) = filters.limit {
            query = query.size(limit);
        }

        let results =
            crate::adapters::yahoo::discovery::screeners::fetch_custom(&self.client, query).await?;
        Ok(screener_quotes_to_screener_matches(results.quotes))
    }
}

fn add_range_condition<F: crate::models::discovery::screeners::ScreenerField>(
    query: crate::models::discovery::screeners::ScreenerQuery<F>,
    field: F,
    min: Option<f64>,
    max: Option<f64>,
) -> crate::models::discovery::screeners::ScreenerQuery<F> {
    use crate::models::discovery::screeners::ScreenerFieldExt;

    match (min, max) {
        (Some(min), Some(max)) => query.add_condition(field.between(min, max)),
        (Some(min), None) => query.add_condition(field.gte(min)),
        (None, Some(max)) => query.add_condition(field.lte(max)),
        (None, None) => query,
    }
}

fn search_quotes_to_symbol_matches(
    quotes: crate::models::discovery::search::SearchQuotes,
) -> Vec<crate::models::discovery::reference::SymbolMatch> {
    quotes
        .into_iter()
        .map(|q| crate::models::discovery::reference::SymbolMatch {
            symbol: q.symbol,
            id: None,
            name: q.short_name.or(q.long_name),
            exchange: q.exchange,
            asset_type: q.quote_type,
            currency: None,
            active: None,
            market_cap_rank: None,
            thumbnail: q.logo_url.clone(),
            image: q.logo_url,
        })
        .collect()
}

fn screener_quotes_to_screener_matches(
    quotes: Vec<crate::models::discovery::screeners::ScreenerQuote>,
) -> Vec<crate::models::discovery::reference::ScreenerMatch> {
    quotes
        .into_iter()
        .map(|q| crate::models::discovery::reference::ScreenerMatch {
            symbol: q.symbol,
            name: Some(q.short_name).filter(|s| !s.is_empty()).or(q.long_name),
            price: q.regular_market_price.raw,
            market_cap: q.market_cap.and_then(|v| v.raw).map(|v| v as f64),
            sector: None,
            industry: None,
            beta: None,
            volume: q
                .regular_market_volume
                .and_then(|v| v.raw)
                .map(|v| v as f64),
            exchange: Some(q.exchange),
            country: None,
            is_etf: Some(q.quote_type == "ETF"),
            is_actively_trading: None,
        })
        .collect()
}

#[async_trait::async_trait]
impl CalendarProvider for YahooProvider {
    /// Yahoo serves live exchange open/closed status only. Other kinds fall
    /// through to the next routed provider. `from`/`to` are ignored: this is
    /// a snapshot, not a dated event.
    async fn fetch_market_calendar(
        &self,
        kind: crate::models::calendar::market::CalendarKind,
        _from: &str,
        _to: &str,
    ) -> Result<Vec<crate::models::calendar::market::MarketCalendarEntry>> {
        if kind != crate::models::calendar::market::CalendarKind::MarketStatus {
            return Err(self.not_supported(kind.operation()));
        }
        let hours = crate::adapters::yahoo::market::hours::fetch(&self.client, None).await?;
        Ok(market_times_to_calendar_entries(hours.markets))
    }
}

fn market_times_to_calendar_entries(
    markets: Vec<crate::models::market::hours::MarketTime>,
) -> Vec<crate::models::calendar::market::MarketCalendarEntry> {
    markets
        .into_iter()
        .map(|m| crate::models::calendar::market::MarketCalendarEntry {
            symbol: None,
            date: None,
            detail: crate::models::calendar::market::CalendarDetail::MarketHoliday {
                name: Some(m.name),
                exchange: None,
                status: Some(m.status),
                open: m.open,
                close: m.close,
            },
        })
        .collect()
}

#[async_trait::async_trait]
impl OptionsProvider for YahooProvider {
    async fn fetch_options(
        &self,
        symbol: &str,
        date: Option<i64>,
    ) -> Result<crate::models::options::Options> {
        crate::adapters::yahoo::options::fetch(&self.client, symbol, date).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for YahooProvider {
    fn as_quote(&self) -> Option<&dyn QuoteProvider> {
        Some(self)
    }
    fn as_chart(&self) -> Option<&dyn ChartProvider> {
        Some(self)
    }
    fn as_fundamentals(&self) -> Option<&dyn FundamentalsProvider> {
        Some(self)
    }
    fn as_corporate(&self) -> Option<&dyn CorporateProvider> {
        Some(self)
    }
    fn as_options(&self) -> Option<&dyn OptionsProvider> {
        Some(self)
    }
    fn as_market(&self) -> Option<&dyn MarketProvider> {
        Some(self)
    }
    fn as_indices(&self) -> Option<&dyn IndicesProvider> {
        Some(self)
    }
    fn as_commodities(&self) -> Option<&dyn CommoditiesProvider> {
        Some(self)
    }
    fn as_discovery(&self) -> Option<&dyn DiscoveryProvider> {
        Some(self)
    }
    fn as_calendar(&self) -> Option<&dyn CalendarProvider> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screener_rows_map_to_mover_quotes() {
        let results: crate::models::discovery::screeners::ScreenerResults =
            serde_json::from_value(serde_json::json!({
                "quotes": [{
                    "symbol": "NVDA",
                    "shortName": "NVIDIA Corporation",
                    "quoteType": "EQUITY",
                    "exchange": "NMS",
                    "regularMarketPrice": {"raw": 1234.5, "fmt": "1,234.50"},
                    "regularMarketChange": {"raw": 56.7, "fmt": "56.70"},
                    "regularMarketChangePercent": {"raw": 4.81, "fmt": "4.81%"}
                }],
                "type": "day_gainers",
                "description": "Day gainers"
            }))
            .unwrap();
        let movers = screener_quotes_to_movers(results);
        assert_eq!(movers.len(), 1);
        assert_eq!(movers[0].symbol, "NVDA");
        assert_eq!(movers[0].name.as_deref(), Some("NVIDIA Corporation"));
        assert_eq!(movers[0].price, Some(1234.5));
        assert_eq!(movers[0].change, Some(56.7));
        assert_eq!(movers[0].change_percent, Some(4.81));
    }

    fn fixture_price() -> crate::models::quote::price::Price {
        serde_json::from_value(serde_json::json!({
            "shortName": "S&P 500",
            "regularMarketPrice": {"raw": 5678.9, "fmt": "5,678.90"},
            "regularMarketChange": {"raw": 12.3, "fmt": "12.30"},
            "regularMarketChangePercent": {"raw": 0.22, "fmt": "0.22%"},
            "regularMarketTime": 1_700_000_000
        }))
        .unwrap()
    }

    #[test]
    fn price_maps_to_index_quote() {
        let quote = price_to_index_quote("^GSPC", Some(&fixture_price()));
        assert_eq!(quote.symbol, "^GSPC");
        assert_eq!(quote.name.as_deref(), Some("S&P 500"));
        assert_eq!(quote.price, Some(5678.9));
        assert_eq!(quote.change, Some(12.3));
        assert_eq!(quote.change_percent, Some(0.22));
        assert_eq!(quote.timestamp, Some(1_700_000_000));
    }

    #[test]
    fn price_maps_to_commodity_quote() {
        let quote = price_to_commodity_quote("GC=F", Some(&fixture_price()));
        assert_eq!(quote.symbol, "GC=F");
        assert_eq!(quote.name.as_deref(), Some("S&P 500"));
        assert_eq!(quote.unit, None);
        assert_eq!(quote.price, Some(5678.9));
        assert_eq!(quote.change, Some(12.3));
        assert_eq!(quote.change_percent, Some(0.22));
    }

    #[test]
    fn search_quotes_map_to_symbol_matches() {
        let quotes: crate::models::discovery::search::SearchQuotes =
            serde_json::from_value(serde_json::json!([{
                "symbol": "AAPL",
                "shortName": "Apple Inc.",
                "quoteType": "EQUITY",
                "exchange": "NMS",
                "logoUrl": "https://logo.example/aapl.png"
            }]))
            .unwrap();
        let matches = search_quotes_to_symbol_matches(quotes);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].symbol, "AAPL");
        assert_eq!(matches[0].name.as_deref(), Some("Apple Inc."));
        assert_eq!(matches[0].exchange.as_deref(), Some("NMS"));
        assert_eq!(matches[0].asset_type.as_deref(), Some("EQUITY"));
        assert_eq!(
            matches[0].thumbnail.as_deref(),
            Some("https://logo.example/aapl.png")
        );
    }

    #[test]
    fn screener_quotes_map_to_screener_matches() {
        let quotes: Vec<crate::models::discovery::screeners::ScreenerQuote> =
            serde_json::from_value(serde_json::json!([{
                "symbol": "NVDA",
                "shortName": "NVIDIA Corporation",
                "quoteType": "EQUITY",
                "exchange": "NMS",
                "regularMarketPrice": {"raw": 1234.5, "fmt": "1,234.50"},
                "regularMarketChange": {"raw": 56.7, "fmt": "56.70"},
                "regularMarketChangePercent": {"raw": 4.81, "fmt": "4.81%"},
                "marketCap": {"raw": 3_000_000_000_000_i64, "fmt": "3.00T"},
                "regularMarketVolume": {"raw": 45_000_000, "fmt": "45M"}
            }]))
            .unwrap();
        let matches = screener_quotes_to_screener_matches(quotes);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].symbol, "NVDA");
        assert_eq!(matches[0].name.as_deref(), Some("NVIDIA Corporation"));
        assert_eq!(matches[0].price, Some(1234.5));
        assert_eq!(matches[0].market_cap, Some(3_000_000_000_000.0));
        assert_eq!(matches[0].volume, Some(45_000_000.0));
        assert_eq!(matches[0].exchange.as_deref(), Some("NMS"));
        assert_eq!(matches[0].is_etf, Some(false));
    }

    #[test]
    fn range_condition_picks_between_gte_or_lte() {
        use crate::models::discovery::screeners::{EquityField, EquityScreenerQuery};

        let both = add_range_condition(
            EquityScreenerQuery::new(),
            EquityField::PeRatio,
            Some(10.0),
            Some(25.0),
        );
        let min_only = add_range_condition(
            EquityScreenerQuery::new(),
            EquityField::PeRatio,
            Some(10.0),
            None,
        );
        let neither =
            add_range_condition(EquityScreenerQuery::new(), EquityField::PeRatio, None, None);
        assert_eq!(both.query.operands.len(), 1);
        assert_eq!(min_only.query.operands.len(), 1);
        assert_eq!(neither.query.operands.len(), 0);
    }

    #[test]
    fn market_times_map_to_calendar_entries() {
        let markets = vec![
            serde_json::from_value::<crate::models::market::hours::MarketTime>(serde_json::json!({
                "id": "us",
                "name": "U.S. markets",
                "status": "closed",
                "open": "2026-08-03T00:00:00Z",
                "close": "2026-08-03T06:30:00Z"
            }))
            .unwrap(),
        ];
        let entries = market_times_to_calendar_entries(markets);
        assert_eq!(entries.len(), 1);
        let crate::models::calendar::market::CalendarDetail::MarketHoliday {
            name,
            exchange,
            status,
            open,
            close,
        } = &entries[0].detail
        else {
            unreachable!()
        };
        assert_eq!(name.as_deref(), Some("U.S. markets"));
        assert_eq!(*exchange, None);
        assert_eq!(status.as_deref(), Some("closed"));
        assert_eq!(open.as_deref(), Some("2026-08-03T00:00:00Z"));
        assert_eq!(close.as_deref(), Some("2026-08-03T06:30:00Z"));
    }
}

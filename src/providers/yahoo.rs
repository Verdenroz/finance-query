//! Yahoo Finance provider implementation.
//!
//! Wraps YahooClient and delegates conversion to adapter functions
//! to keep this file focused on routing and lifecycle.

use super::{
    CalendarProvider, Capability, ChartProvider, CommoditiesProvider, CorporateProvider,
    DiscoveryProvider, FundamentalsProvider, FuturesProvider, IndicesProvider, MarketProvider,
    OptionsProvider, ProviderAdapter, ProviderCore, QuoteProvider,
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
    .union(Capability::CALENDAR)
    .union(Capability::FUTURES);

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

    /// Derived from `recommendationTrend`'s current-period ("0m") counts
    /// rather than a dedicated endpoint. `consensus` (a headline label like
    /// `"Buy"`) has no Yahoo equivalent and stays unset.
    async fn fetch_rating_consensus(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::RatingConsensus> {
        let resp =
            crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await?;
        Ok(recommendation_trend_to_rating_consensus(
            symbol,
            resp.recommendation_trend,
        ))
    }

    /// Assembled from `assetProfile` and `price` — Yahoo has no single
    /// endpoint matching this shape's fields.
    async fn fetch_company_profile(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::CompanyProfile> {
        let resp =
            crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await?;
        Ok(profile_and_price_to_company_profile(
            symbol,
            resp.asset_profile,
            resp.price,
        ))
    }

    /// Derived from `financialData`'s target-price fields rather than a
    /// dedicated endpoint.
    async fn fetch_price_target_consensus(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::PriceTargetConsensus> {
        let resp =
            crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await?;
        Ok(financial_data_to_price_target_consensus(
            symbol,
            resp.financial_data,
        ))
    }

    /// Derived from `upgradeDowngradeHistory`, the same module
    /// `Ticker::grading_history()` surfaces.
    async fn fetch_grading_history(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::fundamentals::GradingAction>> {
        let resp =
            crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await?;
        Ok(upgrade_downgrade_history_to_grading_actions(
            symbol,
            resp.upgrade_downgrade_history,
        ))
    }

    /// Derived from `earningsHistory`.
    async fn fetch_earnings_surprises(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::fundamentals::EarningsSurprise>> {
        let resp =
            crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await?;
        Ok(earnings_history_to_surprises(symbol, resp.earnings_history))
    }

    /// Assembled from `fundProfile`, `summaryDetail`, and `topHoldings` —
    /// modules already fetched on every quote call. Coverage is ragged
    /// against FMP/Alpha Vantage's own partial `EtfProfile` sources:
    /// `inception_date` stays unset since none of Yahoo's fund modules
    /// carry it.
    async fn fetch_etf_profile(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::EtfProfile> {
        let resp =
            crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await?;
        Ok(fund_modules_to_etf_profile(
            symbol,
            resp.price,
            resp.summary_detail,
            resp.fund_profile,
            resp.top_holdings,
        ))
    }

    /// Computed locally from `fetch_financials`/`fetch_quote` — see
    /// `super::yahoo_ttm` for which fields are honestly derivable from a
    /// standard three-statement filing and which stay unset.
    async fn fetch_key_metrics_ttm(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::KeyMetricsTtm> {
        super::yahoo_ttm::fetch_key_metrics_ttm(&self.client, symbol).await
    }

    /// Computed locally from `fetch_financials`/`fetch_quote` — see
    /// `super::yahoo_ttm` for which fields are honestly derivable from a
    /// standard three-statement filing and which stay unset.
    async fn fetch_ratios_ttm(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::FinancialRatiosTtm> {
        super::yahoo_ttm::fetch_ratios_ttm(&self.client, symbol).await
    }
}

fn recommendation_trend_to_rating_consensus(
    symbol: &str,
    trend: Option<crate::models::corporate::RecommendationTrend>,
) -> crate::models::fundamentals::RatingConsensus {
    let period = trend.and_then(|t| {
        t.trend
            .into_iter()
            .find(|p| p.period.as_deref() == Some("0m"))
    });
    crate::models::fundamentals::RatingConsensus {
        symbol: Some(symbol.to_string()),
        strong_buy: period.as_ref().and_then(|p| p.strong_buy).map(i64::from),
        buy: period.as_ref().and_then(|p| p.buy).map(i64::from),
        hold: period.as_ref().and_then(|p| p.hold).map(i64::from),
        sell: period.as_ref().and_then(|p| p.sell).map(i64::from),
        strong_sell: period.as_ref().and_then(|p| p.strong_sell).map(i64::from),
        consensus: None,
    }
}

fn profile_and_price_to_company_profile(
    symbol: &str,
    profile: Option<crate::models::corporate::AssetProfile>,
    price: Option<crate::models::quote::price::Price>,
) -> crate::models::fundamentals::CompanyProfile {
    crate::models::fundamentals::CompanyProfile {
        symbol: Some(symbol.to_string()),
        name: price
            .as_ref()
            .and_then(|p| p.short_name.clone().or_else(|| p.long_name.clone())),
        description: profile
            .as_ref()
            .and_then(|p| p.long_business_summary.clone()),
        asset_type: price.as_ref().and_then(|p| p.quote_type.clone()),
        exchange: price.as_ref().and_then(|p| p.exchange_name.clone()),
        currency: price.as_ref().and_then(|p| p.currency.clone()),
        country: profile.as_ref().and_then(|p| p.country.clone()),
        sector: profile.as_ref().and_then(|p| p.sector.clone()),
        industry: profile.as_ref().and_then(|p| p.industry.clone()),
        market_capitalization: price
            .as_ref()
            .and_then(|p| p.market_cap.as_ref())
            .and_then(|v| v.raw)
            .map(|v| v as f64),
    }
}

fn financial_data_to_price_target_consensus(
    symbol: &str,
    data: Option<crate::models::quote::FinancialData>,
) -> crate::models::fundamentals::PriceTargetConsensus {
    crate::models::fundamentals::PriceTargetConsensus {
        symbol: Some(symbol.to_string()),
        target_high: data
            .as_ref()
            .and_then(|d| d.target_high_price.as_ref())
            .and_then(|v| v.raw),
        target_low: data
            .as_ref()
            .and_then(|d| d.target_low_price.as_ref())
            .and_then(|v| v.raw),
        target_consensus: data
            .as_ref()
            .and_then(|d| d.target_mean_price.as_ref())
            .and_then(|v| v.raw),
        target_median: data
            .as_ref()
            .and_then(|d| d.target_median_price.as_ref())
            .and_then(|v| v.raw),
    }
}

fn upgrade_downgrade_history_to_grading_actions(
    symbol: &str,
    history: Option<crate::models::quote::UpgradeDowngradeHistory>,
) -> Vec<crate::models::fundamentals::GradingAction> {
    history
        .map(|h| h.history)
        .unwrap_or_default()
        .into_iter()
        .map(|g| crate::models::fundamentals::GradingAction {
            symbol: Some(symbol.to_string()),
            date: g.epoch_grade_date.and_then(epoch_to_date),
            grading_company: g.firm,
            previous_grade: g.from_grade,
            new_grade: g.to_grade,
        })
        .collect()
}

fn earnings_history_to_surprises(
    symbol: &str,
    history: Option<crate::models::quote::EarningsHistory>,
) -> Vec<crate::models::fundamentals::EarningsSurprise> {
    history
        .map(|h| h.history)
        .unwrap_or_default()
        .into_iter()
        .map(|e| crate::models::fundamentals::EarningsSurprise {
            symbol: Some(symbol.to_string()),
            date: e.quarter.and_then(|v| v.raw).and_then(epoch_to_date),
            actual_eps: e.eps_actual.and_then(|v| v.raw),
            estimated_eps: e.eps_estimate.and_then(|v| v.raw),
            surprise: e.eps_difference.and_then(|v| v.raw),
            surprise_percent: e.surprise_percent.and_then(|v| v.raw),
        })
        .collect()
}

fn fund_modules_to_etf_profile(
    symbol: &str,
    price: Option<crate::models::quote::price::Price>,
    summary_detail: Option<crate::models::quote::SummaryDetail>,
    fund_profile: Option<crate::models::quote::FundProfile>,
    top_holdings: Option<crate::models::quote::TopHoldings>,
) -> crate::models::fundamentals::EtfProfile {
    let (asset_type, fees) = match fund_profile {
        Some(p) => (p.legal_type, p.fees_expenses_investment),
        None => (None, None),
    };
    let (holdings, sector_weightings) = match top_holdings {
        Some(t) => (t.holdings.unwrap_or_default(), t.sector_weightings),
        None => (Vec::new(), None),
    };

    crate::models::fundamentals::EtfProfile {
        symbol: Some(symbol.to_string()),
        name: price
            .as_ref()
            .and_then(|p| p.short_name.clone().or_else(|| p.long_name.clone())),
        asset_type,
        // Yahoo reports total net assets in millions; the canonical field is
        // an absolute value (confirmed against Alpha Vantage's own mapping).
        net_assets: fees
            .as_ref()
            .and_then(|f| f.total_net_assets.as_ref())
            .and_then(|v| v.raw)
            .map(|millions| millions * 1_000_000.0),
        net_expense_ratio: fees
            .as_ref()
            .and_then(|f| f.annual_report_expense_ratio.as_ref())
            .and_then(|v| v.raw),
        portfolio_turnover: fees
            .as_ref()
            .and_then(|f| f.annual_holdings_turnover.as_ref())
            .and_then(|v| v.raw),
        dividend_yield: summary_detail
            .and_then(|s| s.dividend_yield)
            .and_then(|v| v.raw),
        inception_date: None,
        holdings: holdings
            .into_iter()
            .map(|h| crate::models::fundamentals::EtfHolding {
                symbol: h.symbol,
                description: h.holding_name,
                weight: h.holding_percent.and_then(|v| v.raw),
            })
            .collect(),
        sector_weightings: sector_weightings
            .map(sector_weighting_rows)
            .unwrap_or_default(),
        country_weightings: Vec::new(),
    }
}

fn sector_weighting_rows(
    sw: crate::models::corporate::top_holdings::SectorWeighting,
) -> Vec<crate::models::fundamentals::EtfSectorWeighting> {
    use crate::constants::sectors::Sector;
    [
        (Sector::RealEstate, sw.realestate),
        (Sector::ConsumerCyclical, sw.consumer_cyclical),
        (Sector::BasicMaterials, sw.basic_materials),
        (Sector::ConsumerDefensive, sw.consumer_defensive),
        (Sector::Technology, sw.technology),
        (Sector::CommunicationServices, sw.communication_services),
        (Sector::FinancialServices, sw.financial_services),
        (Sector::Utilities, sw.utilities),
        (Sector::Industrials, sw.industrials),
        (Sector::Energy, sw.energy),
        (Sector::Healthcare, sw.healthcare),
    ]
    .into_iter()
    .filter_map(|(sector, weight)| {
        weight
            .and_then(|v| v.raw)
            .map(|w| crate::models::fundamentals::EtfSectorWeighting {
                sector: Some(sector.display_name().to_string()),
                weight: Some(w),
            })
    })
    .collect()
}

#[async_trait::async_trait]
impl FuturesProvider for YahooProvider {
    /// Derived from the generic quote endpoint, the same resolution trick
    /// used for indices and commodities. `expiration_date` and `underlying`
    /// stay unset: Yahoo's quote modules carry no contract metadata for
    /// futures instruments.
    async fn fetch_futures_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::futures::FuturesQuote> {
        let resp =
            crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await?;
        Ok(price_and_summary_to_futures_quote(
            symbol,
            resp.price,
            resp.summary_detail,
        ))
    }
}

fn price_and_summary_to_futures_quote(
    symbol: &str,
    price: Option<crate::models::quote::price::Price>,
    summary_detail: Option<crate::models::quote::SummaryDetail>,
) -> crate::models::futures::FuturesQuote {
    crate::models::futures::FuturesQuote {
        symbol: symbol.to_string(),
        name: price
            .as_ref()
            .and_then(|p| p.short_name.clone().or_else(|| p.long_name.clone())),
        underlying: None,
        exchange: price.as_ref().and_then(|p| p.exchange_name.clone()),
        expiration_date: None,
        price: price.as_ref().and_then(|p| p.current_price()),
        change: price.as_ref().and_then(|p| p.day_change()),
        change_percent: price.as_ref().and_then(|p| p.day_change_percent()),
        // summaryDetail's openInterest is loosely typed (serde_json::Value);
        // every other numeric field on this module uses the {raw, fmt} shape,
        // so assume the same here rather than retyping it without a live check.
        open_interest: summary_detail
            .and_then(|s| s.open_interest)
            .and_then(|v| v.get("raw").and_then(|r| r.as_u64())),
        volume: price
            .as_ref()
            .and_then(|p| p.regular_market_volume.as_ref())
            .and_then(|v| v.raw)
            .map(|v| v as u64),
        timestamp: price.and_then(|p| p.regular_market_time),
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

    /// Fans out over Yahoo's per-sector overview pages, one request per
    /// sector — there's no bulk sector-performance endpoint. A sector that
    /// fails to fetch is dropped rather than failing the whole call.
    async fn fetch_sector_performance(
        &self,
    ) -> Result<Vec<crate::models::market::performance::SectorPerformance>> {
        use crate::constants::sectors::Sector;

        let fetches = Sector::all().iter().map(|&sector| async move {
            match crate::adapters::yahoo::market::sectors::fetch(&self.client, sector).await {
                Ok(data) => Some(sector_data_to_performance(data)),
                Err(err) => {
                    tracing::warn!("failed to fetch {sector:?} sector performance: {err}");
                    None
                }
            }
        });
        Ok(futures::future::join_all(fetches)
            .await
            .into_iter()
            .flatten()
            .collect())
    }

    async fn fetch_sector_performance_history(
        &self,
        limit: u32,
    ) -> Result<Vec<crate::models::market::performance::SectorPerformanceHistory>> {
        crate::adapters::yahoo::market::sector_history::fetch_sector_performance_history(
            &self.client,
            limit,
        )
        .await
    }

    /// Fans out over the 11 GICS sectors' equity screeners — there's no
    /// bulk sector-P/E endpoint — aggregating each sector's constituent
    /// trailing P/E by median (more outlier-robust than a mean against
    /// mega-cap or negative-earnings names). Industry P/E stays unrouted:
    /// the same fan-out over ~160 industry slugs is too costly for one
    /// call, and thin industries wouldn't have enough sampled P/Es for a
    /// meaningful aggregate.
    async fn fetch_sector_pe(&self) -> Result<Vec<crate::models::market::performance::SectorPe>> {
        use crate::constants::sectors::Sector;
        use crate::models::discovery::screeners::{
            EquityField, EquityScreenerQuery, ScreenerFieldExt,
        };

        const SAMPLE_SIZE: u32 = 50;

        let fetches = Sector::all().iter().map(|&sector| async move {
            // Unfiltered, market-cap-sorted results skew toward foreign ADR/CDI
            // listings of the same mega-caps (e.g. "AAPL.BA"), which don't carry
            // trailingPE and would starve the median of real samples.
            let query = EquityScreenerQuery::new()
                .add_condition(EquityField::Sector.eq_str(sector))
                .add_condition(EquityField::Region.eq_str("us"))
                .size(SAMPLE_SIZE);
            match crate::adapters::yahoo::discovery::screeners::fetch_custom(&self.client, query)
                .await
            {
                Ok(results) => median_trailing_pe(&results.quotes).map(|pe| {
                    crate::models::market::performance::SectorPe {
                        sector: sector.display_name().to_string(),
                        exchange: None,
                        pe: Some(pe),
                        date: None,
                    }
                }),
                Err(err) => {
                    tracing::warn!("failed to fetch {sector:?} sector P/E: {err}");
                    None
                }
            }
        });
        Ok(futures::future::join_all(fetches)
            .await
            .into_iter()
            .flatten()
            .collect())
    }
}

/// Median trailing P/E across screener rows, excluding non-positive or
/// non-finite values (negative-earnings companies aren't meaningfully
/// "priced" via P/E, so including them would skew the aggregate).
fn median_trailing_pe(
    quotes: &[crate::models::discovery::screeners::ScreenerQuote],
) -> Option<f64> {
    let mut values: Vec<f64> = quotes
        .iter()
        .filter_map(|q| q.trailing_pe.as_ref().and_then(|v| v.raw))
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect();
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.total_cmp(b));
    let mid = values.len() / 2;
    Some(if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    })
}

fn sector_data_to_performance(
    data: crate::models::market::sectors::SectorData,
) -> crate::models::market::performance::SectorPerformance {
    crate::models::market::performance::SectorPerformance {
        sector: data.name,
        exchange: None,
        change_percent: data
            .performance
            .and_then(|p| p.day_change_percent)
            .and_then(|v| v.raw),
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
    fn as_futures(&self) -> Option<&dyn FuturesProvider> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_live_sector_pe() {
        let provider = YahooProvider::new(&crate::adapters::yahoo::client::ClientConfig::default())
            .await
            .unwrap();
        let pes = provider.fetch_sector_pe().await.unwrap();
        assert!(!pes.is_empty());
        assert!(pes.iter().any(|p| p.pe.is_some_and(|pe| pe > 0.0)));
    }

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

    #[test]
    fn sector_data_maps_to_sector_performance() {
        let data: crate::models::market::sectors::SectorData =
            serde_json::from_value(serde_json::json!({
                "name": "Technology",
                "key": "technology",
                "performance": {
                    "dayChangePercent": {"raw": 1.23, "fmt": "1.23%"}
                },
                "topCompanies": [],
                "topEtfs": [],
                "topMutualFunds": [],
                "industries": [],
                "researchReports": []
            }))
            .unwrap();
        let performance = sector_data_to_performance(data);
        assert_eq!(performance.sector, "Technology");
        assert_eq!(performance.exchange, None);
        assert_eq!(performance.change_percent, Some(1.23));
    }

    fn quote_with_pe(pe: Option<f64>) -> crate::models::discovery::screeners::ScreenerQuote {
        serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "quoteType": "EQUITY",
            "exchange": "NMS",
            "regularMarketPrice": {"raw": 200.0, "fmt": "200.00"},
            "regularMarketChange": {"raw": 1.0, "fmt": "1.00"},
            "regularMarketChangePercent": {"raw": 0.5, "fmt": "0.50%"},
            "trailingPE": pe.map(|v| serde_json::json!({"raw": v, "fmt": v.to_string()})),
        }))
        .unwrap()
    }

    #[test]
    fn median_trailing_pe_of_an_odd_sample_is_the_middle_value() {
        let quotes = [10.0, 20.0, 30.0].map(|pe| quote_with_pe(Some(pe)));
        assert_eq!(median_trailing_pe(&quotes), Some(20.0));
    }

    #[test]
    fn median_trailing_pe_of_an_even_sample_averages_the_middle_two() {
        let quotes = [10.0, 20.0, 30.0, 40.0].map(|pe| quote_with_pe(Some(pe)));
        assert_eq!(median_trailing_pe(&quotes), Some(25.0));
    }

    #[test]
    fn median_trailing_pe_excludes_negative_and_missing_values() {
        let quotes = [
            quote_with_pe(Some(15.0)),
            quote_with_pe(Some(-5.0)),
            quote_with_pe(None),
            quote_with_pe(Some(25.0)),
        ];
        assert_eq!(median_trailing_pe(&quotes), Some(20.0));
    }

    #[test]
    fn median_trailing_pe_of_no_valid_samples_is_none() {
        let quotes = [quote_with_pe(Some(-5.0)), quote_with_pe(None)];
        assert_eq!(median_trailing_pe(&quotes), None);
    }

    #[test]
    fn recommendation_trend_maps_current_period_to_rating_consensus() {
        let trend: crate::models::corporate::RecommendationTrend =
            serde_json::from_value(serde_json::json!({
                "trend": [
                    {"period": "0m", "strongBuy": 5, "buy": 10, "hold": 3, "sell": 1, "strongSell": 0},
                    {"period": "-1m", "strongBuy": 4, "buy": 9, "hold": 4, "sell": 1, "strongSell": 0}
                ]
            }))
            .unwrap();
        let consensus = recommendation_trend_to_rating_consensus("AAPL", Some(trend));
        assert_eq!(consensus.symbol.as_deref(), Some("AAPL"));
        assert_eq!(consensus.strong_buy, Some(5));
        assert_eq!(consensus.buy, Some(10));
        assert_eq!(consensus.hold, Some(3));
        assert_eq!(consensus.sell, Some(1));
        assert_eq!(consensus.strong_sell, Some(0));
        assert_eq!(consensus.consensus, None);
    }

    #[test]
    fn profile_and_price_map_to_company_profile() {
        let profile: crate::models::corporate::AssetProfile =
            serde_json::from_value(serde_json::json!({
                "country": "United States",
                "sector": "Technology",
                "industry": "Consumer Electronics",
                "longBusinessSummary": "Designs and sells consumer electronics."
            }))
            .unwrap();
        let price: crate::models::quote::price::Price = serde_json::from_value(serde_json::json!({
            "shortName": "Apple Inc.",
            "quoteType": "EQUITY",
            "exchangeName": "NMS",
            "currency": "USD",
            "marketCap": {"raw": 3_000_000_000_000_i64, "fmt": "3.00T"}
        }))
        .unwrap();
        let company = profile_and_price_to_company_profile("AAPL", Some(profile), Some(price));
        assert_eq!(company.symbol.as_deref(), Some("AAPL"));
        assert_eq!(company.name.as_deref(), Some("Apple Inc."));
        assert_eq!(
            company.description.as_deref(),
            Some("Designs and sells consumer electronics.")
        );
        assert_eq!(company.asset_type.as_deref(), Some("EQUITY"));
        assert_eq!(company.exchange.as_deref(), Some("NMS"));
        assert_eq!(company.currency.as_deref(), Some("USD"));
        assert_eq!(company.country.as_deref(), Some("United States"));
        assert_eq!(company.sector.as_deref(), Some("Technology"));
        assert_eq!(company.industry.as_deref(), Some("Consumer Electronics"));
        assert_eq!(company.market_capitalization, Some(3_000_000_000_000.0));
    }

    #[test]
    fn financial_data_maps_to_price_target_consensus() {
        let data: crate::models::quote::FinancialData = serde_json::from_value(serde_json::json!({
            "targetHighPrice": {"raw": 250.0, "fmt": "250.00"},
            "targetLowPrice": {"raw": 180.0, "fmt": "180.00"},
            "targetMeanPrice": {"raw": 215.0, "fmt": "215.00"},
            "targetMedianPrice": {"raw": 210.0, "fmt": "210.00"}
        }))
        .unwrap();
        let consensus = financial_data_to_price_target_consensus("AAPL", Some(data));
        assert_eq!(consensus.symbol.as_deref(), Some("AAPL"));
        assert_eq!(consensus.target_high, Some(250.0));
        assert_eq!(consensus.target_low, Some(180.0));
        assert_eq!(consensus.target_consensus, Some(215.0));
        assert_eq!(consensus.target_median, Some(210.0));
    }

    #[test]
    fn upgrade_downgrade_history_maps_to_grading_actions() {
        let history: crate::models::quote::UpgradeDowngradeHistory =
            serde_json::from_value(serde_json::json!({
                "history": [{
                    "epochGradeDate": 1_700_000_000,
                    "firm": "Morgan Stanley",
                    "fromGrade": "Hold",
                    "toGrade": "Buy",
                    "action": "up"
                }]
            }))
            .unwrap();
        let actions = upgrade_downgrade_history_to_grading_actions("AAPL", Some(history));
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(actions[0].date.as_deref(), Some("2023-11-14"));
        assert_eq!(
            actions[0].grading_company.as_deref(),
            Some("Morgan Stanley")
        );
        assert_eq!(actions[0].previous_grade.as_deref(), Some("Hold"));
        assert_eq!(actions[0].new_grade.as_deref(), Some("Buy"));
    }

    #[test]
    fn earnings_history_maps_to_surprises() {
        let history: crate::models::quote::EarningsHistory =
            serde_json::from_value(serde_json::json!({
                "history": [{
                    "quarter": {"raw": 1_700_000_000, "fmt": "3Q2023"},
                    "epsActual": {"raw": 1.5, "fmt": "1.50"},
                    "epsEstimate": {"raw": 1.4, "fmt": "1.40"},
                    "epsDifference": {"raw": 0.1, "fmt": "0.10"},
                    "surprisePercent": {"raw": 7.1, "fmt": "7.10%"}
                }]
            }))
            .unwrap();
        let surprises = earnings_history_to_surprises("AAPL", Some(history));
        assert_eq!(surprises.len(), 1);
        assert_eq!(surprises[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(surprises[0].date.as_deref(), Some("2023-11-14"));
        assert_eq!(surprises[0].actual_eps, Some(1.5));
        assert_eq!(surprises[0].estimated_eps, Some(1.4));
        assert_eq!(surprises[0].surprise, Some(0.1));
        assert_eq!(surprises[0].surprise_percent, Some(7.1));
    }

    #[test]
    fn fund_modules_map_to_etf_profile() {
        let price = fixture_price();
        let summary_detail: crate::models::quote::SummaryDetail =
            serde_json::from_value(serde_json::json!({
                "dividendYield": {"raw": 0.0058, "fmt": "0.58%"}
            }))
            .unwrap();
        let fund_profile: crate::models::quote::FundProfile =
            serde_json::from_value(serde_json::json!({
                "legalType": "Exchange Traded Fund",
                "feesExpensesInvestment": {
                    "annualReportExpenseRatio": {"raw": 0.002, "fmt": "0.20%"},
                    "annualHoldingsTurnover": {"raw": 0.07, "fmt": "7.00%"},
                    "totalNetAssets": {"raw": 300_000.0, "fmt": "300,000"}
                }
            }))
            .unwrap();
        let top_holdings: crate::models::quote::TopHoldings =
            serde_json::from_value(serde_json::json!({
                "holdings": [{
                    "symbol": "MSFT",
                    "holdingName": "Microsoft Corp",
                    "holdingPercent": {"raw": 0.081, "fmt": "8.10%"}
                }],
                "sectorWeightings": [
                    {"technology": {"raw": 0.45, "fmt": "45.00%"}},
                    {"healthcare": {"raw": 0.12, "fmt": "12.00%"}}
                ]
            }))
            .unwrap();
        let profile = fund_modules_to_etf_profile(
            "QQQ",
            Some(price),
            Some(summary_detail),
            Some(fund_profile),
            Some(top_holdings),
        );
        assert_eq!(profile.symbol.as_deref(), Some("QQQ"));
        assert_eq!(profile.name.as_deref(), Some("S&P 500"));
        assert_eq!(profile.asset_type.as_deref(), Some("Exchange Traded Fund"));
        assert_eq!(profile.net_assets, Some(300_000_000_000.0));
        assert_eq!(profile.net_expense_ratio, Some(0.002));
        assert_eq!(profile.portfolio_turnover, Some(0.07));
        assert_eq!(profile.dividend_yield, Some(0.0058));
        assert_eq!(profile.inception_date, None);
        assert_eq!(profile.holdings.len(), 1);
        assert_eq!(profile.holdings[0].symbol.as_deref(), Some("MSFT"));
        assert_eq!(profile.holdings[0].weight, Some(0.081));
        assert_eq!(profile.sector_weightings.len(), 2);
        assert!(
            profile
                .sector_weightings
                .iter()
                .any(|s| s.sector.as_deref() == Some("Technology") && s.weight == Some(0.45))
        );
    }

    #[test]
    fn price_and_summary_map_to_futures_quote() {
        let price: crate::models::quote::price::Price = serde_json::from_value(serde_json::json!({
            "shortName": "E-mini S&P 500 Dec 26",
            "exchangeName": "CME",
            "regularMarketPrice": {"raw": 5678.9, "fmt": "5,678.90"},
            "regularMarketChange": {"raw": 12.3, "fmt": "12.30"},
            "regularMarketChangePercent": {"raw": 0.22, "fmt": "0.22%"},
            "regularMarketVolume": {"raw": 150_000, "fmt": "150,000"},
            "regularMarketTime": 1_700_000_000
        }))
        .unwrap();
        let summary_detail: crate::models::quote::SummaryDetail =
            serde_json::from_value(serde_json::json!({
                "openInterest": {"raw": 42_000, "fmt": "42,000"}
            }))
            .unwrap();
        let quote = price_and_summary_to_futures_quote("ESZ26", Some(price), Some(summary_detail));
        assert_eq!(quote.symbol, "ESZ26");
        assert_eq!(quote.name.as_deref(), Some("E-mini S&P 500 Dec 26"));
        assert_eq!(quote.exchange.as_deref(), Some("CME"));
        assert_eq!(quote.underlying, None);
        assert_eq!(quote.expiration_date, None);
        assert_eq!(quote.price, Some(5678.9));
        assert_eq!(quote.change, Some(12.3));
        assert_eq!(quote.change_percent, Some(0.22));
        assert_eq!(quote.open_interest, Some(42_000));
        assert_eq!(quote.volume, Some(150_000));
        assert_eq!(quote.timestamp, Some(1_700_000_000));
    }
}

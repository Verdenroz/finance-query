//! Display titles and behaviour hints advertised with every tool.
//!
//! Every tool reads from an upstream data source and changes nothing, so the
//! hints are uniform and only the title varies.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::ToolAnnotations;

const TITLES: &[(&str, &str)] = &[
    ("get_analysis", "Analyst Data"),
    ("get_calendar", "Upcoming Events for Symbols"),
    ("get_chart", "Price Chart (OHLCV)"),
    ("get_commodity", "Commodity Quote"),
    ("get_company_profile", "Company Profile"),
    ("get_congressional_trades", "Congressional Trades"),
    ("get_crypto", "Top Cryptocurrencies"),
    ("get_crypto_news", "Crypto News"),
    ("get_dividends", "Dividend History"),
    ("get_earnings_surprises", "Earnings Surprises"),
    ("get_earnings_transcript", "Earnings Call Transcript"),
    ("get_edgar_facts", "SEC EDGAR XBRL Facts"),
    ("get_edgar_search", "SEC EDGAR Full-Text Search"),
    ("get_edgar_submissions", "SEC EDGAR Filing History"),
    ("get_etf_profile", "ETF Profile and Holdings"),
    ("get_executive_compensation", "Executive Compensation"),
    ("get_fails_to_deliver", "Fails-to-Deliver Records"),
    ("get_fear_and_greed", "Fear & Greed Index"),
    ("get_feeds", "Financial News Feeds"),
    ("get_filing_sections", "SEC Filing Sections"),
    ("get_financials", "Financial Statements"),
    ("get_forex", "Exchange Rate"),
    ("get_forex_news", "Forex News"),
    ("get_fred_series", "FRED Economic Series"),
    ("get_futures", "Futures Quote"),
    ("get_grading_actions", "Analyst Upgrades and Downgrades"),
    ("get_holders", "Ownership and Insider Activity"),
    ("get_index_constituent_changes", "Index Constituent Changes"),
    ("get_index_constituents", "Index Constituents"),
    ("get_indicators", "Technical Indicators"),
    ("get_indices", "World Market Indices"),
    ("get_industry", "Industry Overview"),
    ("get_key_metrics_ttm", "Key Metrics (TTM)"),
    ("get_market_calendar", "Market-Wide Event Calendar"),
    ("get_market_hours", "Market Hours"),
    ("get_market_summary", "Market Summary"),
    ("get_news", "Stock and Market News"),
    ("get_options", "Options Chain"),
    ("get_press_releases", "Company Press Releases"),
    ("get_price_target_consensus", "Price Target Consensus"),
    ("get_price_target_summary", "Price Target Summary"),
    ("get_protocol_tvl", "DeFi Protocol TVL"),
    ("get_protocol_tvl_history", "DeFi Protocol TVL History"),
    ("get_quote", "Stock Quote"),
    ("get_rating_consensus", "Analyst Rating Consensus"),
    ("get_ratios_ttm", "Financial Ratios (TTM)"),
    ("get_recommendations", "Similar Stocks"),
    ("get_risk", "Risk Analytics"),
    ("get_risk_factors", "Filing Risk Factors"),
    ("get_sector", "Sector Overview"),
    ("get_sector_pe", "Sector P/E Ratios"),
    ("get_sector_performance", "Sector Performance"),
    (
        "get_sector_performance_history",
        "Sector Performance History",
    ),
    ("get_short_volume", "Short-Sale Volume"),
    ("get_spark", "Price Sparklines"),
    ("get_splits", "Stock Split History"),
    ("get_symbol_details", "Symbol Reference Details"),
    ("get_transcripts", "Earnings Call Transcripts"),
    ("get_treasury_auctions", "Treasury Auction Results"),
    ("get_treasury_yields", "Treasury Yield Curve"),
    ("get_trending", "Trending Tickers"),
    ("lookup", "Ticker Lookup by Type"),
    ("run_backtest", "Strategy Backtest"),
    ("screener", "Stock Screener"),
    ("search", "Symbol Search"),
];

fn title_for(tool: &str) -> Option<&'static str> {
    TITLES
        .iter()
        .find(|(name, _)| *name == tool)
        .map(|(_, title)| *title)
}

/// Returns `router` with a title and read-only hints on every tool it lists.
pub(super) fn annotate<S>(mut router: ToolRouter<S>) -> ToolRouter<S> {
    for route in router.map.values_mut() {
        let title = title_for(&route.attr.name).map(str::to_string);
        let mut annotations = ToolAnnotations::new()
            .read_only(true)
            .destructive(false)
            .idempotent(true)
            .open_world(true);
        annotations.title = title.clone();
        route.attr.annotations = Some(annotations);
        route.attr.title = title;
    }
    router
}

#[cfg(test)]
mod tests {
    use super::super::FinanceTools;
    use super::*;

    fn annotated_router() -> ToolRouter<FinanceTools> {
        #[allow(unused_mut)]
        let mut router = FinanceTools::tool_router();
        #[cfg(feature = "backtesting")]
        {
            router += FinanceTools::backtest_tool_router();
        }
        annotate(router)
    }

    #[test]
    fn every_tool_is_titled_and_read_only() {
        for tool in annotated_router().list_all() {
            let annotations = tool
                .annotations
                .unwrap_or_else(|| panic!("{} has no annotations", tool.name));
            assert!(annotations.title.is_some(), "{} has no title", tool.name);
            assert_eq!(tool.title, annotations.title, "{}", tool.name);
            assert_eq!(annotations.read_only_hint, Some(true), "{}", tool.name);
            assert_eq!(annotations.destructive_hint, Some(false), "{}", tool.name);
        }
    }

    #[test]
    fn titles_are_unique() {
        let mut titles: Vec<&str> = TITLES.iter().map(|(_, title)| *title).collect();
        titles.sort_unstable();
        titles.dedup();
        assert_eq!(titles.len(), TITLES.len());
    }

    #[cfg(feature = "backtesting")]
    #[test]
    fn every_title_names_a_registered_tool() {
        let router = annotated_router();
        for (name, _) in TITLES {
            assert!(router.has_route(name), "{name} is titled but not a tool");
        }
    }
}

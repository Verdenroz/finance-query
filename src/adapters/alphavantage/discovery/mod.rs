//! DISCOVERY capability: symbol search and the listed-security universe.

use std::collections::BTreeSet;

use crate::adapters::alphavantage::models::{ListingEntryDTO, SymbolMatchDTO};
use crate::error::Result;
use crate::models::discovery::reference::{ExchangeInfo, SymbolMatch};

pub(crate) fn to_symbol_match(dto: SymbolMatchDTO) -> SymbolMatch {
    SymbolMatch {
        symbol: dto.symbol,
        name: Some(dto.name),
        exchange: Some(dto.region),
        asset_type: Some(dto.asset_type),
        currency: Some(dto.currency),
        // SYMBOL_SEARCH only returns tradable symbols, so a hit is active.
        active: Some(true),
    }
}

/// Search Alpha Vantage's symbol universe.
pub async fn fetch_symbol_search_response(query: &str, limit: u32) -> Result<Vec<SymbolMatch>> {
    let matches = crate::adapters::alphavantage::quote::symbol_search(query).await?;
    Ok(matches
        .into_iter()
        .take(limit as usize)
        .map(to_symbol_match)
        .collect())
}

pub(crate) fn to_symbol_match_from_listing(dto: ListingEntryDTO) -> SymbolMatch {
    let active = dto
        .status
        .as_deref()
        .map(|s| s.eq_ignore_ascii_case("active"));
    SymbolMatch {
        symbol: dto.symbol,
        name: dto.name,
        exchange: dto.exchange,
        asset_type: dto.asset_type,
        currency: None,
        active,
    }
}

/// Derive the distinct venue list from the listing-status universe.
///
/// Alpha Vantage has no exchange endpoint; the listed-security CSV is the only
/// place it names venues. Only `name` can be populated — the CSV carries no
/// MIC, locale, or venue type.
pub(crate) fn listings_to_exchanges(listings: Vec<ListingEntryDTO>) -> Vec<ExchangeInfo> {
    let names: BTreeSet<String> = listings
        .into_iter()
        .filter_map(|l| l.exchange)
        .filter(|e| !e.trim().is_empty())
        .collect();

    names
        .into_iter()
        .map(|name| ExchangeInfo {
            name: Some(name),
            ..Default::default()
        })
        .collect()
}

/// Fetch the distinct exchanges represented in the active listing universe.
pub async fn fetch_exchanges_response() -> Result<Vec<ExchangeInfo>> {
    let listings =
        crate::adapters::alphavantage::fundamentals::listing_status(Some("active")).await?;
    Ok(listings_to_exchanges(listings))
}

/// Fetch the listed-security universe as symbol matches.
///
/// * `active` — `true` for currently listed securities, `false` for delisted.
pub async fn fetch_listing_status_response(active: bool) -> Result<Vec<SymbolMatch>> {
    let state = if active { "active" } else { "delisted" };
    let listings = crate::adapters::alphavantage::fundamentals::listing_status(Some(state)).await?;
    Ok(listings
        .into_iter()
        .map(to_symbol_match_from_listing)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing(symbol: &str, exchange: Option<&str>, status: &str) -> ListingEntryDTO {
        serde_json::from_value(serde_json::json!({
            "symbol": symbol,
            "name": format!("{symbol} Inc"),
            "exchange": exchange,
            "asset_type": "Stock",
            "ipo_date": "1999-01-01",
            "delisting_date": null,
            "status": status
        }))
        .unwrap()
    }

    #[test]
    fn maps_a_search_hit_onto_the_neutral_model() {
        let dto: SymbolMatchDTO = serde_json::from_value(serde_json::json!({
            "symbol": "TSCO.LON",
            "name": "Tesco PLC",
            "asset_type": "Equity",
            "region": "United Kingdom",
            "market_open": "08:00",
            "market_close": "16:30",
            "timezone": "UTC+01",
            "currency": "GBX",
            "match_score": 0.7273
        }))
        .unwrap();

        let out = to_symbol_match(dto);
        assert_eq!(out.symbol, "TSCO.LON");
        assert_eq!(out.name.as_deref(), Some("Tesco PLC"));
        assert_eq!(out.exchange.as_deref(), Some("United Kingdom"));
        assert_eq!(out.currency.as_deref(), Some("GBX"));
        assert_eq!(out.active, Some(true));
    }

    #[test]
    fn listing_status_maps_active_flag_case_insensitively() {
        assert_eq!(
            to_symbol_match_from_listing(listing("AAPL", Some("NASDAQ"), "Active")).active,
            Some(true)
        );
        assert_eq!(
            to_symbol_match_from_listing(listing("ENRN", Some("NYSE"), "delisted")).active,
            Some(false)
        );
    }

    #[test]
    fn exchanges_are_deduplicated_and_blank_entries_dropped() {
        let listings = vec![
            listing("AAPL", Some("NASDAQ"), "Active"),
            listing("MSFT", Some("NASDAQ"), "Active"),
            listing("IBM", Some("NYSE"), "Active"),
            listing("XXX", Some("  "), "Active"),
            listing("YYY", None, "Active"),
        ];

        let names: Vec<String> = listings_to_exchanges(listings)
            .into_iter()
            .filter_map(|e| e.name)
            .collect();
        assert_eq!(names, vec!["NASDAQ".to_string(), "NYSE".to_string()]);
    }
}

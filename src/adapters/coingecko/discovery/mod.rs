//! Symbol search (`/search`) — CoinGecko as an additional keyless route for
//! [`Capability::DISCOVERY`](crate::providers::Capability::DISCOVERY),
//! alongside FMP/Polygon/Alpha Vantage.

use crate::error::Result;
use crate::models::discovery::reference::SymbolMatch;

use super::models::SearchCoinDTO;

/// Convert one coin match into the canonical [`SymbolMatch`].
///
/// `symbol` carries the CoinGecko coin **id** (e.g. `"bitcoin"`) rather than
/// the ticker — that's the identifier
/// [`Providers::crypto`](crate::Providers::crypto) actually accepts, and
/// tickers collide across coins in a way ids don't. CoinGecko coins also
/// aren't tied to a single listing exchange or quote currency, so
/// `exchange`/`currency` stay `None`; every returned coin is presumed
/// tradable, so `active` is `Some(true)`.
fn to_symbol_match(dto: SearchCoinDTO) -> SymbolMatch {
    SymbolMatch {
        symbol: dto.id,
        name: Some(dto.name),
        exchange: None,
        asset_type: Some("Cryptocurrency".to_string()),
        currency: None,
        active: Some(true),
    }
}

/// Search CoinGecko's coin catalog by free-text query, as canonical
/// [`SymbolMatch`]es. `limit` truncates the result — CoinGecko's `/search`
/// takes no page size and returns every match in one response.
pub async fn fetch_symbol_search_response(query: &str, limit: u32) -> Result<Vec<SymbolMatch>> {
    let resp = super::client()?.search(query).await?;
    Ok(resp
        .coins
        .into_iter()
        .take(limit as usize)
        .map(to_symbol_match)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_a_search_hit_onto_the_neutral_model() {
        let dto = SearchCoinDTO {
            id: "bitcoin".to_string(),
            name: "Bitcoin".to_string(),
        };
        let out = to_symbol_match(dto);
        assert_eq!(out.symbol, "bitcoin");
        assert_eq!(out.name.as_deref(), Some("Bitcoin"));
        assert_eq!(out.asset_type.as_deref(), Some("Cryptocurrency"));
        assert_eq!(out.exchange, None);
        assert_eq!(out.active, Some(true));
    }

    #[test]
    fn search_response_parses_and_limit_truncates() {
        let resp: super::super::models::SearchResponseDTO =
            serde_json::from_value(serde_json::json!({
                "coins": [
                    { "id": "bitcoin", "name": "Bitcoin", "symbol": "btc", "market_cap_rank": 1 },
                    { "id": "bitcoin-cash", "name": "Bitcoin Cash", "symbol": "bch", "market_cap_rank": 20 }
                ],
                "exchanges": [],
                "icos": [],
                "categories": [],
                "nfts": []
            }))
            .unwrap();
        assert_eq!(resp.coins.len(), 2);
        let matches: Vec<_> = resp
            .coins
            .into_iter()
            .take(1)
            .map(to_symbol_match)
            .collect();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].symbol, "bitcoin");
    }
}

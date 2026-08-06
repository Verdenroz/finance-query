//! Symbol search (`/search`) — CoinGecko as an additional keyless route for
//! [`Capability::DISCOVERY`](crate::providers::Capability::DISCOVERY),
//! alongside FMP/Polygon/Alpha Vantage.

use crate::error::Result;
use crate::models::discovery::reference::SymbolMatch;

use super::models::SearchCoinDTO;

/// Convert one coin match into the canonical [`SymbolMatch`].
///
/// `symbol` is the ticker (uppercased), matching every other DISCOVERY
/// provider; the CoinGecko coin id goes in `id`, since that — not the ticker —
/// is what [`Providers::crypto`](crate::Providers::crypto) accepts. Coins
/// aren't tied to one listing exchange or quote currency, so
/// `exchange`/`currency` stay `None`; a returned coin is presumed tradable.
fn to_symbol_match(dto: SearchCoinDTO) -> SymbolMatch {
    SymbolMatch {
        // Fall back to the id only when CoinGecko omits the ticker outright —
        // `symbol` is non-optional and an empty string would be worse.
        symbol: dto
            .symbol
            .map(|s| s.to_ascii_uppercase())
            .unwrap_or_else(|| dto.id.clone()),
        id: Some(dto.id),
        name: Some(dto.name),
        exchange: None,
        asset_type: Some("Cryptocurrency".to_string()),
        currency: None,
        active: Some(true),
        market_cap_rank: dto.market_cap_rank,
        thumbnail: dto.thumb,
        image: dto.large,
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
            symbol: Some("btc".to_string()),
            market_cap_rank: Some(1),
            thumb: Some("https://example.com/thumb.png".to_string()),
            large: Some("https://example.com/large.png".to_string()),
        };
        let out = to_symbol_match(dto);
        // Ticker in `symbol`, CoinGecko id in `id` — matches every other
        // DISCOVERY provider's use of `symbol`.
        assert_eq!(out.symbol, "BTC");
        assert_eq!(out.id.as_deref(), Some("bitcoin"));
        assert_eq!(out.name.as_deref(), Some("Bitcoin"));
        assert_eq!(out.asset_type.as_deref(), Some("Cryptocurrency"));
        assert_eq!(out.exchange, None);
        assert_eq!(out.active, Some(true));
        assert_eq!(out.market_cap_rank, Some(1));
        assert_eq!(
            out.thumbnail.as_deref(),
            Some("https://example.com/thumb.png")
        );
        assert_eq!(out.image.as_deref(), Some("https://example.com/large.png"));
    }

    #[test]
    fn a_hit_without_a_ticker_falls_back_to_the_id() {
        let dto = SearchCoinDTO {
            id: "some-coin".to_string(),
            name: "Some Coin".to_string(),
            symbol: None,
            market_cap_rank: None,
            thumb: None,
            large: None,
        };
        let out = to_symbol_match(dto);
        assert_eq!(out.symbol, "some-coin");
        assert_eq!(out.id.as_deref(), Some("some-coin"));
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
        assert_eq!(matches[0].symbol, "BTC");
        assert_eq!(matches[0].id.as_deref(), Some("bitcoin"));
    }
}

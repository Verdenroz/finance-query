//! CoinGecko provider implementation.

use super::types;
use crate::error::Result;

pub(crate) struct CoinGeckoProvider;

#[async_trait::async_trait]
impl super::ProviderAdapter for CoinGeckoProvider {
    fn id(&self) -> &'static str {
        "coingecko"
    }
    fn name(&self) -> &'static str {
        "CoinGecko"
    }

    fn capabilities(&self) -> super::Capability {
        super::Capability::CRYPTO
    }

    async fn fetch_crypto_quote(
        &self,
        id: &str,
        vs_currency: &str,
    ) -> Result<types::CryptoQuoteData> {
        let quote = crate::adapters::coingecko::coin(id, vs_currency).await?;
        Ok(types::CryptoQuoteData {
            provider_id: "coingecko",
            id: quote.id,
            symbol: quote.symbol,
            name: quote.name,
            price: quote.current_price,
            market_cap: quote.market_cap,
            volume_24h: quote.total_volume,
            change_percent_24h: quote.price_change_percentage_24h,
            circulating_supply: quote.circulating_supply,
            ..Default::default()
        })
    }

    async fn fetch_crypto_coins(
        &self,
        vs_currency: &str,
        count: u32,
    ) -> Result<Vec<types::CryptoCoinData>> {
        let coins = crate::adapters::coingecko::coins(vs_currency, count as usize).await?;
        Ok(coins
            .into_iter()
            .map(|c| types::CryptoCoinData {
                provider_id: "coingecko",
                id: c.id,
                symbol: c.symbol,
                name: c.name,
                market_cap_rank: c.market_cap_rank,
                price: c.current_price,
                market_cap: c.market_cap,
                volume_24h: c.total_volume,
                change_percent_24h: c.price_change_percentage_24h,
                image_url: c.image,
                extras: Default::default(),
            })
            .collect())
    }
}

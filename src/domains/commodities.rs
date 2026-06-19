//! Commodity price quote handle.
//!
//! Created via [`Providers::commodity`](crate::Providers::commodity).

use crate::error::Result;

domain_handle! {
    /// A commodity backed by configured data providers.
    ///
    /// Created via [`Providers::commodity`](crate::Providers::commodity).
    pub struct Commodity { symbol, symbol }
    cache: crate::models::commodities::CommodityQuote
}

impl Commodity {
    /// Fetch the current quote for this commodity.
    pub async fn quote(&self) -> Result<crate::models::commodities::CommodityQuote> {
        fetch_via!(
            self,
            symbol,
            COMMODITIES,
            fetch_commodities_quote,
            crate::models::commodities::CommodityQuote
        )
    }
}

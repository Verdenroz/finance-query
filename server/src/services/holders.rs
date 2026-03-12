use crate::cache::{self, Cache};
use finance_query::Ticker;

use super::{ServiceError, ServiceResult};

/// Holder type identifiers matching the REST path param.
#[derive(Debug, Clone, Copy)]
pub enum HolderType {
    Major,
    Institutional,
    Mutualfund,
    InsiderTransactions,
    InsiderPurchases,
    InsiderRoster,
}

impl HolderType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "major" => Some(Self::Major),
            "institutional" => Some(Self::Institutional),
            "mutualfund" | "mutual-fund" => Some(Self::Mutualfund),
            "insider-transactions" => Some(Self::InsiderTransactions),
            "insider-purchases" => Some(Self::InsiderPurchases),
            "insider-roster" => Some(Self::InsiderRoster),
            _ => None,
        }
    }

    pub fn valid_types() -> &'static str {
        "major, institutional, mutualfund, insider-transactions, insider-purchases, insider-roster"
    }
}

pub async fn get_holders(
    cache: &Cache,
    symbol: &str,
    holder_type: HolderType,
    holder_type_str: &str,
) -> ServiceResult {
    let cache_key = Cache::key("holders", &[&symbol.to_uppercase(), holder_type_str]);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HOLDERS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let json: serde_json::Value = match holder_type {
                    HolderType::Major => {
                        let data = ticker.major_holders().await?;
                        serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)?
                    }
                    HolderType::Institutional => {
                        let data = ticker.institution_ownership().await?;
                        serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)?
                    }
                    HolderType::Mutualfund => {
                        let data = ticker.fund_ownership().await?;
                        serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)?
                    }
                    HolderType::InsiderTransactions => {
                        let data = ticker.insider_transactions().await?;
                        serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)?
                    }
                    HolderType::InsiderPurchases => {
                        let data = ticker.share_purchase_activity().await?;
                        serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)?
                    }
                    HolderType::InsiderRoster => {
                        let data = ticker.insider_holders().await?;
                        serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)?
                    }
                };
                Ok(json)
            },
        )
        .await
}

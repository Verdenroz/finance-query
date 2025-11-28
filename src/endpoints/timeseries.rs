/// Fundamentals timeseries endpoint
///
/// Fetches financial statement data over time (revenue, income, etc.).
use crate::client::YahooClient;
use crate::constants::{Frequency, api_params, endpoints};
use crate::error::Result;
use tracing::info;

/// Fetch fundamentals timeseries data (financial statements)
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol
/// * `period1` - Start Unix timestamp
/// * `period2` - End Unix timestamp
/// * `types` - List of fundamental types (e.g., "annualTotalRevenue", "quarterlyNetIncome")
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::timeseries;
/// let types = vec!["annualTotalRevenue", "annualNetIncome"];
/// let financials = timeseries::fetch(&client, "AAPL", 0, 9999999999, &types).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    symbol: &str,
    period1: i64,
    period2: i64,
    types: &[&str],
) -> Result<serde_json::Value> {
    super::common::validate_symbol(symbol)?;

    if types.is_empty() {
        return Err(crate::error::YahooError::InvalidParameter(
            "No types provided for timeseries".to_string(),
        ));
    }

    info!("Fetching fundamentals timeseries for: {}", symbol);

    let url = endpoints::financials(symbol);

    // Use client config for lang and region
    let config = client.config();
    let params = [
        ("merge", api_params::MERGE),
        ("padTimeSeries", api_params::PAD_TIMESERIES),
        ("period1", &period1.to_string()),
        ("period2", &period2.to_string()),
        ("type", &types.join(",")),
        ("lang", config.lang.as_str()),
        ("region", config.region.as_str()),
    ];
    let response = client.request_with_params(&url, &params).await?;

    Ok(response.json().await?)
}

/// Helper to build fundamental type strings with frequency prefix
///
/// # Arguments
///
/// * `frequency` - The frequency (Annual or Quarterly)
/// * `field_type` - The field type constant (e.g., from `fundamental_types`)
///
/// # Example
///
/// ```
/// use finance_query::endpoints::timeseries::build_fundamental_type;
/// use finance_query::constants::{Frequency, fundamental_types};
///
/// let field = build_fundamental_type(Frequency::Annual, fundamental_types::TOTAL_REVENUE);
/// assert_eq!(field, "annualTotalRevenue");
/// ```
pub fn build_fundamental_type(frequency: Frequency, field_type: &str) -> String {
    frequency.prefix(field_type)
}

/// Helper to build multiple fundamental types at once
///
/// # Arguments
///
/// * `frequency` - The frequency (Annual or Quarterly)
/// * `field_types` - Slice of field type constants
///
/// # Example
///
/// ```
/// use finance_query::endpoints::timeseries::build_fundamental_types;
/// use finance_query::constants::{Frequency, fundamental_types};
///
/// let fields = build_fundamental_types(
///     Frequency::Annual,
///     &[fundamental_types::TOTAL_REVENUE, fundamental_types::NET_INCOME]
/// );
/// assert_eq!(fields, vec!["annualTotalRevenue", "annualNetIncome"]);
/// ```
pub fn build_fundamental_types(frequency: Frequency, field_types: &[&str]) -> Vec<String> {
    field_types
        .iter()
        .map(|&field| build_fundamental_type(frequency, field))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_timeseries() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(
            &client,
            "AAPL",
            0,
            9999999999,
            &["annualTotalRevenue", "annualNetIncome"],
        )
        .await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("timeseries").is_some());
    }

    #[tokio::test]
    async fn test_empty_types() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "AAPL", 0, 9999999999, &[]).await;
        assert!(result.is_err());
    }
}

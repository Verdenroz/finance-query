use super::urls::builders;
use crate::client::YahooClient;
use crate::constants::screener_types::ScreenerType;
use crate::error::Result;
use crate::models::screeners::{ScreenerQuery, ScreenerResults};

/// Fetch data from a predefined Yahoo Finance screener
///
/// Returns a flattened, user-friendly response structure with quotes
/// matching the screener criteria.
///
/// # Arguments
///
/// * `client` - Yahoo Finance client
/// * `screener_type` - The predefined screener type to use
/// * `count` - Number of results to return (max 250)
///
/// # Example
///
/// ```ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::ScreenerType;
/// let result = client.get_screener(ScreenerType::MostActives, 25).await?;
/// println!("Screener type: {}", result.screener_type);
/// for quote in &result.quotes {
///     println!("  {} - {}", quote.symbol, quote.short_name);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    screener_type: ScreenerType,
    count: u32,
) -> Result<ScreenerResults> {
    let url = builders::screener(screener_type, count);
    let response = client.request_with_crumb(&url).await?;
    let json: serde_json::Value = response.json().await?;

    // Parse and flatten Yahoo Finance response internally
    parse_screeners_response(&json)
}

/// Fetch data using a custom screener query
///
/// Allows flexible filtering of stocks/funds/ETFs based on various criteria.
/// Uses POST request with JSON body.
///
/// # Arguments
///
/// * `client` - Yahoo Finance client
/// * `query` - The custom screener query to execute
///
/// # Example
///
/// ```ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::screener_query::{ScreenerQuery, QueryCondition, Operator};
///
/// let query = ScreenerQuery::new()
///     .size(25)
///     .sort_by("intradaymarketcap", false)
///     .add_condition(QueryCondition::new("region", Operator::Eq).value_str("us"))
///     .add_condition(QueryCondition::new("avgdailyvol3m", Operator::Gt).value(200000));
///
/// let result = client.custom_screener(query).await?;
/// for quote in &result.quotes {
///     println!("  {} - {}", quote.symbol, quote.short_name);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn fetch_custom(client: &YahooClient, query: ScreenerQuery) -> Result<ScreenerResults> {
    let url = builders::custom_screener();
    let response = client.request_post_with_crumb(&url, &query).await?;
    let json: serde_json::Value = response.json().await?;

    // Parse and flatten Yahoo Finance custom screener response
    parse_custom_screeners_response(&json)
}

/// Parse Yahoo Finance screener response into clean ScreenerResults
///
/// Handles all Yahoo-specific nested structure and data transformation internally.
fn parse_screeners_response(json: &serde_json::Value) -> Result<ScreenerResults> {
    ScreenerResults::from_response(json).map_err(|e| {
        crate::error::YahooError::ResponseStructureError {
            field: "screeners".to_string(),
            context: e,
        }
    })
}

/// Parse Yahoo Finance custom screener response into clean ScreenerResults
fn parse_custom_screeners_response(json: &serde_json::Value) -> Result<ScreenerResults> {
    ScreenerResults::from_custom_response(json).map_err(|e| {
        crate::error::YahooError::ResponseStructureError {
            field: "custom_screener".to_string(),
            context: e,
        }
    })
}

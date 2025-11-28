use crate::client::YahooClient;
use crate::error::Result;

/// Fetch earnings call transcript
///
/// # Arguments
///
/// * `client` - Yahoo Finance client
/// * `event_id` - Event ID for the earnings call
/// * `company_id` - Company ID (quartrId from quote_type endpoint)
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::earnings_transcript;
/// let transcript = earnings_transcript::fetch(&client, "12345", "0P00000000").await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    event_id: &str,
    company_id: &str,
) -> Result<serde_json::Value> {
    let url = "https://finance.yahoo.com/xhr/transcript";

    let params = [
        ("eventType", "earnings_call"),
        ("quartrId", company_id),
        ("eventId", event_id),
        ("lang", &client.config().lang),
        ("region", &client.config().region),
    ];

    let response = client.request_with_params(url, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access and valid event_id/company_id
    async fn test_fetch_earnings_transcript() {
        let _client = YahooClient::new(ClientConfig::default()).await.unwrap();
        // Note: This test requires valid event_id and company_id
        // let result = fetch(&_client, "12345", "0P00000000").await;
        // assert!(result.is_ok());
    }
}

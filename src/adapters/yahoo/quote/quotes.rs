/// Batch quotes endpoint
///
/// Fetches basic quote data for multiple symbols in a single request.
/// This uses the /v7/finance/quote endpoint which is more efficient for batch requests
/// than calling quoteSummary for each symbol individually.
use crate::adapters::yahoo::client::YahooClient;
use crate::adapters::yahoo::endpoints::api;
use crate::error::Result;
use crate::models::quote::{FormattedValue, Price, QuoteSummaryResponse};
use tracing::info;

/// Fetch batch quotes for multiple symbols
///
/// This endpoint returns basic quote data (price, volume, market cap, etc.) for multiple
/// symbols in a single API call. It's more efficient than quoteSummary for batch requests.
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbols` - Array of stock symbols to fetch quotes for
///
/// # Example
///
/// ```ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::quotes;
/// let quotes = quotes::fetch(&client, &["AAPL", "GOOGL", "MSFT"]).await?;
/// # Ok(())
/// # }
/// ```
#[allow(dead_code)] // raw-JSON variant kept alongside the typed fetch; no current caller
pub(crate) async fn fetch(client: &YahooClient, symbols: &[&str]) -> Result<serde_json::Value> {
    crate::adapters::yahoo::common::validate_symbols(symbols)?;

    info!("Fetching batch quotes for {} symbols", symbols.len());

    let params = [("symbols", symbols.join(","))];
    let response = client.request_with_params(api::QUOTES, &params).await?;

    Ok(response.json().await?)
}

/// Fetch batch quotes with custom fields and options
///
/// This advanced version allows you to specify which fields to fetch and whether to include
/// logo URLs with specific dimensions. Supports selective field fetching for efficiency.
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbols` - Array of stock symbols to fetch quotes for
/// * `fields` - Optional list of specific fields to fetch (e.g., ["logoUrl", "regularMarketPrice"])
/// * `formatted` - Whether to return formatted values (e.g., "102,05 %" vs 1.0205495)
/// * `include_logo` - Whether to include logo URLs with 50x50 dimensions
///
/// # Available Fields
///
/// Common fields include: `logoUrl`, `companyLogoUrl`, `longName`, `shortName`,
/// `regularMarketPrice`, `regularMarketChange`, `regularMarketChangePercent`,
/// `regularMarketVolume`, `marketCap`, `fiftyTwoWeekHigh`, `fiftyTwoWeekLow`,
/// `preMarketPrice`, `postMarketPrice`, `quartrId`, and many more.
///
/// # Example
///
/// ```ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::quotes;
///
/// // Fetch specific fields with logos
/// let fields = vec!["logoUrl", "longName", "regularMarketPrice", "marketCap"];
/// let quotes = quotes::fetch_with_fields(
///     &client,
///     &["AAPL", "TSLA", "NVDA"],
///     Some(&fields),
///     true,  // formatted values
///     true   // include logos
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub(crate) async fn fetch_with_fields(
    client: &YahooClient,
    symbols: &[&str],
    fields: Option<&[&str]>,
    formatted: bool,
    include_logo: bool,
) -> Result<serde_json::Value> {
    crate::adapters::yahoo::common::validate_symbols(symbols)?;

    info!(
        "Fetching batch quotes for {} symbols with custom fields (formatted={}, include_logo={})",
        symbols.len(),
        formatted,
        include_logo
    );

    // Get client config for lang and region (read once)
    let config = client.config();

    // Build parameters — static string literals avoid per-call allocations
    let mut params: Vec<(&str, std::borrow::Cow<str>)> = vec![
        ("symbols", symbols.join(",").into()),
        (
            "formatted",
            if formatted {
                "true".into()
            } else {
                "false".into()
            },
        ),
    ];

    // Add fields if specified
    if let Some(field_list) = fields {
        params.push(("fields", field_list.join(",").into()));
    }

    // Add logo parameters if requested
    if include_logo {
        params.push(("imgHeights", "50".into()));
        params.push(("imgWidths", "50".into()));
        params.push(("imgLabels", "logoUrl".into()));
    }

    // Add overnight price support
    params.push(("overnightPrice", "true".into()));

    params.push(("lang", (&*config.lang).into()));
    params.push(("region", (&*config.region).into()));

    let response = client.request_with_params(api::QUOTES, &params).await?;

    Ok(response.json().await?)
}

/// Same request as [`fetch`], but hands back the raw body so callers can
/// deserialize straight into a typed struct instead of via `serde_json::Value`.
async fn fetch_bytes(
    client: &YahooClient,
    symbols: &[&str],
) -> Result<impl std::ops::Deref<Target = [u8]>> {
    crate::adapters::yahoo::common::validate_symbols(symbols)?;

    info!("Fetching batch quotes for {} symbols", symbols.len());

    let params = [("symbols", symbols.join(","))];
    let response = client.request_with_params(api::QUOTES, &params).await?;

    Ok(response.bytes().await?)
}

/// Generates a field deserializer that mirrors a `serde_json::Value` accessor:
/// any JSON type the accessor rejects yields `None` rather than a
/// deserialization error, because Yahoo returns the same field as a bare
/// number, a quoted string, or a `{"raw":..,"fmt":..}` object depending on the
/// endpoint, and one odd field must not fail the whole batch.
macro_rules! lenient_accessor {
    ($name:ident, $out:ty, $expecting:literal, $($accept:item),* $(,)?) => {
        fn $name<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Option<$out>, D::Error> {
            struct V;
            impl<'de> serde::de::Visitor<'de> for V {
                type Value = Option<$out>;
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str($expecting)
                }
                $($accept)*
                fn visit_bool<E>(self, _: bool) -> std::result::Result<Self::Value, E> {
                    Ok(None)
                }
                fn visit_unit<E>(self) -> std::result::Result<Self::Value, E> {
                    Ok(None)
                }
                fn visit_none<E>(self) -> std::result::Result<Self::Value, E> {
                    Ok(None)
                }
                fn visit_some<D2: serde::Deserializer<'de>>(
                    self,
                    d: D2,
                ) -> std::result::Result<Self::Value, D2::Error> {
                    d.deserialize_any(V)
                }
                fn visit_map<A: serde::de::MapAccess<'de>>(
                    self,
                    mut m: A,
                ) -> std::result::Result<Self::Value, A::Error> {
                    while m
                        .next_entry::<serde::de::IgnoredAny, serde::de::IgnoredAny>()?
                        .is_some()
                    {}
                    Ok(None)
                }
                fn visit_seq<A: serde::de::SeqAccess<'de>>(
                    self,
                    mut s: A,
                ) -> std::result::Result<Self::Value, A::Error> {
                    while s.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                    Ok(None)
                }
            }
            d.deserialize_any(V)
        }
    };
}

lenient_accessor!(
    lenient_f64,
    f64,
    "a number or any other JSON value",
    fn visit_f64<E>(self, v: f64) -> std::result::Result<Self::Value, E> {
        Ok(Some(v))
    },
    fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E> {
        Ok(Some(v as f64))
    },
    fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E> {
        Ok(Some(v as f64))
    },
    fn visit_str<E>(self, _: &str) -> std::result::Result<Self::Value, E> {
        Ok(None)
    }
);

lenient_accessor!(
    lenient_i64,
    i64,
    "an integer or any other JSON value",
    // `Number::as_i64` returns `None` for every `N::Float`, even an integral
    // one like `2.0`, so this arm is unconditionally `None`.
    fn visit_f64<E>(self, _: f64) -> std::result::Result<Self::Value, E> {
        Ok(None)
    },
    fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E> {
        Ok(Some(v))
    },
    fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E> {
        Ok(i64::try_from(v).ok())
    },
    fn visit_str<E>(self, _: &str) -> std::result::Result<Self::Value, E> {
        Ok(None)
    }
);

lenient_accessor!(
    lenient_string,
    String,
    "a string or any other JSON value",
    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E> {
        Ok(Some(v.to_owned()))
    },
    fn visit_f64<E>(self, _: f64) -> std::result::Result<Self::Value, E> {
        Ok(None)
    },
    fn visit_i64<E>(self, _: i64) -> std::result::Result<Self::Value, E> {
        Ok(None)
    },
    fn visit_u64<E>(self, _: u64) -> std::result::Result<Self::Value, E> {
        Ok(None)
    }
);

/// Mirrors `as_str().unwrap_or("")` for the symbol key.
fn lenient_symbol<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    Ok(lenient_string(d)?.unwrap_or_default())
}

/// Missing and explicitly-null containers both collapse to the default, matching
/// the chained `.get(..).and_then(..)` the `Value` walk used to do.
fn null_to_default<'de, D, T>(d: D) -> std::result::Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    Ok(<Option<T> as serde::Deserialize>::deserialize(d)?.unwrap_or_default())
}

#[derive(Default, serde::Deserialize)]
struct QuotesEnvelope {
    #[serde(
        rename = "quoteResponse",
        default,
        deserialize_with = "null_to_default"
    )]
    quote_response: QuotesResult,
}

#[derive(Default, serde::Deserialize)]
struct QuotesResult {
    #[serde(default, deserialize_with = "null_to_default")]
    result: Vec<QuoteRow>,
}

#[derive(Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct QuoteRow {
    #[serde(deserialize_with = "lenient_symbol")]
    symbol: String,
    #[serde(deserialize_with = "lenient_string")]
    short_name: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    long_name: Option<String>,
    #[serde(rename = "fullExchangeName", deserialize_with = "lenient_string")]
    exchange_name: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    exchange: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    quote_type: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    currency: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    market_state: Option<String>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_price: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_change: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_change_percent: Option<f64>,
    #[serde(deserialize_with = "lenient_i64")]
    regular_market_volume: Option<i64>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_previous_close: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_open: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_day_high: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_day_low: Option<f64>,
    #[serde(deserialize_with = "lenient_i64")]
    market_cap: Option<i64>,
}

/// Fetch batch quotes and convert to canonical `(symbol, QuoteSummaryResponse)` pairs.
///
/// The batch endpoint returns basic fields only (price module), not full quoteSummary data.
/// This function constructs partial `QuoteSummaryResponse` objects from the batch response.
pub(crate) async fn fetch_quotes_batch(
    client: &YahooClient,
    symbols: &[&str],
) -> Result<Vec<(String, QuoteSummaryResponse)>> {
    let body = fetch_bytes(client, symbols).await?;
    let envelope: QuotesEnvelope = serde_json::from_slice(&body)?;
    let rows = envelope.quote_response.result;

    let mut quotes = Vec::with_capacity(rows.len());
    for row in rows {
        let price = Price {
            short_name: row.short_name,
            long_name: row.long_name,
            exchange_name: row.exchange_name,
            exchange: row.exchange,
            quote_type: row.quote_type,
            currency: row.currency,
            market_state: row.market_state,
            regular_market_price: row.regular_market_price.map(FormattedValue::new),
            regular_market_change: row.regular_market_change.map(FormattedValue::new),
            regular_market_change_percent: row
                .regular_market_change_percent
                .map(FormattedValue::new),
            regular_market_volume: row.regular_market_volume.map(FormattedValue::new),
            regular_market_previous_close: row
                .regular_market_previous_close
                .map(FormattedValue::new),
            regular_market_open: row.regular_market_open.map(FormattedValue::new),
            regular_market_day_high: row.regular_market_day_high.map(FormattedValue::new),
            regular_market_day_low: row.regular_market_day_low.map(FormattedValue::new),
            market_cap: row.market_cap.map(FormattedValue::new),
            ..Default::default()
        };
        let response = QuoteSummaryResponse {
            symbol: row.symbol.clone(),
            price: Some(price),
            ..Default::default()
        };
        quotes.push((row.symbol, response));
    }
    Ok(quotes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::yahoo::client::ClientConfig;

    #[test]
    fn typed_rows_reproduce_value_accessor_tolerance() {
        let body = r#"{"quoteResponse":{"result":[
            {"symbol":"AAPL","shortName":"Apple","regularMarketPrice":150.5,"marketCap":2500000000},
            {"symbol":"STRNUM","regularMarketPrice":"150.5","marketCap":"2500000000"},
            {"symbol":"OBJ","regularMarketPrice":{"raw":150.5,"fmt":"150.50"}},
            {"symbol":"NULLS","shortName":null,"regularMarketPrice":null},
            {"symbol":"BARE"},
            {"symbol":"WRONGTYPE","shortName":42,"regularMarketPrice":true}
        ]}}"#;
        let env: QuotesEnvelope = serde_json::from_str(body).unwrap();
        let rows = &env.quote_response.result;
        assert_eq!(rows.len(), 6);

        assert_eq!(rows[0].short_name.as_deref(), Some("Apple"));
        assert_eq!(rows[0].regular_market_price, Some(150.5));
        assert_eq!(rows[0].market_cap, Some(2_500_000_000));

        assert_eq!(rows[1].regular_market_price, None);
        assert_eq!(rows[1].market_cap, None);

        assert_eq!(rows[2].regular_market_price, None);

        assert_eq!(rows[3].short_name, None);
        assert_eq!(rows[3].regular_market_price, None);
        assert_eq!(rows[4].short_name, None);
        assert_eq!(rows[4].regular_market_price, None);
        assert_eq!(rows[5].short_name, None);
        assert_eq!(rows[5].regular_market_price, None);

        assert_eq!(rows[5].symbol, "WRONGTYPE");
    }

    #[test]
    fn lenient_i64_matches_as_i64_on_floats() {
        let body = r#"{"quoteResponse":{"result":[
            {"symbol":"F","marketCap":2.0,"regularMarketVolume":1e3}
        ]}}"#;
        let env: QuotesEnvelope = serde_json::from_str(body).unwrap();
        let row = &env.quote_response.result[0];
        assert_eq!(row.market_cap, None);
        assert_eq!(row.regular_market_volume, None);
    }

    #[test]
    fn missing_and_malformed_containers_do_not_error() {
        let env: QuotesEnvelope = serde_json::from_str(r#"{"quoteResponse":{}}"#).unwrap();
        assert!(env.quote_response.result.is_empty());

        let body = r#"{"quoteResponse":{"result":[
            {"symbol":"S","shortName":["a","b"],"regularMarketVolume":{"raw":5,"fmt":"5"},
             "longName":{"x":{"y":[1,2,{"z":null}]}},"currency":1.5}
        ]}}"#;
        let env: QuotesEnvelope = serde_json::from_str(body).unwrap();
        let row = &env.quote_response.result[0];
        assert_eq!(row.symbol, "S");
        assert_eq!(row.short_name, None);
        assert_eq!(row.long_name, None);
        assert_eq!(row.currency, None);
        assert_eq!(row.regular_market_volume, None);
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_batch_quotes() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, &["AAPL", "GOOGL"]).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("quoteResponse").is_some());
    }

    #[tokio::test]
    #[ignore = "requires network access - validation tested in common::tests"]
    async fn test_empty_symbols() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, &[]).await;
        assert!(result.is_err());
    }
}

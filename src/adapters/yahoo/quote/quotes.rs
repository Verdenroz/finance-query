use crate::adapters::yahoo::client::YahooClient;
use crate::adapters::yahoo::endpoints::api;
use crate::error::Result;
/// Batch quotes endpoint
///
/// Fetches basic quote data for multiple symbols in a single request.
/// This uses the /v7/finance/quote endpoint which is more efficient for batch requests
/// than calling quoteSummary for each symbol individually.
use crate::init;
use crate::models::fundamentals::{DefaultKeyStatistics, SummaryDetail};
use crate::models::quote::{FormattedValue, Price, QuoteSummaryResponse, QuoteTypeData};
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
    #[serde(deserialize_with = "lenient_string")]
    quote_source_name: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    underlying_symbol: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    exchange_timezone_name: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    exchange_timezone_short_name: Option<String>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_price: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_change: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_change_percent: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_previous_close: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_open: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_day_high: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    regular_market_day_low: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    pre_market_price: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    pre_market_change: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    pre_market_change_percent: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    post_market_price: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    post_market_change: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    post_market_change_percent: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    fifty_two_week_high: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    fifty_two_week_low: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    fifty_two_week_change_percent: Option<f64>,
    #[serde(rename = "trailingPE", deserialize_with = "lenient_f64")]
    trailing_pe: Option<f64>,
    #[serde(rename = "forwardPE", deserialize_with = "lenient_f64")]
    forward_pe: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    fifty_day_average: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    two_hundred_day_average: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    dividend_rate: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    dividend_yield: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    trailing_annual_dividend_rate: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    trailing_annual_dividend_yield: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    bid: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    ask: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    price_to_book: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    book_value: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    eps_trailing_twelve_months: Option<f64>,
    #[serde(deserialize_with = "lenient_f64")]
    eps_forward: Option<f64>,
    #[serde(deserialize_with = "lenient_i64")]
    regular_market_volume: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    regular_market_time: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    pre_market_time: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    post_market_time: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    market_cap: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    average_daily_volume3_month: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    average_daily_volume10_day: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    exchange_data_delayed_by: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    price_hint: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    bid_size: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    ask_size: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    shares_outstanding: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    implied_shares_outstanding: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    gmt_off_set_milliseconds: Option<i64>,
    #[serde(deserialize_with = "lenient_i64")]
    first_trade_date_milliseconds: Option<i64>,
}

fn formatted<T>(value: Option<T>) -> Option<FormattedValue<T>> {
    value.map(FormattedValue::new)
}

fn fraction_from_percent(value: Option<f64>) -> Option<f64> {
    value.map(|v| v / 100.0)
}

fn row_to_response(row: QuoteRow) -> (String, QuoteSummaryResponse) {
    let price = init!(Price {
        short_name: row.short_name,
        long_name: row.long_name,
        exchange_name: row.exchange_name,
        exchange: row.exchange,
        quote_type: row.quote_type,
        underlying_symbol: row.underlying_symbol,
        quote_source_name: row.quote_source_name,
        currency: row.currency,
        market_state: row.market_state,
        exchange_data_delayed_by: row
            .exchange_data_delayed_by
            .and_then(|v| i32::try_from(v).ok()),
        price_hint: formatted(row.price_hint),
        regular_market_price: formatted(row.regular_market_price),
        regular_market_change: formatted(row.regular_market_change),
        regular_market_change_percent: formatted(row.regular_market_change_percent),
        regular_market_time: row.regular_market_time,
        regular_market_volume: formatted(row.regular_market_volume),
        regular_market_previous_close: formatted(row.regular_market_previous_close),
        regular_market_open: formatted(row.regular_market_open),
        regular_market_day_high: formatted(row.regular_market_day_high),
        regular_market_day_low: formatted(row.regular_market_day_low),
        pre_market_price: formatted(row.pre_market_price),
        pre_market_change: formatted(row.pre_market_change),
        pre_market_change_percent: formatted(row.pre_market_change_percent),
        pre_market_time: row.pre_market_time,
        post_market_price: formatted(row.post_market_price),
        post_market_change: formatted(row.post_market_change),
        post_market_change_percent: formatted(row.post_market_change_percent),
        post_market_time: row.post_market_time,
        average_daily_volume3_month: formatted(row.average_daily_volume3_month),
        average_daily_volume10_day: formatted(row.average_daily_volume10_day),
        market_cap: formatted(row.market_cap),
        ..Default::default()
    });
    let summary_detail = init!(SummaryDetail {
        day_high: formatted(row.regular_market_day_high),
        day_low: formatted(row.regular_market_day_low),
        open: formatted(row.regular_market_open),
        previous_close: formatted(row.regular_market_previous_close),
        volume: formatted(row.regular_market_volume),
        bid: formatted(row.bid),
        bid_size: formatted(row.bid_size),
        ask: formatted(row.ask),
        ask_size: formatted(row.ask_size),
        fifty_two_week_high: formatted(row.fifty_two_week_high),
        fifty_two_week_low: formatted(row.fifty_two_week_low),
        fifty_day_average: formatted(row.fifty_day_average),
        two_hundred_day_average: formatted(row.two_hundred_day_average),
        trailing_pe: formatted(row.trailing_pe),
        forward_pe: formatted(row.forward_pe),
        dividend_rate: formatted(row.dividend_rate),
        dividend_yield: formatted(fraction_from_percent(row.dividend_yield)),
        trailing_annual_dividend_rate: formatted(row.trailing_annual_dividend_rate),
        trailing_annual_dividend_yield: formatted(row.trailing_annual_dividend_yield),
        ..Default::default()
    });
    let key_statistics = init!(DefaultKeyStatistics {
        price_to_book: formatted(row.price_to_book),
        book_value: formatted(row.book_value),
        trailing_eps: formatted(row.eps_trailing_twelve_months),
        forward_eps: formatted(row.eps_forward),
        shares_outstanding: formatted(row.shares_outstanding),
        implied_shares_outstanding: formatted(row.implied_shares_outstanding),
        week_52_change: formatted(fraction_from_percent(row.fifty_two_week_change_percent)),
        ..Default::default()
    });
    let quote_type = init!(QuoteTypeData {
        time_zone_full_name: row.exchange_timezone_name,
        time_zone_short_name: row.exchange_timezone_short_name,
        gmt_off_set_milliseconds: row.gmt_off_set_milliseconds,
        first_trade_date_epoch_utc: row.first_trade_date_milliseconds.map(|ms| ms / 1000),
        ..Default::default()
    });
    let response = init!(QuoteSummaryResponse {
        symbol: row.symbol.clone(),
        price: Some(price),
        summary_detail: Some(summary_detail),
        default_key_statistics: Some(key_statistics),
        quote_type: Some(quote_type),
        ..Default::default()
    });
    (row.symbol, response)
}

/// Fetch batch quotes and convert to canonical `(symbol, QuoteSummaryResponse)` pairs.
///
/// The batch endpoint carries the price, valuation, 52-week and dividend fields but no
/// profile or risk data. Rows are returned in the order Yahoo sends them; a dividend yield
/// arrives in percent and a 52-week change in percent, and both are stored as fractions to
/// match quoteSummary.
pub(crate) async fn fetch_quotes_batch(
    client: &YahooClient,
    symbols: &[&str],
) -> Result<Vec<(String, QuoteSummaryResponse)>> {
    let body = fetch_bytes(client, symbols).await?;
    parse_quotes_batch(&body)
}

pub(crate) fn parse_quotes_batch(body: &[u8]) -> Result<Vec<(String, QuoteSummaryResponse)>> {
    let envelope: QuotesEnvelope = serde_json::from_slice(body)?;
    Ok(envelope
        .quote_response
        .result
        .into_iter()
        .map(row_to_response)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::yahoo::client::ClientConfig;
    use crate::models::quote::Quote;

    const BATCH_FIXTURE: &[u8] =
        include_bytes!("../../../../tests/fixtures/yahoo/batch_quote.json");

    fn fixture_quotes() -> Vec<(String, Quote)> {
        parse_quotes_batch(BATCH_FIXTURE)
            .unwrap()
            .into_iter()
            .map(|(symbol, response)| (symbol, Quote::from_response(&response, None, None)))
            .collect()
    }

    fn raw(value: &Option<FormattedValue<f64>>) -> f64 {
        value.as_ref().and_then(|v| v.raw).unwrap()
    }

    #[test]
    fn batch_rows_keep_yahoo_order() {
        let symbols: Vec<String> = fixture_quotes().into_iter().map(|(s, _)| s).collect();
        assert_eq!(symbols, ["AAPL", "SPY", "7203.T"]);
    }

    #[test]
    fn batch_quote_populates_price_valuation_and_range_fields() {
        let quotes = fixture_quotes();
        let aapl = &quotes[0].1;

        assert_eq!(aapl.regular_market_time, Some(1794603601));
        assert_eq!(raw(&aapl.fifty_two_week_high), 360.83);
        assert_eq!(raw(&aapl.fifty_two_week_low), 243.42);
        assert!((raw(&aapl.trailing_pe) - 39.67372).abs() < 1e-6);
        assert!((raw(&aapl.forward_pe) - 33.700924).abs() < 1e-6);
        assert!((raw(&aapl.price_to_book) - 46.190845).abs() < 1e-6);
        assert_eq!(raw(&aapl.book_value), 7.713);
        assert_eq!(raw(&aapl.trailing_eps), 8.98);
        assert_eq!(raw(&aapl.forward_eps), 10.57152);
        assert_eq!(raw(&aapl.fifty_day_average), 339.0364);
        assert_eq!(raw(&aapl.two_hundred_day_average), 301.2529);
        assert_eq!(raw(&aapl.bid), 355.03);
        assert_eq!(raw(&aapl.dividend_rate), 1.08);
        assert_eq!(
            aapl.market_cap.as_ref().and_then(|v| v.raw),
            Some(5_177_380_175_872)
        );
        assert_eq!(
            aapl.average_daily_volume3_month
                .as_ref()
                .and_then(|v| v.raw),
            Some(42_760_568)
        );
        assert_eq!(
            aapl.shares_outstanding.as_ref().and_then(|v| v.raw),
            Some(14_532_182_000)
        );
        assert_eq!(raw(&aapl.post_market_price), 355.3849);
        assert_eq!(aapl.exchange_data_delayed_by, Some(15));
        assert_eq!(
            aapl.time_zone_full_name.as_deref(),
            Some("America/New_York")
        );
    }

    #[test]
    fn batch_quote_reports_percent_and_fraction_fields_like_quote_summary() {
        let quotes = fixture_quotes();
        let aapl = &quotes[0].1;

        assert!((raw(&aapl.regular_market_change_percent) - 0.3916772).abs() < 1e-9);
        assert!((raw(&aapl.post_market_change_percent) + 0.24843556).abs() < 1e-9);
        assert!((raw(&aapl.dividend_yield) - 0.003).abs() < 1e-9);
        assert!((raw(&aapl.trailing_annual_dividend_yield) - 0.002986925).abs() < 1e-9);
        assert!((raw(&aapl.week_52_change) - 0.3320496).abs() < 1e-9);
    }

    #[test]
    fn batch_quote_leaves_profile_and_risk_fields_empty() {
        let quotes = fixture_quotes();
        let aapl = &quotes[0].1;

        assert!(aapl.sector.is_none());
        assert!(aapl.industry.is_none());
        assert!(aapl.beta.is_none());
    }

    #[test]
    fn batch_quote_handles_symbols_without_equity_fields() {
        let quotes = fixture_quotes();
        let spy = &quotes[1].1;

        assert!(spy.market_cap.is_none());
        assert_eq!(spy.regular_market_time, Some(1794603600));
        assert!((raw(&spy.trailing_pe) - 23.999777).abs() < 1e-6);
        assert_eq!(quotes[2].1.currency.as_deref(), Some("JPY"));
    }

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

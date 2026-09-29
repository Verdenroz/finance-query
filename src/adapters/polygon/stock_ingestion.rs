//! Complete stock-directory and price pages for durable consumers.

use super::{
    build_client,
    client::PathRule,
    discovery::{TickerDetailsResponseDTO, details_to_canonical},
    models::AggregateResponseDTO,
};
use crate::{
    FinanceError, PageCursor, Provider, ProviderPage, Result, SortOrder, StockBar,
    StockBarsRequest, StockListing, StockListingRequest, StockType, SymbolDetails,
};
use serde::Deserialize;

fn invalid(field: &str) -> FinanceError {
    FinanceError::ResponseStructureError {
        field: field.into(),
        context: "invalid stock provider response".into(),
    }
}

fn validate_delisting(value: Option<&str>) -> Result<()> {
    if let Some(value) = value
        && chrono::DateTime::parse_from_rfc3339(value).is_err()
        && crate::models::discovery::listings::date(value).is_err()
    {
        return Err(invalid("delisted_utc"));
    }
    Ok(())
}

#[derive(Deserialize)]
struct Listing {
    ticker: String,
    name: Option<String>,
    primary_exchange: Option<String>,
    #[serde(rename = "type")]
    stock_type: Option<String>,
    locale: Option<String>,
    active: Option<bool>,
    cik: Option<String>,
    composite_figi: Option<String>,
    share_class_figi: Option<String>,
    list_date: Option<String>,
    delisted_utc: Option<String>,
}

pub(crate) async fn listings(
    request: &StockListingRequest,
    cursor: Option<&PageCursor>,
) -> Result<ProviderPage<StockListing>> {
    if request.locale.is_empty() || request.stock_type.as_ref().is_some_and(|s| s.is_empty()) {
        return Err(FinanceError::InvalidParameter {
            param: "stock_listings".into(),
            reason: "locale and stock type must not be empty".into(),
        });
    }
    let client = build_client()?;
    let identity = serde_json::to_string(request)?;
    let active = request.active.to_string();
    let limit = request.limit.to_string();
    let mut params = vec![
        ("market", "stocks"),
        ("locale", request.locale.as_str()),
        ("active", active.as_str()),
        ("date", request.date.as_str()),
        ("limit", limit.as_str()),
        ("sort", "ticker"),
        ("order", "asc"),
    ];
    if let Some(kind) = &request.stock_type {
        params.push(("type", kind.as_str()));
    }
    let path = "/v3/reference/tickers";
    let (body, current): (super::models::PaginatedResponseDTO<Listing>, _) = client
        .page(
            path,
            &params,
            PathRule::Exact,
            cursor,
            "stock_listings_page",
            &identity,
        )
        .await?;
    let rows = body.results.ok_or_else(|| invalid("results"))?;
    let items = rows
        .into_iter()
        .map(|row| {
            if row.ticker.is_empty() || row.active.is_some_and(|active| active != request.active) {
                return Err(invalid("listing"));
            }
            if let Some(date) = &row.list_date {
                crate::models::discovery::listings::date(date).map_err(|_| invalid("list_date"))?;
            }
            validate_delisting(row.delisted_utc.as_deref())?;
            if row
                .locale
                .as_ref()
                .is_some_and(|locale| locale != &request.locale)
                || row
                    .stock_type
                    .as_ref()
                    .zip(request.stock_type.as_ref())
                    .is_some_and(|(actual, expected)| actual != expected)
            {
                return Err(invalid("listing filters"));
            }
            Ok(StockListing {
                symbol: row.ticker,
                name: row.name,
                exchange: row.primary_exchange,
                stock_type: row.stock_type,
                locale: row.locale,
                active: row.active,
                cik: row.cik,
                composite_figi: row.composite_figi,
                share_class_figi: row.share_class_figi,
                list_date: row.list_date,
                delisted_utc: row.delisted_utc,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let next = client.continuation(
        body.next_url,
        path,
        &params,
        PathRule::Exact,
        "stock_listings_page",
        &identity,
        &current,
    )?;
    Ok(ProviderPage {
        items,
        next,
        provider_id: Provider::Polygon,
        request_id: body.request_id,
        results_count: body.results_count,
        query_count: body.query_count,
        reported_symbol: body.ticker,
        adjusted: body.adjusted,
    })
}

pub(crate) async fn details_at(symbol: &str, date: &str) -> Result<SymbolDetails> {
    crate::models::discovery::listings::date(date)?;
    if symbol.is_empty() || symbol == "." || symbol == ".." {
        return Err(FinanceError::InvalidParameter {
            param: "symbol".into(),
            reason: "stock symbol is required".into(),
        });
    }
    let client = build_client()?;
    let path = format!(
        "/v3/reference/tickers/{}",
        crate::adapters::common::encode_path_segment(symbol)
    );
    let (body, _): (TickerDetailsResponseDTO, _) = client
        .page(
            &path,
            &[("date", date)],
            PathRule::Exact,
            None,
            "symbol_details_at",
            "",
        )
        .await?;
    if !body
        .results
        .as_ref()
        .and_then(|r| r.ticker.as_deref())
        .is_some_and(|ticker| ticker.eq_ignore_ascii_case(symbol))
    {
        return Err(invalid("ticker"));
    }
    let details = details_to_canonical(symbol, body)?;
    validate_delisting(details.delisted_utc.as_deref())?;
    if let Some(date) = &details.list_date {
        crate::models::discovery::listings::date(date).map_err(|_| invalid("list_date"))?;
    }
    Ok(details)
}

pub(crate) async fn stock_types(locale: &str) -> Result<Vec<StockType>> {
    let client = build_client()?;
    let (body, _): (super::models::PaginatedResponseDTO<StockType>, _) = client
        .page(
            "/v3/reference/tickers/types",
            &[("asset_class", "stocks"), ("locale", locale)],
            PathRule::Exact,
            None,
            "stock_types",
            "",
        )
        .await?;
    if body.next_url.is_some() {
        return Err(invalid("next_url"));
    }
    let rows = body.results.ok_or_else(|| invalid("results"))?;
    if rows.iter().any(|row| row.code.is_empty()) {
        return Err(invalid("code"));
    }
    Ok(rows)
}

pub(crate) async fn bars(
    request: &StockBarsRequest,
    cursor: Option<&PageCursor>,
) -> Result<ProviderPage<StockBar>> {
    let client = build_client()?;
    let identity = serde_json::to_string(request)?;
    let adjusted = request.adjusted()?;
    let adjustment = adjusted.to_string();
    let limit = request.limit.to_string();
    let sort = match request.sort {
        SortOrder::Ascending => "asc",
        SortOrder::Descending => "desc",
    };
    let params = [
        ("adjusted", adjustment.as_str()),
        ("sort", sort),
        ("limit", limit.as_str()),
    ];
    let path = format!(
        "/v2/aggs/ticker/{}/range/1/{}/{}/{}",
        crate::adapters::common::encode_path_segment(&request.symbol),
        request.timespan,
        request.from,
        request.to
    );
    let (body, current): (AggregateResponseDTO, _) = client
        .page(
            &path,
            &params,
            PathRule::AggregateWindow,
            cursor,
            "stock_bars_page",
            &identity,
        )
        .await?;
    if body
        .ticker
        .as_ref()
        .is_some_and(|symbol| !symbol.eq_ignore_ascii_case(&request.symbol))
        || body.adjusted.is_some_and(|value| value != adjusted)
    {
        return Err(invalid("ticker/adjusted"));
    }
    let rows = match body.results {
        Some(rows) => rows,
        None if body.results_count == Some(0) => Vec::new(),
        None => return Err(invalid("results")),
    };
    if body.results_count.is_some_and(|count| count != rows.len()) {
        return Err(invalid("resultsCount"));
    }
    let items = rows
        .into_iter()
        .map(|row| {
            if chrono::DateTime::from_timestamp_millis(row.timestamp).is_none()
                || [row.open, row.high, row.low, row.close, row.volume]
                    .iter()
                    .any(|v| !v.is_finite())
                || row.volume < 0.0
                || row.vwap.is_some_and(|v| !v.is_finite())
            {
                return Err(invalid("bar"));
            }
            Ok(StockBar {
                timestamp_ms: row.timestamp,
                open: row.open,
                high: row.high,
                low: row.low,
                close: row.close,
                volume: row.volume,
                transactions: row.transactions,
                vwap: row.vwap,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let next = client.continuation(
        body.next_url,
        &path,
        &params,
        PathRule::AggregateWindow,
        "stock_bars_page",
        &identity,
        &current,
    )?;
    Ok(ProviderPage {
        items,
        next,
        provider_id: Provider::Polygon,
        request_id: body.request_id,
        results_count: body.results_count,
        query_count: body.query_count,
        reported_symbol: body.ticker,
        adjusted: body.adjusted,
    })
}

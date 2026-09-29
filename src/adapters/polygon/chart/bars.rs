//! Resumable minute and daily stock-bar pages.

use crate::adapters::polygon::client::PathRule;
use crate::adapters::polygon::models::{AggBarDTO, AggregateResponseDTO};
use crate::adapters::polygon::{build_client, invalid_page};
use crate::{PageCursor, Provider, ProviderPage, Result, SortOrder, StockBar, StockBarsRequest};

fn bar_to_canonical(row: AggBarDTO) -> Result<StockBar> {
    if chrono::DateTime::from_timestamp_millis(row.timestamp).is_none()
        || [row.open, row.high, row.low, row.close, row.volume]
            .iter()
            .any(|v| !v.is_finite())
        || row.volume < 0.0
        || row.vwap.is_some_and(|v| !v.is_finite())
    {
        return Err(invalid_page("bar"));
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
}

pub(crate) async fn fetch_stock_bars_page(
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
        return Err(invalid_page("ticker/adjusted"));
    }
    let rows = match body.results {
        Some(rows) => rows,
        None if body.results_count == Some(0) => Vec::new(),
        None => return Err(invalid_page("results")),
    };
    if body.results_count.is_some_and(|count| count != rows.len()) {
        return Err(invalid_page("resultsCount"));
    }
    let items = rows
        .into_iter()
        .map(bar_to_canonical)
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

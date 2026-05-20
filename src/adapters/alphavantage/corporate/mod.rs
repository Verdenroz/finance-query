//! Alpha Intelligence endpoints: news sentiment, earnings call transcripts, top movers.
#![allow(dead_code)]

use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::*;

/// Fetch market news and sentiment for given tickers and/or topics.
///
/// # Arguments
///
/// * `tickers` - Optional slice of ticker symbols to filter (e.g., `&["AAPL", "MSFT"]`)
/// * `topics` - Optional slice of topics (e.g., `&["technology", "earnings"]`)
/// * `limit` - Maximum number of articles (default 50, max 1000)
pub async fn news_sentiment(
    tickers: Option<&[&str]>,
    topics: Option<&[&str]>,
    limit: Option<u32>,
) -> Result<Vec<NewsArticleDTO>> {
    let client = build_client()?;

    let tickers_str = tickers.map(|t| t.join(","));
    let topics_str = topics.map(|t| t.join(","));
    let limit_str = limit.map(|l| l.to_string());

    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(ref t) = tickers_str {
        params.push(("tickers", t));
    }
    if let Some(ref t) = topics_str {
        params.push(("topics", t));
    }
    if let Some(ref l) = limit_str {
        params.push(("limit", l));
    }

    let json = client.get("NEWS_SENTIMENT", &params).await?;

    let feed = json.get("feed").and_then(|v| v.as_array()).ok_or_else(|| {
        FinanceError::ResponseStructureError {
            field: "feed".to_string(),
            context: "Missing feed array in news sentiment response".to_string(),
        }
    })?;

    Ok(feed
        .iter()
        .filter_map(|article| {
            let ticker_sentiment = article
                .get("ticker_sentiment")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|ts| {
                            Some(TickerSentimentDTO {
                                ticker: ts.get("ticker")?.as_str()?.to_string(),
                                relevance_score: ts
                                    .get("relevancyScore")
                                    .and_then(|v| v.as_str()?.parse().ok()),
                                ticker_sentiment_score: ts
                                    .get("tickerSentimentScore")
                                    .and_then(|v| v.as_str()?.parse().ok()),
                                ticker_sentiment_label: ts
                                    .get("tickerSentimentLabel")
                                    .and_then(|v| v.as_str())
                                    .map(String::from),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            Some(NewsArticleDTO {
                title: article.get("title")?.as_str()?.to_string(),
                url: article.get("url")?.as_str()?.to_string(),
                time_published: article
                    .get("time_published")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                source: article
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                summary: article
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                overall_sentiment_score: article
                    .get("overall_sentiment_score")
                    .and_then(|v| v.as_f64()),
                overall_sentiment_label: article
                    .get("overall_sentiment_label")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                ticker_sentiment,
            })
        })
        .collect())
}

/// Fetch earnings call transcript for a symbol and quarter.
///
/// # Arguments
///
/// * `symbol` - Ticker symbol (e.g., `"AAPL"`)
/// * `quarter` - Quarter identifier in `YYYYQN` format (e.g., `"2024Q1"`)
pub async fn earnings_call_transcript(
    symbol: &str,
    quarter: &str,
) -> Result<EarningsCallTranscriptDTO> {
    let client = build_client()?;
    let json = client
        .get(
            "EARNINGS_CALL_TRANSCRIPT",
            &[("symbol", symbol), ("quarter", quarter)],
        )
        .await?;

    let transcript = json
        .get("transcript")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(EarningsCallTranscriptDTO {
        symbol: symbol.to_string(),
        quarter: quarter.to_string(),
        transcript,
    })
}

/// Fetch top gainers, losers, and most actively traded tickers.
pub async fn top_gainers_losers() -> Result<TopMoversDTO> {
    let client = build_client()?;
    let json = client.get("TOP_GAINERS_LOSERS", &[]).await?;

    fn parse_movers(json: &serde_json::Value, key: &str) -> Vec<TopMoverTickerDTO> {
        json.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| {
                        Some(TopMoverTickerDTO {
                            ticker: t.get("ticker")?.as_str()?.to_string(),
                            price: t
                                .get("price")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0")
                                .to_string(),
                            change_amount: t
                                .get("change_amount")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0")
                                .to_string(),
                            change_percentage: t
                                .get("change_percentage")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0%")
                                .to_string(),
                            volume: t
                                .get("volume")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0")
                                .to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    Ok(TopMoversDTO {
        last_updated: json
            .get("last_updated")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        top_gainers: parse_movers(&json, "top_gainers"),
        top_losers: parse_movers(&json, "top_losers"),
        most_actively_traded: parse_movers(&json, "most_actively_traded"),
    })
}

// ============================================================================
// Canonical model conversion functions
// ============================================================================

/// Fetch canonical news articles for a symbol.
pub async fn fetch_news_response(
    symbol: &str,
) -> Result<Vec<crate::models::corporate::news::News>> {
    let articles = news_sentiment(Some(&[symbol]), None, Some(50)).await?;
    Ok(articles
        .into_iter()
        .map(|a| crate::models::corporate::news::News {
            title: a.title,
            link: a.url,
            source: a.source,
            img: String::new(),
            time: a.time_published,
            provider_id: Some(crate::Provider::AlphaVantage),
        })
        .collect())
}

/// Fetch canonical chart events (dividends + splits) for a symbol.
pub async fn fetch_events_response(
    symbol: &str,
) -> Result<crate::models::chart::events::ChartEvents> {
    let divs = super::fundamentals::dividends(symbol).await?;
    let splits = super::fundamentals::splits(symbol).await?;

    let mut chart_events = crate::models::chart::events::ChartEvents::default();
    chart_events.dividends = divs
        .into_iter()
        .filter_map(|d| {
            let ts = parse_av_date(d.ex_dividend_date.as_deref()?)?;
            Some((
                ts.to_string(),
                crate::models::chart::events::DividendEvent {
                    date: ts,
                    amount: d.amount.unwrap_or(0.0),
                },
            ))
        })
        .collect();
    chart_events.splits = splits
        .into_iter()
        .filter_map(|s| {
            let ts = parse_av_date(s.effective_date.as_deref()?)?;
            let (num, den) = parse_split_ratio(s.split_ratio.as_deref().unwrap_or("1:1"));
            Some((
                ts.to_string(),
                crate::models::chart::events::SplitEvent {
                    date: ts,
                    numerator: num as f64,
                    denominator: den as f64,
                    split_ratio: format!("{}:{}", num, den),
                },
            ))
        })
        .collect();
    Ok(chart_events)
}

/// Parse an Alpha Vantage date string (YYYY-MM-DD) to a Unix timestamp.
fn parse_av_date(date_str: &str) -> Option<i64> {
    if date_str.is_empty() {
        return None;
    }
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp())
}

/// Parse a split ratio string like "4:1" into (numerator, denominator).
fn parse_split_ratio(ratio: &str) -> (u32, u32) {
    let parts: Vec<&str> = ratio.split(':').collect();
    if parts.len() == 2 {
        let num = parts[0].parse::<u32>().unwrap_or(1);
        let den = parts[1].parse::<u32>().unwrap_or(1);
        (num, den)
    } else {
        (1, 1)
    }
}

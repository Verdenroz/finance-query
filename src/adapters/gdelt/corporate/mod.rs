//! `CORPORATE` capability for GDELT — news only. GDELT has no corporate
//! calendar (earnings/dividends/splits), so this adapter never implements
//! `fetch_events`.

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::error::Result;
use crate::models::corporate::news::News;

use super::models::GdeltArticle;

/// Articles requested per symbol query. Generous enough to cover a busy
/// news day without paging (the DOC API caps at 250 per call anyway).
const MAX_RECORDS: u32 = 50;

/// Lookback window for a symbol query. GDELT's `timespan` takes a bare
/// integer + unit (`h`/`d`/`w`/`m`); two weeks balances freshness against
/// still returning *something* for a quietly-traded symbol.
const TIMESPAN: &str = "2w";

/// Build the DOC 2.0 `query` parameter for a ticker symbol.
///
/// GDELT has no ticker vocabulary of its own — [`Ticker::news`](crate::Ticker::news)
/// only ever hands this a bare symbol, never a company name — so the symbol
/// itself, quoted for an exact phrase match, is the search term. This
/// favours precision over recall: it surfaces articles that print the
/// ticker verbatim (common in financial press, e.g. `"(NASDAQ: AAPL)"`)
/// rather than every article about the underlying company by name, which
/// would require a separate symbol-to-company-name lookup this adapter
/// doesn't perform.
pub(crate) fn build_query(symbol: &str) -> String {
    format!("\"{}\"", symbol.trim())
}

/// Fetch canonical news articles for a symbol.
pub(crate) async fn fetch_news_response(symbol: &str) -> Result<Vec<News>> {
    let query = build_query(symbol);
    let response = super::client()?
        .article_search(&query, TIMESPAN, MAX_RECORDS)
        .await?;
    Ok(response.articles.into_iter().map(to_news).collect())
}

/// Map one GDELT article onto the canonical [`News`] model.
pub(crate) fn to_news(article: GdeltArticle) -> News {
    News {
        title: article.title.unwrap_or_default(),
        link: article.url,
        source: article.domain.unwrap_or_default(),
        img: article.socialimage.unwrap_or_default(),
        time: article
            .seendate
            .as_deref()
            .map(|d| relative_time_at(d, Utc::now()))
            .unwrap_or_default(),
        provider_id: Some(crate::Provider::Gdelt),
        #[cfg(feature = "sentiment")]
        sentiment: None,
    }
}

/// Convert a GDELT `seendate` into a relative time string (`"3 hours ago"`),
/// matching the other `CORPORATE` providers' convention for [`News::time`].
/// Falls back to the raw string when the timestamp doesn't parse, rather
/// than dropping it silently.
fn relative_time_at(seendate: &str, now: DateTime<Utc>) -> String {
    let Some(seen) = parse_seendate(seendate) else {
        return seendate.to_string();
    };
    let minutes = now.signed_duration_since(seen).num_minutes();
    if minutes < 1 {
        "just now".to_string()
    } else if minutes < 60 {
        format!("{minutes} minute{} ago", plural(minutes))
    } else if minutes < 60 * 24 {
        let hours = minutes / 60;
        format!("{hours} hour{} ago", plural(hours))
    } else {
        let days = minutes / (60 * 24);
        format!("{days} day{} ago", plural(days))
    }
}

fn plural(n: i64) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// Parse GDELT's `seendate`. The documented form is `"YYYYMMDDTHHMMSSZ"`;
/// the bare `"YYYYMMDDHHMMSS"` form (no separators) is also accepted since
/// GDELT's own request-side date parameters use it and some responses have
/// been observed to echo it back, so a format drift doesn't blank out every
/// article's time.
fn parse_seendate(seendate: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(seendate, "%Y%m%dT%H%M%SZ")
        .or_else(|_| NaiveDateTime::parse_from_str(seendate, "%Y%m%d%H%M%S"))
        .ok()
        .map(|naive| naive.and_utc())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_now() -> DateTime<Utc> {
        "2026-08-05T12:00:00Z".parse().unwrap()
    }

    #[test]
    fn query_wraps_the_symbol_in_quotes_for_an_exact_match() {
        assert_eq!(build_query("AAPL"), "\"AAPL\"");
        assert_eq!(build_query(" TSLA "), "\"TSLA\"");
    }

    #[test]
    fn iso_basic_seendate_parses() {
        assert_eq!(
            parse_seendate("20260805T113000Z"),
            Some("2026-08-05T11:30:00Z".parse().unwrap())
        );
    }

    #[test]
    fn bare_seendate_without_separators_also_parses() {
        assert_eq!(
            parse_seendate("20260805113000"),
            Some("2026-08-05T11:30:00Z".parse().unwrap())
        );
    }

    #[test]
    fn unparseable_seendate_returns_none() {
        assert_eq!(parse_seendate("not-a-date"), None);
    }

    #[test]
    fn recent_article_reads_minutes_ago() {
        // 30 minutes before fixed_now().
        assert_eq!(
            relative_time_at("20260805T113000Z", fixed_now()),
            "30 minutes ago"
        );
    }

    #[test]
    fn single_minute_is_not_pluralised() {
        assert_eq!(
            relative_time_at("20260805T115900Z", fixed_now()),
            "1 minute ago"
        );
    }

    #[test]
    fn same_minute_reads_just_now() {
        assert_eq!(
            relative_time_at("20260805T120000Z", fixed_now()),
            "just now"
        );
    }

    #[test]
    fn hours_old_article_reads_hours_ago() {
        // 3 hours before fixed_now().
        assert_eq!(
            relative_time_at("20260805T090000Z", fixed_now()),
            "3 hours ago"
        );
    }

    #[test]
    fn days_old_article_reads_days_ago() {
        // 2 days before fixed_now().
        assert_eq!(
            relative_time_at("20260803T120000Z", fixed_now()),
            "2 days ago"
        );
    }

    #[test]
    fn unparseable_seendate_survives_as_the_raw_string() {
        assert_eq!(relative_time_at("garbage", fixed_now()), "garbage");
    }

    #[test]
    fn article_without_a_title_maps_to_an_empty_string_not_a_panic() {
        let article = GdeltArticle {
            url: "https://example.com/a".to_string(),
            title: None,
            seendate: None,
            domain: None,
            socialimage: None,
        };
        let news = to_news(article);
        assert_eq!(news.title, "");
        assert_eq!(news.link, "https://example.com/a");
        assert_eq!(news.source, "");
        assert_eq!(news.img, "");
        assert_eq!(news.time, "");
        assert_eq!(news.provider_id, Some(crate::Provider::Gdelt));
    }
}

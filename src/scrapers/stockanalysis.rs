//! StockAnalysis.com news scraper.
//!
//! Scrapes news from stockanalysis.com

use crate::error::{Result, YahooError};
use crate::models::news::News;
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use std::collections::HashMap;
use tracing::info;

/// Yahoo Finance exchange code to StockAnalysis exchange code mapping
static EXCHANGE_MAPPING: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        // Americas
        ("OTC", "OTC"), // US OTC
        ("BA", "BCBA"), // Buenos Aires Stock Exchange
        ("MX", "BMV"),  // Mexican Stock Exchange
        ("TO", "TSX"),  // Toronto Stock Exchange
        ("V", "TSXV"),  // TSX Venture Exchange
        ("CN", "CSE"),  // Canadian Securities Exchange
        ("SA", "BVMF"), // Brazil Stock Exchange
        ("CR", "BVC"),  // Colombia Stock Exchange
        // Asia Pacific
        ("BO", "BOM"),    // Bombay Stock Exchange
        ("NS", "NSE"),    // National Stock Exchange of India
        ("T", "TYO"),     // Tokyo Stock Exchange
        ("HK", "HKG"),    // Hong Kong Stock Exchange
        ("SZ", "SHE"),    // Shenzhen Stock Exchange
        ("SS", "SHA"),    // Shanghai Stock Exchange
        ("KS", "KRX"),    // Korea Stock Exchange
        ("KQ", "KOSDAQ"), // KOSDAQ
        ("TW", "TPE"),    // Taiwan Stock Exchange
        ("TWO", "TPEX"),  // Taipei Exchange
        ("KL", "KLSE"),   // Bursa Malaysia
        ("BK", "BKK"),    // Stock Exchange of Thailand
        ("JK", "IDX"),    // Indonesia Stock Exchange
        ("AX", "ASX"),    // Australian Securities Exchange
        ("NZ", "NZE"),    // New Zealand Stock Exchange
        ("SI", "SGX"),    // Singapore Exchange
        // Europe
        ("L", "LON"),    // London Stock Exchange
        ("IL", "LON"),   // London Stock Exchange
        ("PA", "EPA"),   // Euronext Paris
        ("F", "FRA"),    // Frankfurt Stock Exchange
        ("DE", "ETR"),   // Deutsche Börse Xetra
        ("MI", "BIT"),   // Borsa Italiana
        ("MC", "BME"),   // Madrid Stock Exchange
        ("AS", "AMS"),   // Euronext Amsterdam
        ("BR", "EBR"),   // Euronext Brussels
        ("ST", "STO"),   // Nasdaq Stockholm
        ("CO", "CPH"),   // Copenhagen Stock Exchange
        ("HE", "HEL"),   // Nasdaq Helsinki
        ("OL", "OSL"),   // Oslo Børs
        ("SW", "SWX"),   // SIX Swiss Exchange
        ("LS", "ELI"),   // Euronext Lisbon
        ("AT", "ATH"),   // Athens Stock Exchange
        ("VI", "VIE"),   // Vienna Stock Exchange
        ("BE", "BELEX"), // Belgrade Stock Exchange
        ("PR", "PRA"),   // Prague Stock Exchange
        ("WA", "WSE"),   // Warsaw Stock Exchange
        // Middle East & Africa
        ("TA", "TLV"),     // Tel Aviv Stock Exchange
        ("KW", "KWSE"),    // Kuwait Stock Exchange
        ("QA", "QSE"),     // Qatar Stock Exchange
        ("SR", "TADAWUL"), // Saudi Stock Exchange
        ("JO", "ASE"),     // Amman Stock Exchange
        ("CA", "CBSE"),    // Casablanca Stock Exchange
        ("J", "JSE"),      // Johannesburg Stock Exchange
    ])
});

/// Parse a Yahoo Finance symbol into base symbol and StockAnalysis exchange code.
fn parse_symbol_exchange(yahoo_symbol: &str) -> (&str, Option<&'static str>) {
    if let Some(dot_pos) = yahoo_symbol.rfind('.') {
        let base_symbol = &yahoo_symbol[..dot_pos];
        let yahoo_exchange = &yahoo_symbol[dot_pos + 1..];
        let stockanalysis_exchange = EXCHANGE_MAPPING.get(yahoo_exchange).copied();
        (base_symbol, stockanalysis_exchange)
    } else {
        (yahoo_symbol, None)
    }
}

/// Build URLs for a symbol
fn build_symbol_urls(symbol: &str) -> Vec<String> {
    let (base_symbol, exchange) = parse_symbol_exchange(symbol);

    if let Some(exchange) = exchange {
        // International symbol with known exchange
        vec![format!(
            "https://stockanalysis.com/quote/{}/{}",
            exchange.to_lowercase(),
            base_symbol
        )]
    } else {
        // US symbol - try stocks, ETF, then OTC
        vec![
            format!("https://stockanalysis.com/stocks/{}", base_symbol),
            format!("https://stockanalysis.com/etf/{}", base_symbol),
            format!("https://stockanalysis.com/quote/otc/{}", base_symbol),
        ]
    }
}

/// Parse news articles from HTML content
fn parse_news(html: &str) -> Result<Vec<News>> {
    let document = Html::parse_document(html);

    // Find all divs that contain the news item structure:
    // - h3 with a link (title)
    // - div with title attribute (source/time)
    let all_divs = Selector::parse("div").unwrap();
    let img_sel = Selector::parse("img").unwrap();
    let title_sel = Selector::parse("h3 a").unwrap();
    let source_time_sel = Selector::parse("div[title]").unwrap();

    let mut news_list = Vec::new();
    let mut seen_titles = std::collections::HashSet::new();

    for item in document.select(&all_divs) {
        let title_elem = item.select(&title_sel).next();
        let source_time_elem = item.select(&source_time_sel).next();

        if let (Some(title_el), Some(source_time_el)) = (title_elem, source_time_elem) {
            let title = title_el.text().collect::<String>().trim().to_string();
            let link = title_el.value().attr("href").unwrap_or("").to_string();

            // Skip if missing essential fields or already seen (dedup)
            if title.is_empty() || link.is_empty() || seen_titles.contains(&title) {
                continue;
            }
            seen_titles.insert(title.clone());

            let source_time_text = source_time_el.text().collect::<String>();
            let img = item
                .select(&img_sel)
                .next()
                .and_then(|e| e.value().attr("src"))
                .unwrap_or("")
                .to_string();

            // Parse "14 hours ago - Seeking Alpha" format
            let (time, source) = if let Some(pos) = source_time_text.find(" - ") {
                (
                    source_time_text[..pos].trim().to_string(),
                    source_time_text[pos + 3..].trim().to_string(),
                )
            } else {
                (source_time_text.trim().to_string(), String::new())
            };

            news_list.push(News::new(title, link, source, img, time));
        }
    }

    Ok(news_list)
}

/// Scrape news for a specific stock symbol.
pub(crate) async fn scrape_symbol_news(symbol: &str) -> Result<Vec<News>> {
    let urls = build_symbol_urls(symbol);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    for url in urls {
        info!("Trying URL: {}", url);

        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                let html = response.text().await?;

                let news = parse_news(&html)?;
                if !news.is_empty() {
                    return Ok(news);
                }
            }
            Ok(_) => continue,
            Err(_) => continue,
        }
    }

    Err(YahooError::SymbolNotFound {
        symbol: Some(symbol.to_string()),
        context: "Could not find news for symbol".to_string(),
    })
}

/// Scrape general market news.
pub(crate) async fn scrape_general_news() -> Result<Vec<News>> {
    let url = "https://stockanalysis.com/news/";

    info!("Fetching general news");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(YahooError::ServerError {
            status: response.status().as_u16(),
            context: "Failed to fetch news".to_string(),
        });
    }

    let html = response.text().await?;

    let news = parse_news(&html)?;

    if news.is_empty() {
        return Err(YahooError::ResponseStructureError {
            field: "news".to_string(),
            context: "No news articles found".to_string(),
        });
    }

    Ok(news)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_symbol_exchange() {
        assert_eq!(parse_symbol_exchange("AAPL"), ("AAPL", None));
        assert_eq!(parse_symbol_exchange("VOD.L"), ("VOD", Some("LON")));
        assert_eq!(parse_symbol_exchange("7203.T"), ("7203", Some("TYO")));
        assert_eq!(parse_symbol_exchange("NVDA.TO"), ("NVDA", Some("TSX")));
        assert_eq!(parse_symbol_exchange("INVALID.XX"), ("INVALID", None));
    }

    #[test]
    fn test_build_symbol_urls() {
        let urls = build_symbol_urls("AAPL");
        assert_eq!(urls.len(), 3);
        assert!(urls[0].contains("stocks/AAPL"));

        let urls = build_symbol_urls("VOD.L");
        assert_eq!(urls.len(), 1);
        assert!(urls[0].contains("quote/lon/VOD"));
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_scrape_symbol_news() {
        let news = scrape_symbol_news("AAPL").await;
        assert!(news.is_ok(), "Failed: {:?}", news.err());
        let articles = news.unwrap();
        assert!(!articles.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_scrape_general_news() {
        let news = scrape_general_news().await;
        assert!(news.is_ok());
        let articles = news.unwrap();
        assert!(!articles.is_empty());
    }
}

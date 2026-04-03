//! Fundamental data endpoints: company overview, financials, earnings, dividends, splits, calendars.

use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::*;

/// Fetch company overview and fundamentals.
pub async fn company_overview(symbol: &str) -> Result<CompanyOverview> {
    let client = build_client()?;
    let json = client.get("OVERVIEW", &[("symbol", symbol)]).await?;

    let obj = json
        .as_object()
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "overview".to_string(),
            context: "Expected object in OVERVIEW response".to_string(),
        })?;

    fn str_val(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<String> {
        let v = obj.get(key)?.as_str()?;
        if v == "None" || v == "-" || v.is_empty() {
            None
        } else {
            Some(v.to_string())
        }
    }

    fn f64_val(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<f64> {
        obj.get(key)?.as_str()?.parse().ok()
    }

    fn u32_val(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<u32> {
        obj.get(key)?.as_str()?.parse().ok()
    }

    Ok(CompanyOverview {
        symbol: str_val(obj, "Symbol").unwrap_or_else(|| symbol.to_string()),
        asset_type: str_val(obj, "AssetType"),
        name: str_val(obj, "Name"),
        description: str_val(obj, "Description"),
        exchange: str_val(obj, "Exchange"),
        currency: str_val(obj, "Currency"),
        country: str_val(obj, "Country"),
        sector: str_val(obj, "Sector"),
        industry: str_val(obj, "Industry"),
        market_capitalization: f64_val(obj, "MarketCapitalization"),
        pe_ratio: f64_val(obj, "PERatio"),
        peg_ratio: f64_val(obj, "PEGRatio"),
        book_value: f64_val(obj, "BookValue"),
        dividend_per_share: f64_val(obj, "DividendPerShare"),
        dividend_yield: f64_val(obj, "DividendYield"),
        eps: f64_val(obj, "EPS"),
        revenue_per_share_ttm: f64_val(obj, "RevenuePerShareTTM"),
        profit_margin: f64_val(obj, "ProfitMargin"),
        operating_margin_ttm: f64_val(obj, "OperatingMarginTTM"),
        return_on_assets_ttm: f64_val(obj, "ReturnOnAssetsTTM"),
        return_on_equity_ttm: f64_val(obj, "ReturnOnEquityTTM"),
        revenue_ttm: f64_val(obj, "RevenueTTM"),
        gross_profit_ttm: f64_val(obj, "GrossProfitTTM"),
        ebitda: f64_val(obj, "EBITDA"),
        week_52_high: f64_val(obj, "52WeekHigh"),
        week_52_low: f64_val(obj, "52WeekLow"),
        moving_average_50day: f64_val(obj, "50DayMovingAverage"),
        moving_average_200day: f64_val(obj, "200DayMovingAverage"),
        shares_outstanding: f64_val(obj, "SharesOutstanding"),
        beta: f64_val(obj, "Beta"),
        forward_pe: f64_val(obj, "ForwardPE"),
        price_to_sales_ratio_ttm: f64_val(obj, "PriceToSalesRatioTTM"),
        price_to_book_ratio: f64_val(obj, "PriceToBookRatio"),
        analyst_target_price: f64_val(obj, "AnalystTargetPrice"),
        analyst_rating_strong_buy: u32_val(obj, "AnalystRatingStrongBuy"),
        analyst_rating_buy: u32_val(obj, "AnalystRatingBuy"),
        analyst_rating_hold: u32_val(obj, "AnalystRatingHold"),
        analyst_rating_sell: u32_val(obj, "AnalystRatingSell"),
        analyst_rating_strong_sell: u32_val(obj, "AnalystRatingStrongSell"),
    })
}

/// Fetch ETF profile and top holdings.
pub async fn etf_profile(symbol: &str) -> Result<EtfProfile> {
    let client = build_client()?;
    let json = client.get("ETF_PROFILE", &[("symbol", symbol)]).await?;

    fn str_val(v: &serde_json::Value, key: &str) -> Option<String> {
        let s = v.get(key)?.as_str()?;
        if s == "None" || s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    }

    fn f64_val(v: &serde_json::Value, key: &str) -> Option<f64> {
        v.get(key)?.as_str()?.parse().ok()
    }

    let holdings = json
        .get("holdings")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|h| EtfHolding {
                    symbol: h.get("symbol").and_then(|v| v.as_str()).map(String::from),
                    description: h
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    weight: h.get("weight").and_then(|v| v.as_str()?.parse().ok()),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(EtfProfile {
        symbol: str_val(&json, "symbol").unwrap_or_else(|| symbol.to_string()),
        name: str_val(&json, "name"),
        asset_type: str_val(&json, "asset_type"),
        net_assets: f64_val(&json, "net_assets"),
        net_expense_ratio: f64_val(&json, "net_expense_ratio"),
        portfolio_turnover: f64_val(&json, "portfolio_turnover"),
        dividend_yield: f64_val(&json, "dividend_yield"),
        inception_date: str_val(&json, "inception_date"),
        holdings,
    })
}

/// Fetch income statement (annual and quarterly).
pub async fn income_statement(symbol: &str) -> Result<FinancialStatements> {
    fetch_financial_statement(symbol, "INCOME_STATEMENT").await
}

/// Fetch balance sheet (annual and quarterly).
pub async fn balance_sheet(symbol: &str) -> Result<FinancialStatements> {
    fetch_financial_statement(symbol, "BALANCE_SHEET").await
}

/// Fetch cash flow statement (annual and quarterly).
pub async fn cash_flow(symbol: &str) -> Result<FinancialStatements> {
    fetch_financial_statement(symbol, "CASH_FLOW").await
}

async fn fetch_financial_statement(symbol: &str, function: &str) -> Result<FinancialStatements> {
    let client = build_client()?;
    let json = client.get(function, &[("symbol", symbol)]).await?;

    fn parse_reports(json: &serde_json::Value, key: &str) -> Vec<FinancialReport> {
        json.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| {
                        let obj = r.as_object()?;
                        let fiscal_date = obj.get("fiscalDateEnding")?.as_str()?.to_string();
                        let currency = obj
                            .get("reportedCurrency")
                            .and_then(|v| v.as_str())
                            .unwrap_or("USD")
                            .to_string();
                        let mut fields = std::collections::HashMap::new();
                        for (k, v) in obj {
                            if k != "fiscalDateEnding" && k != "reportedCurrency" {
                                fields.insert(k.clone(), v.clone());
                            }
                        }
                        Some(FinancialReport {
                            fiscal_date_ending: fiscal_date,
                            reported_currency: currency,
                            fields,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    Ok(FinancialStatements {
        symbol: json
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or(symbol)
            .to_string(),
        annual_reports: parse_reports(&json, "annualReports"),
        quarterly_reports: parse_reports(&json, "quarterlyReports"),
    })
}

/// Fetch earnings history (annual and quarterly EPS data).
pub async fn earnings(symbol: &str) -> Result<EarningsHistory> {
    let client = build_client()?;
    let json = client.get("EARNINGS", &[("symbol", symbol)]).await?;

    fn parse_earnings(json: &serde_json::Value, key: &str) -> Vec<EarningsData> {
        json.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|e| EarningsData {
                        fiscal_date_ending: e
                            .get("fiscalDateEnding")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        reported_date: e
                            .get("reportedDate")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        reported_eps: e.get("reportedEPS").and_then(|v| v.as_str()?.parse().ok()),
                        estimated_eps: e.get("estimatedEPS").and_then(|v| v.as_str()?.parse().ok()),
                        surprise: e.get("surprise").and_then(|v| v.as_str()?.parse().ok()),
                        surprise_percentage: e
                            .get("surprisePercentage")
                            .and_then(|v| v.as_str()?.parse().ok()),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    Ok(EarningsHistory {
        symbol: json
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or(symbol)
            .to_string(),
        annual_earnings: parse_earnings(&json, "annualEarnings"),
        quarterly_earnings: parse_earnings(&json, "quarterlyEarnings"),
    })
}

/// Fetch historical dividend events.
pub async fn dividends(symbol: &str) -> Result<Vec<DividendEvent>> {
    let client = build_client()?;
    let csv = client.get_csv("DIVIDENDS", &[("symbol", symbol)]).await?;
    parse_dividend_csv(&csv)
}

fn parse_dividend_csv(csv: &str) -> Result<Vec<DividendEvent>> {
    let mut events = Vec::new();
    let mut lines = csv.lines();
    let _header = lines.next(); // skip header

    for line in lines {
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() >= 2 {
            events.push(DividendEvent {
                ex_dividend_date: Some(cols[0].to_string()),
                declaration_date: cols.get(1).map(|s| s.to_string()),
                record_date: cols.get(2).map(|s| s.to_string()),
                payment_date: cols.get(3).map(|s| s.to_string()),
                amount: cols.get(4).and_then(|s| s.parse().ok()),
            });
        }
    }
    Ok(events)
}

/// Fetch historical stock split events.
pub async fn splits(symbol: &str) -> Result<Vec<SplitEvent>> {
    let client = build_client()?;
    let csv = client.get_csv("SPLITS", &[("symbol", symbol)]).await?;
    parse_split_csv(&csv)
}

fn parse_split_csv(csv: &str) -> Result<Vec<SplitEvent>> {
    let mut events = Vec::new();
    let mut lines = csv.lines();
    let _header = lines.next();

    for line in lines {
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() >= 2 {
            events.push(SplitEvent {
                effective_date: Some(cols[0].to_string()),
                split_ratio: Some(cols[1].to_string()),
            });
        }
    }
    Ok(events)
}

/// Fetch listing status (active or delisted).
///
/// * `status` - `"active"` (default) or `"delisted"`
pub async fn listing_status(status: Option<&str>) -> Result<Vec<ListingEntry>> {
    let client = build_client()?;
    let params: Vec<(&str, &str)> = match status {
        Some(s) => vec![("state", s)],
        None => vec![],
    };
    let csv = client.get_csv("LISTING_STATUS", &params).await?;

    let mut entries = Vec::new();
    let mut lines = csv.lines();
    let _header = lines.next();

    for line in lines {
        let cols: Vec<&str> = line.split(',').collect();
        if !cols.is_empty() {
            entries.push(ListingEntry {
                symbol: cols[0].to_string(),
                name: cols.get(1).map(|s| s.to_string()),
                exchange: cols.get(2).map(|s| s.to_string()),
                asset_type: cols.get(3).map(|s| s.to_string()),
                ipo_date: cols.get(4).map(|s| s.to_string()),
                delisting_date: cols.get(5).map(|s| s.to_string()),
                status: cols.get(6).map(|s| s.to_string()),
            });
        }
    }
    Ok(entries)
}

/// Fetch upcoming earnings calendar.
pub async fn earnings_calendar() -> Result<Vec<EarningsCalendarEntry>> {
    let client = build_client()?;
    let csv = client.get_csv("EARNINGS_CALENDAR", &[]).await?;

    let mut entries = Vec::new();
    let mut lines = csv.lines();
    let _header = lines.next();

    for line in lines {
        let cols: Vec<&str> = line.split(',').collect();
        if !cols.is_empty() {
            entries.push(EarningsCalendarEntry {
                symbol: cols[0].to_string(),
                name: cols.get(1).map(|s| s.to_string()),
                report_date: cols.get(2).map(|s| s.to_string()),
                fiscal_date_ending: cols.get(3).map(|s| s.to_string()),
                estimate: cols.get(4).and_then(|s| s.parse().ok()),
                currency: cols.get(5).map(|s| s.to_string()),
            });
        }
    }
    Ok(entries)
}

/// Fetch upcoming IPO calendar.
pub async fn ipo_calendar() -> Result<Vec<IpoCalendarEntry>> {
    let client = build_client()?;
    let csv = client.get_csv("IPO_CALENDAR", &[]).await?;

    let mut entries = Vec::new();
    let mut lines = csv.lines();
    let _header = lines.next();

    for line in lines {
        let cols: Vec<&str> = line.split(',').collect();
        if !cols.is_empty() {
            entries.push(IpoCalendarEntry {
                symbol: cols.first().map(|s| s.to_string()),
                name: cols.get(1).map(|s| s.to_string()),
                ipo_date: cols.get(2).map(|s| s.to_string()),
                price_range: cols.get(3).map(|s| s.to_string()),
                exchange: cols.get(4).map(|s| s.to_string()),
            });
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dividend_csv() {
        let csv = "ex_dividend_date,declaration_date,record_date,payment_date,amount\n\
                   2024-01-12,2024-01-02,2024-01-15,2024-02-01,0.24\n\
                   2023-10-13,2023-10-02,2023-10-16,2023-11-01,0.24";

        let events = parse_dividend_csv(csv).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].ex_dividend_date.as_deref(), Some("2024-01-12"));
        assert!((events[0].amount.unwrap() - 0.24).abs() < 0.001);
    }

    #[test]
    fn test_parse_split_csv() {
        let csv = "effective_date,split_ratio\n\
                   2020-08-31,4:1\n\
                   2014-06-09,7:1";

        let events = parse_split_csv(csv).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].effective_date.as_deref(), Some("2020-08-31"));
        assert_eq!(events[0].split_ratio.as_deref(), Some("4:1"));
    }

    #[test]
    fn test_parse_empty_csv() {
        let csv = "header_only";
        let events = parse_dividend_csv(csv).unwrap();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_company_overview_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("function".into(), "OVERVIEW".into()),
                mockito::Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "Symbol": "AAPL",
                    "AssetType": "Common Stock",
                    "Name": "Apple Inc",
                    "Exchange": "NASDAQ",
                    "Currency": "USD",
                    "Country": "USA",
                    "Sector": "TECHNOLOGY",
                    "Industry": "ELECTRONIC COMPUTERS",
                    "MarketCapitalization": "2850000000000",
                    "PERatio": "28.5",
                    "EPS": "6.43",
                    "DividendYield": "0.0055",
                    "52WeekHigh": "199.62",
                    "52WeekLow": "164.08",
                    "SharesOutstanding": "15460000000",
                    "Beta": "1.24"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client.get("OVERVIEW", &[("symbol", "AAPL")]).await.unwrap();

        let obj = json.as_object().unwrap();
        assert_eq!(obj.get("Symbol").unwrap().as_str().unwrap(), "AAPL");
        assert_eq!(obj.get("Name").unwrap().as_str().unwrap(), "Apple Inc");
    }

    #[tokio::test]
    async fn test_csv_endpoint_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "function".into(),
                "DIVIDENDS".into(),
            )]))
            .with_status(200)
            .with_body(
                "ex_dividend_date,declaration_date,record_date,payment_date,amount\n\
                 2024-01-12,2024-01-02,2024-01-15,2024-02-01,0.24",
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let csv = client
            .get_csv("DIVIDENDS", &[("symbol", "AAPL")])
            .await
            .unwrap();
        let events = parse_dividend_csv(&csv).unwrap();
        assert_eq!(events.len(), 1);
        assert!((events[0].amount.unwrap() - 0.24).abs() < 0.001);
    }
}

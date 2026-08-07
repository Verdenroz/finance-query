#![cfg(feature = "fmp")]

use chrono::{Days, Utc};
use finance_query::fmp::{self, TechnicalIndicator};

#[tokio::test]
#[ignore = "requires FMP_API_KEY and consumes live API quota"]
async fn stable_research_endpoints_work_live() -> finance_query::Result<()> {
    fmp::init(std::env::var("FMP_API_KEY").expect("FMP_API_KEY must be set"))?;

    assert!(!fmp::name_search("Apple").await?.is_empty());
    assert!(!fmp::cik_search("320193").await?.is_empty());
    assert!(!fmp::cusip_search("037833100").await?.is_empty());
    assert!(!fmp::isin_search("US0378331005").await?.is_empty());
    assert!(!fmp::exchange_variants("AAPL").await?.is_empty());

    assert!(!fmp::quote_short("AAPL").await?.is_empty());
    assert!(!fmp::batch_quote_short(&["AAPL", "MSFT"]).await?.is_empty());
    fmp::aftermarket_trade("AAPL").await?;
    fmp::batch_aftermarket_trade(&["AAPL", "MSFT"]).await?;
    fmp::aftermarket_quote("AAPL").await?;
    fmp::batch_aftermarket_quote(&["AAPL", "MSFT"]).await?;
    assert!(!fmp::stock_price_change("AAPL").await?.is_empty());

    assert!(!fmp::financial_scores("AAPL").await?.is_empty());
    assert!(!fmp::owner_earnings("AAPL").await?.is_empty());

    let today = Utc::now().date_naive();
    let from = today.checked_sub_days(Days::new(365)).unwrap().to_string();
    let to = today.to_string();
    assert!(
        !fmp::treasury_rates(Some(&from), Some(&to))
            .await?
            .is_empty()
    );
    assert!(!fmp::economic_indicators("GDP").await?.is_empty());
    assert!(!fmp::market_risk_premium().await?.is_empty());

    for indicator in [
        TechnicalIndicator::Sma,
        TechnicalIndicator::Ema,
        TechnicalIndicator::Wma,
        TechnicalIndicator::Dema,
        TechnicalIndicator::Tema,
        TechnicalIndicator::Rsi,
        TechnicalIndicator::StandardDeviation,
        TechnicalIndicator::Williams,
        TechnicalIndicator::Adx,
    ] {
        assert!(
            !fmp::technical_indicator(indicator, "AAPL", 14, "1day", Some(&from), Some(&to),)
                .await?
                .is_empty()
        );
    }

    assert!(
        !fmp::sec_filings_by_symbol("AAPL", &from, &to, 0, 5)
            .await?
            .is_empty()
    );
    assert!(
        !fmp::sec_filings_by_form_type("10-Q", &from, &to, 0, 5)
            .await?
            .is_empty()
    );
    assert!(
        !fmp::sec_filings_by_cik("0000320193", &from, &to, 0, 5)
            .await?
            .is_empty()
    );

    Ok(())
}

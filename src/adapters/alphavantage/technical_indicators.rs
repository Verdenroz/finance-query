//! Technical indicator endpoints: SMA, EMA, RSI, MACD, BBANDS, STOCH, ADX, and 50+ more.

use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::*;

/// Fetch any technical indicator by function name.
///
/// This is the generic entrypoint. For convenience, typed wrappers are provided below.
///
/// # Arguments
///
/// * `function` - Indicator function name (e.g., `"SMA"`, `"RSI"`, `"MACD"`)
/// * `symbol` - Ticker symbol
/// * `interval` - Time interval
/// * `extra_params` - Additional parameters (e.g., `time_period`, `series_type`, `fastperiod`)
pub async fn technical_indicator(
    function: &str,
    symbol: &str,
    interval: AvInterval,
    extra_params: &[(&str, &str)],
) -> Result<TechnicalIndicator> {
    let client = build_client()?;
    let mut params = vec![("symbol", symbol), ("interval", interval.as_str())];
    params.extend_from_slice(extra_params);

    let json = client.get(function, &params).await?;

    let meta = json.get("Meta Data");
    let last_refreshed = meta
        .and_then(|m| m.get("3: Last Refreshed"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let ind_interval = meta
        .and_then(|m| m.get("4: Interval"))
        .and_then(|v| v.as_str())
        .unwrap_or(interval.as_str())
        .to_string();

    // Find the "Technical Analysis: ..." key
    let analysis = json
        .as_object()
        .and_then(|obj| {
            obj.iter()
                .find(|(k, _)| k.starts_with("Technical Analysis"))
                .map(|(_, v)| v)
        })
        .and_then(|v| v.as_object())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "Technical Analysis".to_string(),
            context: format!("Missing Technical Analysis data for {function}"),
        })?;

    let mut data: Vec<IndicatorDataPoint> = analysis
        .iter()
        .map(|(timestamp, values)| {
            let mut indicator_values = std::collections::HashMap::new();
            if let Some(obj) = values.as_object() {
                for (k, v) in obj {
                    if let Some(n) = v.as_str().and_then(|s| s.parse::<f64>().ok()) {
                        indicator_values.insert(k.clone(), n);
                    }
                }
            }
            IndicatorDataPoint {
                timestamp: timestamp.clone(),
                values: indicator_values,
            }
        })
        .collect();

    data.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(TechnicalIndicator {
        indicator: function.to_string(),
        symbol: symbol.to_string(),
        last_refreshed,
        interval: ind_interval,
        data,
    })
}

// ============================================================================
// Typed convenience wrappers
// ============================================================================

/// Simple Moving Average (SMA).
pub async fn sma(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "SMA",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Exponential Moving Average (EMA).
pub async fn ema(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "EMA",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Weighted Moving Average (WMA).
pub async fn wma(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "WMA",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Double Exponential Moving Average (DEMA).
pub async fn dema(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "DEMA",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Triple Exponential Moving Average (TEMA).
pub async fn tema(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "TEMA",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Triangular Moving Average (TRIMA).
pub async fn trima(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "TRIMA",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Kaufman Adaptive Moving Average (KAMA).
pub async fn kama(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "KAMA",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Triple Exponential Moving Average T3.
pub async fn t3(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "T3",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Moving Average Convergence/Divergence (MACD).
pub async fn macd(
    symbol: &str,
    interval: AvInterval,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    technical_indicator(
        "MACD",
        symbol,
        interval,
        &[("series_type", series_type.as_str())],
    )
    .await
}

/// MACD with custom periods.
pub async fn macdext(
    symbol: &str,
    interval: AvInterval,
    series_type: SeriesType,
    fast_period: Option<u32>,
    slow_period: Option<u32>,
    signal_period: Option<u32>,
) -> Result<TechnicalIndicator> {
    let fp = fast_period.unwrap_or(12).to_string();
    let sp = slow_period.unwrap_or(26).to_string();
    let sigp = signal_period.unwrap_or(9).to_string();
    technical_indicator(
        "MACDEXT",
        symbol,
        interval,
        &[
            ("series_type", series_type.as_str()),
            ("fastperiod", &fp),
            ("slowperiod", &sp),
            ("signalperiod", &sigp),
        ],
    )
    .await
}

/// Relative Strength Index (RSI).
pub async fn rsi(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "RSI",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Stochastic Oscillator (STOCH).
pub async fn stoch(symbol: &str, interval: AvInterval) -> Result<TechnicalIndicator> {
    technical_indicator("STOCH", symbol, interval, &[]).await
}

/// Stochastic Fast (STOCHF).
pub async fn stochf(symbol: &str, interval: AvInterval) -> Result<TechnicalIndicator> {
    technical_indicator("STOCHF", symbol, interval, &[]).await
}

/// Stochastic RSI (STOCHRSI).
pub async fn stochrsi(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "STOCHRSI",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Bollinger Bands (BBANDS).
pub async fn bbands(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "BBANDS",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Average Directional Index (ADX).
pub async fn adx(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("ADX", symbol, interval, &[("time_period", &tp)]).await
}

/// Average Directional Index Rating (ADXR).
pub async fn adxr(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("ADXR", symbol, interval, &[("time_period", &tp)]).await
}

/// Absolute Price Oscillator (APO).
pub async fn apo(
    symbol: &str,
    interval: AvInterval,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    technical_indicator(
        "APO",
        symbol,
        interval,
        &[("series_type", series_type.as_str())],
    )
    .await
}

/// Percentage Price Oscillator (PPO).
pub async fn ppo(
    symbol: &str,
    interval: AvInterval,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    technical_indicator(
        "PPO",
        symbol,
        interval,
        &[("series_type", series_type.as_str())],
    )
    .await
}

/// Momentum (MOM).
pub async fn mom(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "MOM",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Rate of Change (ROC).
pub async fn roc(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "ROC",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Rate of Change Ratio (ROCR).
pub async fn rocr(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "ROCR",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Commodity Channel Index (CCI).
pub async fn cci(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("CCI", symbol, interval, &[("time_period", &tp)]).await
}

/// Chande Momentum Oscillator (CMO).
pub async fn cmo(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "CMO",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Williams %R (WILLR).
pub async fn willr(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("WILLR", symbol, interval, &[("time_period", &tp)]).await
}

/// Aroon (AROON).
pub async fn aroon(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("AROON", symbol, interval, &[("time_period", &tp)]).await
}

/// Aroon Oscillator (AROONOSC).
pub async fn aroonosc(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("AROONOSC", symbol, interval, &[("time_period", &tp)]).await
}

/// Balance of Power (BOP).
pub async fn bop(symbol: &str, interval: AvInterval) -> Result<TechnicalIndicator> {
    technical_indicator("BOP", symbol, interval, &[]).await
}

/// Money Flow Index (MFI).
pub async fn mfi(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("MFI", symbol, interval, &[("time_period", &tp)]).await
}

/// Average True Range (ATR).
pub async fn atr(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("ATR", symbol, interval, &[("time_period", &tp)]).await
}

/// Normalized Average True Range (NATR).
pub async fn natr(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("NATR", symbol, interval, &[("time_period", &tp)]).await
}

/// True Range (TRANGE).
pub async fn trange(symbol: &str, interval: AvInterval) -> Result<TechnicalIndicator> {
    technical_indicator("TRANGE", symbol, interval, &[]).await
}

/// Chaikin A/D Line (AD).
pub async fn ad(symbol: &str, interval: AvInterval) -> Result<TechnicalIndicator> {
    technical_indicator("AD", symbol, interval, &[]).await
}

/// Chaikin A/D Oscillator (ADOSC).
pub async fn adosc(symbol: &str, interval: AvInterval) -> Result<TechnicalIndicator> {
    technical_indicator("ADOSC", symbol, interval, &[]).await
}

/// On Balance Volume (OBV).
pub async fn obv(symbol: &str, interval: AvInterval) -> Result<TechnicalIndicator> {
    technical_indicator("OBV", symbol, interval, &[]).await
}

/// Directional Movement Index (DX).
pub async fn dx(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("DX", symbol, interval, &[("time_period", &tp)]).await
}

/// Minus Directional Indicator (MINUS_DI).
pub async fn minus_di(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("MINUS_DI", symbol, interval, &[("time_period", &tp)]).await
}

/// Plus Directional Indicator (PLUS_DI).
pub async fn plus_di(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("PLUS_DI", symbol, interval, &[("time_period", &tp)]).await
}

/// Minus Directional Movement (MINUS_DM).
pub async fn minus_dm(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("MINUS_DM", symbol, interval, &[("time_period", &tp)]).await
}

/// Plus Directional Movement (PLUS_DM).
pub async fn plus_dm(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("PLUS_DM", symbol, interval, &[("time_period", &tp)]).await
}

/// Midpoint (MIDPOINT).
pub async fn midpoint(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "MIDPOINT",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Midpoint Price (MIDPRICE).
pub async fn midprice(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator("MIDPRICE", symbol, interval, &[("time_period", &tp)]).await
}

/// Parabolic SAR (SAR).
pub async fn sar(symbol: &str, interval: AvInterval) -> Result<TechnicalIndicator> {
    technical_indicator("SAR", symbol, interval, &[]).await
}

/// Triple Smooth EMA (TRIX).
pub async fn trix(
    symbol: &str,
    interval: AvInterval,
    time_period: u32,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    let tp = time_period.to_string();
    technical_indicator(
        "TRIX",
        symbol,
        interval,
        &[("time_period", &tp), ("series_type", series_type.as_str())],
    )
    .await
}

/// Ultimate Oscillator (ULTOSC).
pub async fn ultosc(symbol: &str, interval: AvInterval) -> Result<TechnicalIndicator> {
    technical_indicator("ULTOSC", symbol, interval, &[]).await
}

/// Volume Weighted Average Price (VWAP).
pub async fn vwap(symbol: &str, interval: AvInterval) -> Result<TechnicalIndicator> {
    technical_indicator("VWAP", symbol, interval, &[]).await
}

/// Hilbert Transform - Trendline (HT_TRENDLINE).
pub async fn ht_trendline(
    symbol: &str,
    interval: AvInterval,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    technical_indicator(
        "HT_TRENDLINE",
        symbol,
        interval,
        &[("series_type", series_type.as_str())],
    )
    .await
}

/// Hilbert Transform - Sine Wave (HT_SINE).
pub async fn ht_sine(
    symbol: &str,
    interval: AvInterval,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    technical_indicator(
        "HT_SINE",
        symbol,
        interval,
        &[("series_type", series_type.as_str())],
    )
    .await
}

/// Hilbert Transform - Trend vs Cycle Mode (HT_TRENDMODE).
pub async fn ht_trendmode(
    symbol: &str,
    interval: AvInterval,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    technical_indicator(
        "HT_TRENDMODE",
        symbol,
        interval,
        &[("series_type", series_type.as_str())],
    )
    .await
}

/// Hilbert Transform - Dominant Cycle Period (HT_DCPERIOD).
pub async fn ht_dcperiod(
    symbol: &str,
    interval: AvInterval,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    technical_indicator(
        "HT_DCPERIOD",
        symbol,
        interval,
        &[("series_type", series_type.as_str())],
    )
    .await
}

/// Hilbert Transform - Dominant Cycle Phase (HT_DCPHASE).
pub async fn ht_dcphase(
    symbol: &str,
    interval: AvInterval,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    technical_indicator(
        "HT_DCPHASE",
        symbol,
        interval,
        &[("series_type", series_type.as_str())],
    )
    .await
}

/// Hilbert Transform - Phasor Components (HT_PHASOR).
pub async fn ht_phasor(
    symbol: &str,
    interval: AvInterval,
    series_type: SeriesType,
) -> Result<TechnicalIndicator> {
    technical_indicator(
        "HT_PHASOR",
        symbol,
        interval,
        &[("series_type", series_type.as_str())],
    )
    .await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_sma_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("function".into(), "SMA".into()),
                mockito::Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
                mockito::Matcher::UrlEncoded("interval".into(), "daily".into()),
                mockito::Matcher::UrlEncoded("time_period".into(), "20".into()),
                mockito::Matcher::UrlEncoded("series_type".into(), "close".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "Meta Data": {
                        "1: Symbol": "AAPL",
                        "2: Indicator": "Simple Moving Average (SMA)",
                        "3: Last Refreshed": "2024-01-15",
                        "4: Interval": "daily",
                        "5: Time Period": 20,
                        "6: Series Type": "close"
                    },
                    "Technical Analysis: SMA": {
                        "2024-01-15": { "SMA": "187.4350" },
                        "2024-01-12": { "SMA": "186.9100" },
                        "2024-01-11": { "SMA": "186.5500" }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get(
                "SMA",
                &[
                    ("symbol", "AAPL"),
                    ("interval", "daily"),
                    ("time_period", "20"),
                    ("series_type", "close"),
                ],
            )
            .await
            .unwrap();

        // Parse the response manually like technical_indicator does
        let analysis = json
            .as_object()
            .unwrap()
            .iter()
            .find(|(k, _)| k.starts_with("Technical Analysis"))
            .map(|(_, v)| v)
            .unwrap()
            .as_object()
            .unwrap();

        assert_eq!(analysis.len(), 3);
        let sma_val = analysis
            .get("2024-01-15")
            .unwrap()
            .get("SMA")
            .unwrap()
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!((sma_val - 187.435).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_bbands_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "function".into(),
                "BBANDS".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "Meta Data": {
                        "1: Symbol": "AAPL",
                        "2: Indicator": "Bollinger Bands (BBANDS)",
                        "3: Last Refreshed": "2024-01-15",
                        "4: Interval": "daily"
                    },
                    "Technical Analysis: BBANDS": {
                        "2024-01-15": {
                            "Real Upper Band": "192.5000",
                            "Real Middle Band": "187.4350",
                            "Real Lower Band": "182.3700"
                        }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get(
                "BBANDS",
                &[
                    ("symbol", "AAPL"),
                    ("interval", "daily"),
                    ("time_period", "20"),
                    ("series_type", "close"),
                ],
            )
            .await
            .unwrap();

        let analysis = json
            .as_object()
            .unwrap()
            .iter()
            .find(|(k, _)| k.starts_with("Technical Analysis"))
            .unwrap()
            .1
            .as_object()
            .unwrap();

        let bands = analysis.get("2024-01-15").unwrap();
        assert_eq!(
            bands.get("Real Upper Band").unwrap().as_str().unwrap(),
            "192.5000"
        );
        assert_eq!(
            bands.get("Real Lower Band").unwrap().as_str().unwrap(),
            "182.3700"
        );
    }
}

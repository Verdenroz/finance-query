//! Compile and runtime tests for docs/library/indicators.md
//!
//! Requires the `indicators` feature flag:
//!   cargo test --test doc_indicators --features indicators
//!   cargo test --test doc_indicators --features indicators -- --ignored  (network)
//!   cargo test --test doc_indicators --features "indicators,dataframe" -- --ignored

#![cfg(feature = "indicators")]

// ---------------------------------------------------------------------------
// Compile-time — CandlePattern and PatternSentiment variants
// ---------------------------------------------------------------------------

#[test]
fn test_candle_pattern_variants_compile() {
    use finance_query::indicators::CandlePattern;

    let _ = CandlePattern::MorningStar;
    let _ = CandlePattern::EveningStar;
    let _ = CandlePattern::ThreeWhiteSoldiers;
    let _ = CandlePattern::ThreeBlackCrows;
    let _ = CandlePattern::BullishEngulfing;
    let _ = CandlePattern::BearishEngulfing;
    let _ = CandlePattern::BullishHarami;
    let _ = CandlePattern::BearishHarami;
    let _ = CandlePattern::PiercingLine;
    let _ = CandlePattern::DarkCloudCover;
    let _ = CandlePattern::TweezerBottom;
    let _ = CandlePattern::TweezerTop;
    let _ = CandlePattern::Hammer;
    let _ = CandlePattern::InvertedHammer;
    let _ = CandlePattern::HangingMan;
    let _ = CandlePattern::ShootingStar;
    let _ = CandlePattern::BullishMarubozu;
    let _ = CandlePattern::BearishMarubozu;
    let _ = CandlePattern::Doji;
    let _ = CandlePattern::SpinningTop;
}

#[test]
fn test_pattern_sentiment_variants_compile() {
    use finance_query::indicators::PatternSentiment;

    let _ = PatternSentiment::Bullish;
    let _ = PatternSentiment::Bearish;
}

// ---------------------------------------------------------------------------
// Compile-time — IndicatorsSummary compound data type field access
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn _verify_indicators_summary_fields(s: finance_query::IndicatorsSummary) {
    // Moving Averages
    let _: Option<f64> = s.sma_10;
    let _: Option<f64> = s.sma_20;
    let _: Option<f64> = s.sma_50;
    let _: Option<f64> = s.sma_100;
    let _: Option<f64> = s.sma_200;
    let _: Option<f64> = s.ema_10;
    let _: Option<f64> = s.ema_20;
    let _: Option<f64> = s.ema_50;
    let _: Option<f64> = s.ema_100;
    let _: Option<f64> = s.ema_200;
    let _: Option<f64> = s.dema_20;
    let _: Option<f64> = s.tema_20;
    let _: Option<f64> = s.hma_20;
    let _: Option<f64> = s.vwma_20;
    let _: Option<f64> = s.alma_9;
    let _: Option<f64> = s.mcginley_dynamic_20;
    // Momentum
    let _: Option<f64> = s.rsi_14;
    let _: Option<f64> = s.cci_20;
    let _: Option<f64> = s.williams_r_14;
    let _: Option<f64> = s.roc_12;
    let _: Option<f64> = s.momentum_10;
    let _: Option<f64> = s.cmo_14;
    let _: Option<f64> = s.awesome_oscillator;
    let _: Option<f64> = s.coppock_curve;
    let _: Option<f64> = s.adx_14;
    let _: Option<f64> = s.parabolic_sar;
    // Volatility
    let _: Option<f64> = s.atr_14;
    let _: Option<f64> = s.true_range;
    let _: Option<f64> = s.choppiness_index_14;
    // Volume
    let _: Option<f64> = s.obv;
    let _: Option<f64> = s.mfi_14;
    let _: Option<f64> = s.cmf_20;
    let _: Option<f64> = s.chaikin_oscillator;
    let _: Option<f64> = s.accumulation_distribution;
    let _: Option<f64> = s.vwap;
    let _: Option<f64> = s.balance_of_power;
}

// ---------------------------------------------------------------------------
// Network tests — Getting Started / Summary API from indicators.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_indicators_getting_started() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "Getting Started" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let indicators = ticker
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    println!("RSI(14): {:?}", indicators.rsi_14);
    println!("SMA(200): {:?}", indicators.sma_200);
    println!("MACD: {:?}", indicators.macd);
}

// ---------------------------------------------------------------------------
// Network tests — Chart Extension Methods from indicators.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_chart_extension_methods() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "Chart Extension Methods" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    // Calculate indicators with custom periods
    let sma_15 = chart.sma(15); // Custom period: 15
    let rsi_21 = chart.rsi(21).unwrap(); // Custom period: 21
    let macd = chart.macd(12, 26, 9).unwrap(); // Custom MACD parameters

    // Access the last value
    if let Some(&last_sma) = sma_15.last().and_then(|v| v.as_ref()) {
        println!("Latest SMA(15): {:.2}", last_sma);
    }

    // Candlestick patterns (same chart, no extra request)
    let signals = chart.patterns();

    let _ = rsi_21;
    let _ = macd;
    let _ = signals;
}

// ---------------------------------------------------------------------------
// Network tests — Direct Indicator Functions from indicators.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_direct_indicator_functions() {
    use finance_query::indicators::{macd, rsi, sma};
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "Direct Indicator Functions" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    // Extract price data (convenience methods on Chart)
    let closes: Vec<f64> = chart.close_prices();
    let _highs: Vec<f64> = chart.high_prices();
    let _lows: Vec<f64> = chart.low_prices();

    // Calculate indicators directly
    let sma_25 = sma(&closes, 25); // Returns Vec<Option<f64>>
    let rsi_10 = rsi(&closes, 10).unwrap(); // Returns Result<Vec<Option<f64>>>
    let macd_result = macd(&closes, 12, 26, 9).unwrap(); // Returns Result<MacdResult>

    // Access results
    if let Some(&last_rsi) = rsi_10.last().and_then(|v| v.as_ref()) {
        println!("RSI(10): {:.2}", last_rsi);
    }

    // MACD returns a struct with three series
    if let Some(&last_macd) = macd_result.macd_line.last().and_then(|v| v.as_ref()) {
        println!("MACD Line: {:.4}", last_macd);
    }

    let _ = sma_25;
}

// ---------------------------------------------------------------------------
// Network tests — Compound Indicators from indicators.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_compound_indicators() {
    use finance_query::indicators::{bollinger_bands, macd, stochastic};
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "Working with Compound Indicators" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    let closes = chart.close_prices();
    let highs = chart.high_prices();
    let lows = chart.low_prices();

    // Bollinger Bands - returns BollingerBands struct
    let bb = bollinger_bands(&closes, 20, 2.0).unwrap();
    if let Some(&upper) = bb.upper.last().and_then(|v| v.as_ref())
        && let Some(&middle) = bb.middle.last().and_then(|v| v.as_ref())
        && let Some(&lower) = bb.lower.last().and_then(|v| v.as_ref())
    {
        println!(
            "BB: Upper={:.2}, Middle={:.2}, Lower={:.2}",
            upper, middle, lower
        );
    }

    // Stochastic Oscillator - returns StochasticResult struct
    let stoch = stochastic(&highs, &lows, &closes, 14, 1, 3).unwrap();
    if let Some(&k) = stoch.k.last().and_then(|v| v.as_ref())
        && let Some(&d) = stoch.d.last().and_then(|v| v.as_ref())
    {
        println!("Stochastic: %K={:.2}, %D={:.2}", k, d);
    }

    // MACD - returns MacdResult struct
    let macd_data = macd(&closes, 12, 26, 9).unwrap();
    if let Some(&line) = macd_data.macd_line.last().and_then(|v| v.as_ref())
        && let Some(&signal) = macd_data.signal_line.last().and_then(|v| v.as_ref())
        && let Some(&hist) = macd_data.histogram.last().and_then(|v| v.as_ref())
    {
        println!(
            "MACD: Line={:.4}, Signal={:.4}, Histogram={:.4}",
            line, signal, hist
        );
    }
}

// ---------------------------------------------------------------------------
// Network tests — Candlestick Patterns from indicators.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_candlestick_patterns_via_chart() {
    use finance_query::indicators::{CandlePattern, PatternSentiment, patterns};
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "Candlestick Patterns" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    // Via Chart extension method — same length as chart.candles
    let signals = chart.patterns();

    // Or call the function directly with a candle slice
    let _signals2 = patterns(&chart.candles);

    // Each slot is Some(pattern) or None; iterate with candles for context
    for (candle, pattern) in chart.candles.iter().zip(signals.iter()) {
        if let Some(p) = pattern {
            println!(
                "timestamp={}: {:?} ({:?})",
                candle.timestamp,
                p,
                p.sentiment()
            );
        }
    }

    let _ = CandlePattern::Doji; // confirm variant access
    let _ = PatternSentiment::Bullish; // confirm variant access
}

// ---------------------------------------------------------------------------
// Network tests — PatternSentiment from indicators.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_pattern_sentiment_filtering() {
    use finance_query::indicators::PatternSentiment;
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "Using PatternSentiment" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();
    let signals = chart.patterns();

    let bullish = signals
        .iter()
        .filter(|s| {
            s.map(|p| p.sentiment() == PatternSentiment::Bullish)
                .unwrap_or(false)
        })
        .count();

    let bearish = signals
        .iter()
        .filter(|s| {
            s.map(|p| p.sentiment() == PatternSentiment::Bearish)
                .unwrap_or(false)
        })
        .count();

    println!("Bull/Bear ratio: {}/{}", bullish, bearish);
}

// ---------------------------------------------------------------------------
// Network tests — Combining Patterns with Indicators from indicators.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_patterns_combined_with_indicators() {
    use finance_query::indicators::PatternSentiment;
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "Combining Patterns with Indicators" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();
    let rsi = chart.rsi(14).unwrap();
    let signals = chart.patterns();

    // Find bars where RSI is oversold AND a bullish pattern just completed
    for (i, (pattern, rsi_val)) in signals.iter().zip(rsi.iter()).enumerate() {
        let is_bullish_pattern = pattern
            .map(|p| p.sentiment() == PatternSentiment::Bullish)
            .unwrap_or(false);
        let is_oversold = rsi_val.map(|r| r < 30.0).unwrap_or(false);

        if is_bullish_pattern && is_oversold {
            println!(
                "Strong buy signal at bar {}: {:?} with RSI={:.1}",
                i,
                pattern.unwrap(),
                rsi_val.unwrap()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Network tests — Working with Indicator Results from indicators.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_indicator_results_compound_types() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "Working with Indicator Results" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let indicators = ticker
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    // Simple indicators (Option<f64>)
    if let Some(rsi) = indicators.rsi_14 {
        println!("RSI(14): {:.2}", rsi);
        if rsi < 30.0 {
            println!("  Oversold");
        } else if rsi > 70.0 {
            println!("  Overbought");
        }
    }

    // Moving averages
    if let Some(sma200) = indicators.sma_200 {
        println!("SMA(200): {:.2}", sma200);
    }

    // MACD (compound - MacdData struct)
    if let Some(macd) = indicators.macd {
        if let Some(line) = macd.macd {
            println!("MACD Line: {:.4}", line);
        }
        if let Some(signal) = macd.signal {
            println!("Signal: {:.4}", signal);
        }
        if let Some(histogram) = macd.histogram {
            println!("Histogram: {:.4}", histogram);
        }
    }

    // Stochastic (StochasticData struct)
    if let Some(stoch) = indicators.stochastic {
        if let Some(k) = stoch.k {
            println!("%K: {:.2}", k);
        }
        if let Some(d) = stoch.d {
            println!("%D: {:.2}", d);
        }
    }

    // Bollinger Bands (BollingerBandsData struct)
    if let Some(bb) = indicators.bollinger_bands {
        if let Some(upper) = bb.upper {
            println!("Upper: {:.2}", upper);
        }
        if let Some(middle) = bb.middle {
            println!("Middle: {:.2}", middle);
        }
        if let Some(lower) = bb.lower {
            println!("Lower: {:.2}", lower);
        }
    }

    // Aroon (AroonData struct)
    if let Some(aroon) = indicators.aroon {
        if let Some(up) = aroon.aroon_up {
            println!("Aroon Up: {:.2}", up);
        }
        if let Some(down) = aroon.aroon_down {
            println!("Aroon Down: {:.2}", down);
        }
    }

    // Ichimoku (IchimokuData struct)
    if let Some(ichimoku) = indicators.ichimoku {
        if let Some(conversion) = ichimoku.conversion_line {
            println!("Conversion Line: {:.2}", conversion);
        }
        if let Some(base) = ichimoku.base_line {
            println!("Base Line: {:.2}", base);
        }
    }
}

// ---------------------------------------------------------------------------
// Network tests — DataFrame conversion from indicators.md (dataframe feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "dataframe")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_indicators_to_dataframe() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "Converting to DataFrame" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let indicators = ticker
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    let df = indicators.to_dataframe().unwrap();
    println!("{}", df);
    assert!(df.height() > 0 || df.width() > 0);
}

// ---------------------------------------------------------------------------
// Network tests — Caching Behavior from indicators.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_indicators_caching() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "Caching Behavior" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    // First call fetches and caches
    let ind1 = ticker
        .indicators(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    // Second call returns cached result
    let ind2 = ticker
        .indicators(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    // Different range: fetches new data
    let ind3 = ticker
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    let _ = ind1;
    let _ = ind2;
    let _ = ind3;
}

// ---------------------------------------------------------------------------
// Network tests — Common Patterns from indicators.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_trend_confirmation_with_multiple_mas() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "Trend Confirmation with Multiple MAs" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let indicators = ticker
        .indicators(Interval::OneDay, TimeRange::OneYear)
        .await
        .unwrap();

    let sma_200 = indicators.sma_200.unwrap_or(0.0);
    let ema_50 = indicators.ema_50.unwrap_or(0.0);
    let ema_20 = indicators.ema_20.unwrap_or(0.0);

    if ema_20 > ema_50 && ema_50 > sma_200 {
        println!("Uptrend confirmed");
    }

    println!(
        "SMA(200): {:.2}, EMA(50): {:.2}, EMA(20): {:.2}",
        sma_200, ema_50, ema_20
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_rsi_extremes() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "RSI Extremes" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let indicators = ticker
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    if let Some(rsi) = indicators.rsi_14 {
        if rsi < 30.0 {
            println!("Oversold");
        } else if rsi > 70.0 {
            println!("Overbought");
        }
        println!("RSI: {:.2}", rsi);
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_macd_crossover() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "MACD Crossover" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let indicators = ticker
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    if let Some(macd) = indicators.macd
        && let (Some(line), Some(signal)) = (macd.macd, macd.signal)
    {
        if line > signal {
            println!("Bullish MACD crossover");
        } else {
            println!("Bearish MACD crossover");
        }
    }
}

// ---------------------------------------------------------------------------
// Network tests — Best Practices pattern from indicators.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_indicators_best_practices_pattern() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From indicators.md "Best Practices" section — store result once
    let ticker = Ticker::new("AAPL").await.unwrap();
    let indicators = ticker
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    if let Some(rsi) = indicators.rsi_14
        && rsi < 30.0
    {
        // Oversold - check other indicators from same result
        if let Some(ref macd) = indicators.macd
            && let (Some(line), Some(signal)) = (macd.macd, macd.signal)
            && line > signal
        {
            println!("Potential buy: RSI oversold + MACD bullish");
        }
    }
}

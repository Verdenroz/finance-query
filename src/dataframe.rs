use finance_query_types::__dataframe::stack_rows;
use finance_query_types::ToDataFrame;
use polars::prelude::{DataFrame, PolarsResult};

macro_rules! delegate_to_inherent {
    ($($ty:ty),* $(,)?) => {
        $(
            impl ToDataFrame for $ty {
                fn to_dataframe(&self) -> PolarsResult<DataFrame> {
                    <$ty>::to_dataframe(self)
                }
            }
        )*
    };
}

macro_rules! delegate_rows_to_inherent {
    ($($ty:ty),* $(,)?) => {
        $(
            impl ToDataFrame for $ty {
                fn to_dataframe(&self) -> PolarsResult<DataFrame> {
                    <$ty>::to_dataframe(self)
                }

                fn vec_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
                    <$ty>::vec_to_dataframe(items)
                }
            }
        )*
    };
}

macro_rules! delegate_wrapped_rows {
    ($($ty:ty => $row:ty),* $(,)?) => {
        $(
            impl ToDataFrame for $ty {
                fn to_dataframe(&self) -> PolarsResult<DataFrame> {
                    <$ty>::to_dataframe(self)
                }

                fn vec_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
                    stack_rows(<$row>::vec_to_dataframe(&[]), items, Self::to_dataframe)
                }

                fn vec_to_nested_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
                    stack_rows(
                        <$row as ToDataFrame>::vec_to_nested_dataframe(&[]),
                        items,
                        <Self as ToDataFrame>::to_nested_dataframe,
                    )
                }
            }
        )*
    };
}

impl<T: ToDataFrame> ToDataFrame for crate::ProviderPage<T> {
    fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        T::vec_to_dataframe(&self.items)
    }

    fn vec_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
        stack_rows(T::vec_to_dataframe(&[]), items, Self::to_dataframe)
    }

    fn to_nested_dataframe(&self) -> PolarsResult<DataFrame> {
        T::vec_to_nested_dataframe(&self.items)
    }

    fn vec_to_nested_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
        stack_rows(
            T::vec_to_nested_dataframe(&[]),
            items,
            Self::to_nested_dataframe,
        )
    }
}

/// Long-format frame of untyped Yahoo statement entries: one row per numeric field.
///
/// The period comes from `period_key` (a `{raw}` object, a number or a numeric
/// string); fields that are neither a number nor a `{raw}` number are skipped.
fn metric_rows(
    entries: &[serde_json::Value],
    period_key: &str,
    period_column: &str,
) -> PolarsResult<DataFrame> {
    use polars::prelude::{NamedFrom, Series};
    use serde_json::Value;

    fn number(value: &Value) -> Option<&serde_json::Number> {
        match value {
            Value::Number(n) => Some(n),
            Value::Object(o) => o.get("raw").and_then(Value::as_number),
            _ => None,
        }
    }

    let mut periods = Vec::new();
    let mut metrics = Vec::new();
    let mut values = Vec::new();
    for entry in entries.iter().filter_map(Value::as_object) {
        let period = entry.get(period_key).and_then(|value| match value {
            Value::String(s) => s.parse().ok(),
            other => {
                number(other).and_then(|n| n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)))
            }
        });
        for (key, value) in entry {
            if key == period_key || key == "maxAge" {
                continue;
            }
            if let Some(n) = number(value).and_then(serde_json::Number::as_f64) {
                periods.push(period);
                metrics.push(key.as_str());
                values.push(n);
            }
        }
    }

    DataFrame::new(
        values.len(),
        vec![
            Series::new(period_column.into(), periods).into(),
            Series::new("metric".into(), metrics).into(),
            Series::new("value".into(), values).into(),
        ],
    )
}

macro_rules! impl_metric_rows {
    ($($ty:ty => $field:ident, $key:literal, $column:literal),* $(,)?) => {
        $(
            impl $ty {
                /// Converts the entries to a long-format polars DataFrame, one row per numeric field.
                ///
                /// Columns are the entry's period (i64, null when absent), `metric` (the
                /// Yahoo field name) and `value` (f64, the `raw` of `{raw, fmt}` fields).
                /// An empty or missing list yields a zero-row frame with the same columns.
                pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
                    metric_rows(self.$field.as_deref().unwrap_or_default(), $key, $column)
                }
            }

            impl ToDataFrame for $ty {
                fn to_dataframe(&self) -> PolarsResult<DataFrame> {
                    <$ty>::to_dataframe(self)
                }

                fn vec_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame> {
                    stack_rows(metric_rows(&[], $key, $column), items, Self::to_dataframe)
                }
            }
        )*
    };
}

impl_metric_rows!(
    crate::models::fundamentals::BalanceSheetHistory => balance_sheet_statements, "endDate", "end_date",
    crate::models::fundamentals::BalanceSheetHistoryQuarterly => balance_sheet_statements, "endDate", "end_date",
    crate::models::fundamentals::CashflowStatementHistory => cashflow_statements, "endDate", "end_date",
    crate::models::fundamentals::CashflowStatementHistoryQuarterly => cashflow_statements, "endDate", "end_date",
    crate::models::fundamentals::IncomeStatementHistory => income_statement_history, "endDate", "end_date",
    crate::models::fundamentals::IncomeStatementHistoryQuarterly => income_statement_history, "endDate", "end_date",
    crate::models::corporate::fund_performance::PastQuarterlyReturns => returns, "year", "year",
);

delegate_rows_to_inherent!(
    crate::CalendarEvent,
    crate::MarketCalendarEntry,
    crate::models::chart::spark::Spark,
    crate::streaming::AlertEvent,
    crate::SparkData,
    crate::EdgarSearchHit,
);

delegate_to_inherent!(
    crate::BatchCapitalGainsResponse,
    crate::BatchChartsResponse,
    crate::BatchDividendsResponse,
    crate::BatchFinancialsResponse,
    crate::BatchNewsResponse,
    crate::BatchOptionsResponse,
    crate::BatchQuotesResponse,
    crate::BatchRecommendationsResponse,
    crate::BatchSparksResponse,
    crate::BatchSplitsResponse,
    crate::Chart,
    crate::CompanyFacts,
    crate::Contracts,
    crate::EconomicSeries,
    crate::EdgarFilingIndex,
    crate::FactConcept,
    crate::FactsByTaxonomy,
    crate::FinancialStatement,
    crate::LookupResults,
    crate::MarketHours,
    crate::Options,
    crate::ProviderFilings,
    crate::Recommendation,
    crate::ScreenerResults,
);

delegate_wrapped_rows!(
    crate::EdgarFilingRecent => crate::EdgarFiling,
    crate::EdgarSearchHitsContainer => crate::EdgarSearchHit,
    crate::EdgarSearchResults => crate::EdgarSearchHit,
    crate::ResearchReports => crate::ResearchReport,
    crate::SearchQuotes => crate::SearchQuote,
);

#[cfg(feature = "cftc")]
delegate_to_inherent!(crate::models::futures::cot::CommitmentsOfTraders);

#[cfg(feature = "fred")]
delegate_to_inherent!(crate::models::economic::MacroSeries);

#[cfg(feature = "indicators")]
delegate_to_inherent!(crate::BatchIndicatorsResponse);

#[cfg(test)]
mod tests {
    use polars::prelude::{DataFrame, DataType};

    use crate::{Candle, Dividend, News, Provider};

    fn schema(df: &DataFrame) -> Vec<(String, DataType)> {
        df.schema()
            .iter()
            .map(|(name, dtype)| (name.to_string(), dtype.clone()))
            .collect()
    }

    fn columns(expected: &[(&str, DataType)]) -> Vec<(String, DataType)> {
        expected
            .iter()
            .map(|(name, dtype)| (name.to_string(), dtype.clone()))
            .collect()
    }

    #[test]
    fn candle_frame_adds_provider_id_after_existing_columns() {
        let candle = Candle::__new(0, 0.0, 0.0, 0.0, 0.0, 0, None, Some(Provider::Yahoo));
        let df = candle.to_dataframe().unwrap();
        assert_eq!(
            schema(&df),
            columns(&[
                ("timestamp", DataType::Int64),
                ("open", DataType::Float64),
                ("high", DataType::Float64),
                ("low", DataType::Float64),
                ("close", DataType::Float64),
                ("volume", DataType::Int64),
                ("adj_close", DataType::Float64),
                ("provider_id", DataType::String),
            ])
        );
        assert_eq!(
            df.column("provider_id").unwrap().str().unwrap().get(0),
            Some(Provider::Yahoo.as_str())
        );
    }

    #[test]
    fn vec_and_slice_convert_with_method_syntax() {
        use crate::prelude::*;

        let candles: Vec<Candle> = (0..3)
            .map(|i| Candle::__new(i, 1.0, 2.0, 0.5, 1.5, 10, None, None))
            .collect();
        let expected = Candle::vec_to_dataframe(&candles).unwrap();
        let from_vec = candles.to_dataframe().unwrap();
        let from_slice = candles[..2].to_dataframe().unwrap();
        assert_eq!(schema(&from_vec), schema(&expected));
        assert_eq!(from_vec.height(), 3);
        assert_eq!(from_slice.height(), 2);
    }

    #[test]
    fn empty_vec_keeps_the_row_schema() {
        use crate::prelude::*;

        let empty: Vec<Candle> = Vec::new();
        let df = empty.to_dataframe().unwrap();
        assert_eq!(df.height(), 0);
        assert_eq!(schema(&df), schema(&Candle::vec_to_dataframe(&[]).unwrap()));
    }

    #[test]
    fn statement_entries_convert_to_long_metric_rows() {
        use crate::models::fundamentals::IncomeStatementHistory;

        let history: IncomeStatementHistory = serde_json::from_value(serde_json::json!({
            "maxAge": 1,
            "incomeStatementHistory": [
                {"maxAge": 1, "endDate": {"raw": 100, "fmt": "x"},
                 "totalRevenue": {"raw": 5.0, "fmt": "5"}, "netIncome": 2, "label": "skip"},
                {"totalRevenue": {"raw": 6.0}}
            ]
        }))
        .unwrap();
        let df = history.to_dataframe().unwrap();
        assert_eq!(
            schema(&df),
            columns(&[
                ("end_date", DataType::Int64),
                ("metric", DataType::String),
                ("value", DataType::Float64),
            ])
        );
        assert_eq!(df.height(), 3);
        assert_eq!(df.column("end_date").unwrap().null_count(), 1);

        let empty: IncomeStatementHistory = serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(schema(&empty.to_dataframe().unwrap()), schema(&df));
        assert_eq!(
            <IncomeStatementHistory as super::ToDataFrame>::vec_to_dataframe(&[history])
                .unwrap()
                .height(),
            3
        );
    }

    #[test]
    fn past_quarterly_returns_use_the_year_as_period() {
        use crate::models::corporate::fund_performance::PastQuarterlyReturns;

        let returns: PastQuarterlyReturns = serde_json::from_value(serde_json::json!({
            "returns": [{"year": "2024", "q1": {"raw": 0.1, "fmt": "10%"}, "q2": {"raw": 0.2}}]
        }))
        .unwrap();
        let df = returns.to_dataframe().unwrap();
        assert_eq!(
            schema(&df),
            columns(&[
                ("year", DataType::Int64),
                ("metric", DataType::String),
                ("value", DataType::Float64),
            ])
        );
        assert_eq!(df.height(), 2);
        assert_eq!(df.column("year").unwrap().i64().unwrap().get(0), Some(2024));
    }

    #[test]
    fn dividend_frame_adds_provider_id() {
        let dividend: Dividend =
            serde_json::from_value(serde_json::json!({"timestamp": 1, "amount": 0.25})).unwrap();
        let df = Dividend::vec_to_dataframe(&[dividend]).unwrap();
        assert_eq!(
            schema(&df),
            columns(&[
                ("timestamp", DataType::Int64),
                ("amount", DataType::Float64),
                ("provider_id", DataType::String),
            ])
        );
        assert_eq!(df.column("provider_id").unwrap().null_count(), 1);
    }

    #[test]
    fn news_frame_adds_provider_id() {
        let news = News::new(
            "title".into(),
            "link".into(),
            "source".into(),
            "img".into(),
            "time".into(),
        );
        let df = news.to_dataframe().unwrap();
        assert_eq!(
            schema(&df)[..6],
            columns(&[
                ("title", DataType::String),
                ("link", DataType::String),
                ("source", DataType::String),
                ("img", DataType::String),
                ("time", DataType::String),
                ("provider_id", DataType::String),
            ])
        );
    }

    #[cfg(feature = "sentiment")]
    #[test]
    fn news_frames_flatten_sentiment() {
        let sentiment = columns(&[
            ("sentiment_label", DataType::String),
            ("sentiment_score", DataType::Float64),
            ("sentiment_confidence", DataType::Float64),
        ]);
        let news = News::new(
            "title".into(),
            "link".into(),
            "source".into(),
            "img".into(),
            "time".into(),
        );
        assert_eq!(schema(&news.to_dataframe().unwrap())[6..], sentiment);

        let search = crate::SearchNews::vec_to_dataframe(&[]).unwrap();
        let search = schema(&search);
        assert_eq!(search[search.len() - 4..search.len() - 1], sentiment);
        assert_eq!(
            search[search.len() - 1],
            (
                "related_tickers".into(),
                DataType::List(Box::new(DataType::String))
            )
        );
    }

    #[test]
    fn default_vec_to_dataframe_skips_items_without_columns() {
        let concepts: Vec<crate::FactConcept> = serde_json::from_value(serde_json::json!([
            {"label": "Revenue", "units": {"USD": [{"end": "2024-12-31", "val": 1.0}]}},
            {"label": "Unitless"},
        ]))
        .unwrap();
        let df = <crate::FactConcept as super::ToDataFrame>::vec_to_dataframe(&concepts).unwrap();
        assert_eq!(df.height(), 1);
        let unitless =
            <crate::FactConcept as super::ToDataFrame>::vec_to_dataframe(&concepts[1..]).unwrap();
        assert_eq!(unitless.width(), 0);
    }

    #[test]
    fn calendar_rows_keep_schema_through_the_trait() {
        let events = <crate::CalendarEvent as super::ToDataFrame>::vec_to_dataframe(&[]).unwrap();
        assert_eq!(
            schema(&events),
            schema(&crate::CalendarEvent::vec_to_dataframe(&[]).unwrap())
        );
        assert!(events.width() > 0);
        let entries =
            <crate::MarketCalendarEntry as super::ToDataFrame>::vec_to_dataframe(&[]).unwrap();
        assert!(entries.width() > 0);
    }

    #[test]
    fn row_wrappers_convert_through_the_trait() {
        fn frame<T: super::ToDataFrame>(value: &T) -> DataFrame {
            value.to_dataframe().unwrap()
        }
        let history: crate::quote::EarningsHistory =
            serde_json::from_value(serde_json::json!({"history": []})).unwrap();
        assert_eq!(
            schema(&frame(&history)),
            schema(&history.to_dataframe().unwrap())
        );
    }

    #[cfg(feature = "indicators")]
    #[test]
    fn indicators_summary_frame_flattens_composite_indicators() {
        let summary = crate::indicators::IndicatorsSummary {
            macd: Some(crate::indicators::MacdData {
                macd: Some(1.0),
                signal: Some(0.5),
                histogram: Some(0.5),
            }),
            ..Default::default()
        };
        let df = summary.to_dataframe().unwrap();
        let schema = df.schema();
        let expected = [
            ("sma_10", DataType::Float64),
            ("stochastic_k", DataType::Float64),
            ("stochastic_rsi_d", DataType::Float64),
            ("macd_macd", DataType::Float64),
            ("macd_histogram", DataType::Float64),
            ("aroon_aroon_up", DataType::Float64),
            ("supertrend_trend", DataType::String),
            ("ichimoku_lagging_span", DataType::Float64),
            ("bull_bear_power_bear_power", DataType::Float64),
            ("elder_ray_index_bull_power", DataType::Float64),
            ("bollinger_bands_upper", DataType::Float64),
            ("keltner_channels_lower", DataType::Float64),
            ("donchian_channels_middle", DataType::Float64),
            ("pivot_points_pivot", DataType::Float64),
            ("fibonacci_pivot_points_r3", DataType::Float64),
            ("heikin_ashi_close", DataType::Float64),
            ("heikin_ashi_provider_id", DataType::String),
            ("zigzag_last_pivot_index", DataType::UInt64),
            ("zigzag_last_pivot_is_high", DataType::Boolean),
            ("fibonacci_retracement_50_level_61_8", DataType::Float64),
        ];
        for (name, dtype) in expected {
            assert_eq!(schema.get(name), Some(&dtype), "{name}");
        }
        assert_eq!(
            df.column("macd_signal").unwrap().f64().unwrap().get(0),
            Some(0.5)
        );
        assert_eq!(df.column("pivot_points_pivot").unwrap().null_count(), 1);
    }
}

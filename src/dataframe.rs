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

/// Stacks one frame per item under the row type's zero-row schema.
pub(crate) fn stack_rows<T>(
    schema: PolarsResult<DataFrame>,
    items: &[T],
    frame: impl Fn(&T) -> PolarsResult<DataFrame>,
) -> PolarsResult<DataFrame> {
    items
        .iter()
        .map(frame)
        .try_fold(schema?, |acc, frame| acc.vstack(&frame?))
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

delegate_rows_to_inherent!(crate::CalendarEvent, crate::MarketCalendarEntry);

delegate_to_inherent!(
    crate::BatchCapitalGainsResponse,
    crate::BatchChartsResponse,
    crate::BatchDividendsResponse,
    crate::BatchNewsResponse,
    crate::BatchOptionsResponse,
    crate::BatchQuotesResponse,
    crate::BatchRecommendationsResponse,
    crate::BatchSplitsResponse,
    crate::Chart,
    crate::Contracts,
    crate::EconomicSeries,
    crate::EdgarFilingIndex,
    crate::EdgarSearchResults,
    crate::FactConcept,
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
    crate::ResearchReports => crate::ResearchReport,
    crate::SearchQuotes => crate::SearchQuote,
);

#[cfg(feature = "cftc")]
delegate_to_inherent!(crate::models::futures::cot::CommitmentsOfTraders);

#[cfg(feature = "fred")]
delegate_to_inherent!(crate::models::economic::MacroSeries);

#[cfg(feature = "indicators")]
delegate_to_inherent!(crate::BatchIndicatorsResponse);

#[cfg(feature = "backtesting")]
delegate_to_inherent!(crate::backtesting::PerformanceMetrics);

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

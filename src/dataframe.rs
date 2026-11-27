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

delegate_rows_to_inherent!(crate::streaming::AlertEvent);

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
);

#[cfg(feature = "indicators")]
delegate_to_inherent!(crate::BatchIndicatorsResponse);

#[cfg(test)]
mod tests {
    use polars::prelude::DataType;

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

    #[derive(crate::ToDataFrame)]
    #[dataframe(crate = "crate")]
    struct Generic<F: crate::models::format::Format = crate::format::Both> {
        #[dataframe(datetime = "s")]
        time: Option<F::Value<i64>>,
        #[dataframe(date = "s", nonzero)]
        day: Option<F::Value<i64>>,
    }

    #[test]
    fn format_value_epochs_become_time_columns() {
        let value = |raw| crate::FormattedValue {
            fmt: None,
            long_fmt: None,
            raw: Some(raw),
        };
        let df = Generic::<crate::format::Both> {
            time: Some(value(1)),
            day: Some(value(0)),
        }
        .to_dataframe()
        .unwrap();
        let schema = df.schema();
        assert_eq!(
            schema.get("time"),
            Some(&DataType::Datetime(
                polars::prelude::TimeUnit::Microseconds,
                Some(polars::prelude::TimeZone::UTC)
            ))
        );
        assert_eq!(schema.get("day"), Some(&DataType::Date));
        assert_eq!(df.column("day").unwrap().null_count(), 1);
    }
}

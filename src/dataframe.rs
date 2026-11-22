use polars::prelude::{DataFrame, PolarsResult};

/// Conversion of a model into a polars [`DataFrame`].
///
/// Implemented by every type with `#[derive(ToDataFrame)]` and by the
/// collection types that build a frame from their items (for example
/// [`Chart`](crate::Chart) and [`Options`](crate::Options)). Generic code can
/// take `T: ToDataFrame` instead of naming each model.
///
/// The derive keeps the inherent `to_dataframe` and `vec_to_dataframe`
/// methods, so calling them does not require this trait in scope.
///
/// ```no_run
/// use finance_query::{Candle, ToDataFrame};
///
/// fn frame_of<T: ToDataFrame>(items: &[T]) -> finance_query::polars::prelude::PolarsResult<usize> {
///     Ok(T::vec_to_dataframe(items)?.height())
/// }
///
/// # fn example(candles: Vec<Candle>) -> Result<(), Box<dyn std::error::Error>> {
/// let rows = frame_of(&candles)?;
/// # Ok(())
/// # }
/// ```
pub trait ToDataFrame {
    /// Converts this value to a polars DataFrame.
    fn to_dataframe(&self) -> PolarsResult<DataFrame>;

    /// Converts a slice of values to a single DataFrame.
    ///
    /// The default stacks each item's [`to_dataframe`](Self::to_dataframe)
    /// output vertically. For an empty slice it returns a frame with no
    /// columns; derived impls return a zero-row frame with the full schema.
    fn vec_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame>
    where
        Self: Sized,
    {
        let mut frames = items.iter().map(Self::to_dataframe);
        match frames.next() {
            None => Ok(DataFrame::default()),
            Some(first) => frames.try_fold(first?, |acc, frame| acc.vstack(&frame?)),
        }
    }
}

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

delegate_to_inherent!(
    crate::Chart,
    crate::Contracts,
    crate::EdgarFilingRecent,
    crate::EdgarSearchResults,
    crate::FactConcept,
    crate::LookupResults,
    crate::MarketHours,
    crate::Options,
    crate::Recommendation,
    crate::ResearchReports,
    crate::ScreenerResults,
    crate::SearchNewsList,
    crate::SearchQuotes,
);

/// Support code for `#[derive(ToDataFrame)]`. Not public API.
#[doc(hidden)]
pub mod derive {
    use std::fmt::Display;
    use std::marker::PhantomData;

    use polars::prelude::{Column, IntoSeries, ListBuilderTrait, ListStringChunkedBuilder};

    pub trait Columns {
        fn columns(rows: &[Option<&Self>], prefix: &str) -> Vec<Column>;
    }

    pub fn string_list<'a>(
        name: String,
        values: impl Iterator<Item = Option<&'a Vec<String>>>,
    ) -> Column {
        let mut builder = ListStringChunkedBuilder::new(name.into(), 0, 0);
        for value in values {
            match value {
                Some(list) => builder.append_values_iter(list.iter().map(String::as_str)),
                None => builder.append_null(),
            }
        }
        Column::from(builder.finish().into_series())
    }

    /// Selects a column builder by whether `T: Display`, resolved by autoref at the call site.
    pub struct Probe<T>(pub PhantomData<T>);

    pub trait ViaDisplay {
        fn kind(&self) -> DisplayColumn {
            DisplayColumn
        }
    }

    impl<T: Display> ViaDisplay for &Probe<T> {}

    pub trait ViaSkip {
        fn kind(&self) -> SkipColumn {
            SkipColumn
        }
    }

    impl<T> ViaSkip for Probe<T> {}

    pub struct DisplayColumn;

    impl DisplayColumn {
        pub fn column<'a, T: Display + 'a>(
            self,
            name: String,
            values: impl Iterator<Item = Option<&'a T>>,
        ) -> Option<Column> {
            let values: Vec<Option<String>> = values.map(|v| v.map(T::to_string)).collect();
            Some(Column::new(name.into(), values))
        }
    }

    pub struct SkipColumn;

    impl SkipColumn {
        pub fn column<'a, T: 'a>(
            self,
            _name: String,
            _values: impl Iterator<Item = Option<&'a T>>,
        ) -> Option<Column> {
            None
        }
    }
}

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
        let candle = Candle {
            provider_id: Some(Provider::Yahoo),
            ..Candle::default()
        };
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
            schema(&df),
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

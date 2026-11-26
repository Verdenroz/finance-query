use polars::prelude::{Column, DataFrame, NamedFrom, PolarsResult, Series};

use super::{
    AroonResult, BollingerBands, BullBearPowerResult, DonchianChannelsResult, IchimokuResult,
    KeltnerChannelsResult, MacdResult, StochasticResult, SuperTrendResult,
};
use crate::ToDataFrame;

fn column<T>(name: &str, values: &[Option<T>], rows: usize) -> Column
where
    T: Copy,
    Series: NamedFrom<Vec<Option<T>>, [Option<T>]>,
{
    let mut padded = values.to_vec();
    padded.resize(rows, None);
    Column::from(Series::new(name.into(), padded))
}

macro_rules! series_frames {
    ($($ty:ty { $($field:ident: $dtype:ty),+ $(,)? })*) => {
        $(
            impl $ty {
                /// Converts the series to a polars DataFrame, one row per bar.
                ///
                /// Each series is a nullable column named after its field. Series
                /// shorter than the longest are padded with trailing nulls; equal
                /// lengths are left as they are. There is no timestamp column, so
                /// `hstack` the frame onto the candle frame it was computed from.
                pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
                    let rows = [$(self.$field.len()),+].into_iter().max().unwrap_or(0);
                    DataFrame::new(
                        rows,
                        vec![$(column::<$dtype>(stringify!($field), &self.$field, rows)),+],
                    )
                }
            }

            impl ToDataFrame for $ty {
                fn to_dataframe(&self) -> PolarsResult<DataFrame> {
                    <$ty>::to_dataframe(self)
                }
            }
        )*
    };
}

series_frames! {
    MacdResult { macd_line: f64, signal_line: f64, histogram: f64 }
    AroonResult { aroon_up: f64, aroon_down: f64 }
    DonchianChannelsResult { upper: f64, middle: f64, lower: f64 }
    BollingerBands { upper: f64, middle: f64, lower: f64 }
    StochasticResult { k: f64, d: f64 }
    BullBearPowerResult { bull_power: f64, bear_power: f64 }
    SuperTrendResult { value: f64, is_uptrend: bool }
    IchimokuResult {
        conversion_line: f64,
        base_line: f64,
        leading_span_a: f64,
        leading_span_b: f64,
        lagging_span: f64,
    }
    KeltnerChannelsResult { upper: f64, middle: f64, lower: f64 }
}

#[cfg(test)]
mod tests {
    use polars::prelude::DataType;

    use super::*;

    fn dtypes(df: &DataFrame) -> Vec<(String, DataType)> {
        df.schema()
            .iter()
            .map(|(name, dtype)| (name.to_string(), dtype.clone()))
            .collect()
    }

    #[test]
    fn macd_frame_has_one_row_per_bar() {
        let result = MacdResult {
            macd_line: vec![None, Some(1.0)],
            signal_line: vec![None, None],
            histogram: vec![None, Some(0.5)],
        };
        let df = result.to_dataframe().unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(
            dtypes(&df),
            [
                ("macd_line".to_string(), DataType::Float64),
                ("signal_line".to_string(), DataType::Float64),
                ("histogram".to_string(), DataType::Float64),
            ]
        );
        assert_eq!(df.column("macd_line").unwrap().null_count(), 1);
        assert_eq!(df.column("signal_line").unwrap().null_count(), 2);
    }

    #[test]
    fn empty_series_keep_schema() {
        let result = StochasticResult {
            k: vec![],
            d: vec![],
        };
        let df = ToDataFrame::to_dataframe(&result).unwrap();
        assert_eq!(df.height(), 0);
        assert_eq!(df.get_column_names(), ["k", "d"]);
    }

    #[test]
    fn supertrend_trend_column_is_boolean() {
        let result = SuperTrendResult {
            value: vec![Some(10.0)],
            is_uptrend: vec![Some(true)],
        };
        let df = result.to_dataframe().unwrap();
        assert_eq!(df.schema().get("is_uptrend"), Some(&DataType::Boolean));
        assert_eq!(df.schema().get("value"), Some(&DataType::Float64));
    }

    #[test]
    fn shorter_series_are_padded_with_nulls() {
        let result = IchimokuResult {
            conversion_line: vec![Some(1.0), Some(2.0)],
            base_line: vec![Some(1.0), Some(2.0)],
            leading_span_a: vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0)],
            leading_span_b: vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0)],
            lagging_span: vec![Some(1.0)],
        };
        let df = result.to_dataframe().unwrap();
        assert_eq!(df.height(), 4);
        assert_eq!(df.column("conversion_line").unwrap().null_count(), 2);
        assert_eq!(df.column("lagging_span").unwrap().null_count(), 3);
        assert_eq!(df.column("leading_span_a").unwrap().null_count(), 0);
    }

    #[test]
    fn equal_length_series_are_not_padded() {
        let result = BollingerBands {
            upper: vec![Some(3.0), Some(4.0)],
            middle: vec![Some(2.0), Some(3.0)],
            lower: vec![Some(1.0), Some(2.0)],
        };
        let df = result.to_dataframe().unwrap();
        assert_eq!(df.height(), 2);
        assert!(df.columns().iter().all(|c| c.null_count() == 0));
    }
}

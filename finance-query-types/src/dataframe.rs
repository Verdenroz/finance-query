use polars::prelude::{DataFrame, PolarsResult};

/// Conversion of a model into a polars [`DataFrame`].
///
/// Implemented by every type with `#[derive(ToDataFrame)]` and by the
/// collection types that build a frame from their items (for example
/// `Chart` and `Options`). Generic code can take `T: ToDataFrame` instead of
/// naming each model.
///
/// Slices and `Vec`s of any `ToDataFrame` type implement it too, so
/// `candles.to_dataframe()` stacks one row per item. Import the trait (or
/// `finance_query::prelude::*`) to call it with method syntax.
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
    /// output vertically, leaving out items whose frame has no columns. When
    /// no item has columns it returns a frame with no columns; derived impls
    /// return a zero-row frame with the full schema.
    fn vec_to_dataframe(items: &[Self]) -> PolarsResult<DataFrame>
    where
        Self: Sized,
    {
        let mut frames = items
            .iter()
            .map(Self::to_dataframe)
            .filter(|frame| frame.as_ref().map_or(true, |frame| frame.width() > 0));
        match frames.next() {
            None => Ok(DataFrame::default()),
            Some(first) => frames.try_fold(first?, |acc, frame| acc.vstack(&frame?)),
        }
    }

    /// Converts this value to a DataFrame that also carries its nested rows.
    ///
    /// Types with `#[dataframe(nested)]` fields add one `List(Struct)` column
    /// per field. The default is [`to_dataframe`](Self::to_dataframe).
    fn to_nested_dataframe(&self) -> PolarsResult<DataFrame> {
        self.to_dataframe()
    }

    /// Converts a slice of values to one DataFrame that also carries nested rows.
    ///
    /// The default is [`vec_to_dataframe`](Self::vec_to_dataframe).
    fn vec_to_nested_dataframe(items: &[Self]) -> PolarsResult<DataFrame>
    where
        Self: Sized,
    {
        Self::vec_to_dataframe(items)
    }
}

impl<T: ToDataFrame> ToDataFrame for [T] {
    fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        T::vec_to_dataframe(self)
    }

    fn to_nested_dataframe(&self) -> PolarsResult<DataFrame> {
        T::vec_to_nested_dataframe(self)
    }
}

impl<T: ToDataFrame> ToDataFrame for Vec<T> {
    fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        self.as_slice().to_dataframe()
    }

    fn to_nested_dataframe(&self) -> PolarsResult<DataFrame> {
        self.as_slice().to_nested_dataframe()
    }
}

/// Support code for `#[derive(ToDataFrame)]`. Not public API.
#[doc(hidden)]
pub mod derive {
    use std::fmt::Display;
    use std::marker::PhantomData;

    use chrono::{DateTime, NaiveDate};
    use polars::chunked_array::builder::{ListPrimitiveChunkedBuilder, get_list_builder};
    use polars::prelude::{
        Column, DataFrame, DataType, Int32Chunked, Int32Type, Int64Chunked, Int64Type, IntoColumn,
        IntoSeries, ListBuilderTrait, ListStringChunkedBuilder, NamedFrom, NewChunkedArray,
        PlSmallStr, PolarsResult, Series, TimeUnit, TimeZone,
    };

    use super::ToDataFrame;

    /// Stacks one frame per item under the row type's zero-row schema.
    pub fn stack_rows<T>(
        schema: PolarsResult<DataFrame>,
        items: &[T],
        frame: impl Fn(&T) -> PolarsResult<DataFrame>,
    ) -> PolarsResult<DataFrame> {
        items
            .iter()
            .map(frame)
            .try_fold(schema?, |acc, frame| acc.vstack(&frame?))
            .map(|mut stacked| {
                stacked.rechunk_mut();
                stacked
            })
    }

    /// Prepends a string column that repeats `value` on every row.
    pub fn with_leading_column(
        mut frame: DataFrame,
        name: &str,
        value: &str,
    ) -> PolarsResult<DataFrame> {
        let column = Series::new(name.into(), vec![value; frame.height()]);
        frame.insert_column(0, column.into())?;
        Ok(frame)
    }

    /// Collects `values` into a nullable column of `rows` entries, padding the tail with nulls.
    pub fn padded<T>(values: impl Iterator<Item = Option<T>>, rows: usize) -> Vec<Option<T>> {
        values
            .chain(std::iter::repeat_with(|| None))
            .take(rows)
            .collect()
    }

    /// Unit of an epoch integer.
    #[derive(Clone, Copy)]
    pub enum EpochUnit {
        /// Seconds since the Unix epoch.
        Seconds,
        /// Milliseconds since the Unix epoch.
        Millis,
        /// Microseconds since the Unix epoch.
        Micros,
        /// Nanoseconds since the Unix epoch.
        Nanos,
    }

    /// Time unit of every `Datetime` column.
    pub const DATETIME_UNIT: TimeUnit = TimeUnit::Microseconds;
    /// Time zone of every `Datetime` column.
    pub const DATETIME_ZONE: TimeZone = TimeZone::UTC;

    const UNIX_EPOCH_FROM_CE_DAYS: i32 = 719_163;

    fn datetime_dtype() -> DataType {
        DataType::Datetime(DATETIME_UNIT, Some(DATETIME_ZONE))
    }

    fn micros(value: i64, unit: EpochUnit) -> Option<i64> {
        match unit {
            EpochUnit::Seconds => value.checked_mul(1_000_000),
            EpochUnit::Millis => value.checked_mul(1_000),
            EpochUnit::Micros => Some(value),
            EpochUnit::Nanos => Some(value.div_euclid(1_000)),
        }
    }

    fn epoch_micros(value: Option<i64>, unit: EpochUnit, nonzero: bool) -> Option<i64> {
        value
            .filter(|value| !nonzero || *value != 0)
            .and_then(|value| micros(value, unit))
    }

    fn epoch_day(value: Option<i64>, unit: EpochUnit, nonzero: bool) -> Option<i32> {
        let per_day = match unit {
            EpochUnit::Seconds => 86_400,
            EpochUnit::Millis => 86_400_000,
            EpochUnit::Micros => 86_400_000_000,
            EpochUnit::Nanos => 86_400_000_000_000,
        };
        value
            .filter(|value| !nonzero || *value != 0)
            .and_then(|value| i32::try_from(value.div_euclid(per_day)).ok())
    }

    fn datetime_column(name: String, micros: impl Iterator<Item = Option<i64>>) -> Column {
        Int64Chunked::from_iter_options(name.into(), micros)
            .into_datetime(DATETIME_UNIT, Some(DATETIME_ZONE))
            .into_column()
    }

    fn date_column(name: String, days: impl Iterator<Item = Option<i32>>) -> Column {
        Int32Chunked::from_iter_options(name.into(), days)
            .into_date()
            .into_column()
    }

    /// A `Datetime(us, UTC)` column from epoch integers; overflow and, with `nonzero`, `0` become null.
    pub fn epoch_datetime(
        name: String,
        unit: EpochUnit,
        nonzero: bool,
        values: impl Iterator<Item = Option<i64>>,
    ) -> Column {
        datetime_column(name, values.map(|value| epoch_micros(value, unit, nonzero)))
    }

    /// A `Date` column from epoch integers, taking the UTC calendar day.
    pub fn epoch_date(
        name: String,
        unit: EpochUnit,
        nonzero: bool,
        values: impl Iterator<Item = Option<i64>>,
    ) -> Column {
        date_column(name, values.map(|value| epoch_day(value, unit, nonzero)))
    }

    fn parse_date(value: &str) -> Option<i32> {
        let day = NaiveDate::parse_from_str(value.get(..10)?, "%Y-%m-%d").ok()?;
        Some(chrono::Datelike::num_days_from_ce(&day) - UNIX_EPOCH_FROM_CE_DAYS)
    }

    fn parse_rfc3339(value: &str) -> Option<i64> {
        DateTime::parse_from_rfc3339(value)
            .or_else(|_| DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f%#z"))
            .ok()
            .map(|datetime| datetime.timestamp_micros())
    }

    /// A `Date` column from the leading `YYYY-MM-DD` of each string; unparseable values become null.
    pub fn str_date<'a>(name: String, values: impl Iterator<Item = Option<&'a str>>) -> Column {
        date_column(name, values.map(|value| value.and_then(parse_date)))
    }

    /// A `Datetime(us, UTC)` column from RFC 3339 strings; unparseable values become null.
    pub fn rfc3339_datetime<'a>(
        name: String,
        values: impl Iterator<Item = Option<&'a str>>,
    ) -> Column {
        datetime_column(name, values.map(|value| value.and_then(parse_rfc3339)))
    }

    /// A `List(Datetime(us, UTC))` column from lists of epoch integers.
    pub fn datetime_list(
        name: String,
        unit: EpochUnit,
        nonzero: bool,
        values: impl Iterator<Item = Option<Vec<Option<i64>>>>,
    ) -> Column {
        let mut builder =
            ListPrimitiveChunkedBuilder::<Int64Type>::new(name.into(), 0, 0, datetime_dtype());
        for list in values {
            match list {
                Some(list) => builder.append_iter(
                    list.into_iter()
                        .map(|value| epoch_micros(value, unit, nonzero)),
                ),
                None => builder.append_null(),
            }
        }
        Column::from(builder.finish().into_series())
    }

    /// A `List(Date)` column from lists of epoch integers.
    pub fn date_list(
        name: String,
        unit: EpochUnit,
        nonzero: bool,
        values: impl Iterator<Item = Option<Vec<Option<i64>>>>,
    ) -> Column {
        let mut builder =
            ListPrimitiveChunkedBuilder::<Int32Type>::new(name.into(), 0, 0, DataType::Date);
        for list in values {
            match list {
                Some(list) => builder.append_iter(
                    list.into_iter()
                        .map(|value| epoch_day(value, unit, nonzero)),
                ),
                None => builder.append_null(),
            }
        }
        Column::from(builder.finish().into_series())
    }

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

    fn struct_series(frame: DataFrame) -> Series {
        frame.into_struct(PlSmallStr::EMPTY).into_series()
    }

    /// One `List(Struct)` column holding each parent's child rows as a list of structs.
    ///
    /// An empty slice is an empty list. Child rows use their own nested frame,
    /// so nesting recurses.
    pub fn nested_column<'a, T: ToDataFrame + 'a>(
        name: &str,
        rows: impl ExactSizeIterator<Item = &'a [T]>,
    ) -> PolarsResult<Column> {
        let dtype = struct_series(T::vec_to_nested_dataframe(&[])?)
            .dtype()
            .clone();
        let mut builder = get_list_builder(&dtype, rows.len(), rows.len(), name.into());
        for items in rows {
            let child = T::vec_to_nested_dataframe(items)?;
            builder.append_series(&struct_series(child))?;
        }
        Ok(Column::from(builder.finish().into_series()))
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

    #[non_exhaustive]
    pub struct DisplayColumn;

    impl DisplayColumn {
        pub fn column<'a, T: Display + 'a>(
            self,
            name: String,
            values: impl Iterator<Item = Option<&'a T>>,
        ) -> Option<Column> {
            let values: Vec<Option<String>> = values.map(|v| v.map(T::to_string)).collect();
            Some(Column::from(Series::new(name.into(), values)))
        }
    }

    #[non_exhaustive]
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

#[cfg(all(test, feature = "dataframe"))]
mod tests {
    use super::derive::{
        EpochUnit, date_list, datetime_list, epoch_date, epoch_datetime, rfc3339_datetime, str_date,
    };
    use polars::prelude::{DataType, TimeUnit, TimeZone};

    fn utc() -> DataType {
        DataType::Datetime(TimeUnit::Microseconds, Some(TimeZone::UTC))
    }

    #[test]
    fn epoch_units_scale_to_microseconds() {
        let column = epoch_datetime(
            "t".into(),
            EpochUnit::Nanos,
            false,
            [Some(1_500), Some(-1), None].into_iter(),
        );
        assert_eq!(column.dtype(), &utc());
        let values: Vec<_> = column.datetime().unwrap().phys.iter().collect();
        assert_eq!(values, vec![Some(1), Some(-1), None]);
    }

    #[test]
    fn epoch_dates_floor_toward_the_earlier_day() {
        let column = epoch_date(
            "d".into(),
            EpochUnit::Millis,
            false,
            [Some(86_399_999), Some(-1), Some(i64::MAX)].into_iter(),
        );
        let values: Vec<_> = column.date().unwrap().phys.iter().collect();
        assert_eq!(values, vec![Some(0), Some(-1), None]);
    }

    #[test]
    fn string_parsers_return_null_on_failure() {
        let dates = str_date(
            "d".into(),
            [Some("2024-02-29"), Some("2023-02-29"), None].into_iter(),
        );
        let values: Vec<_> = dates.date().unwrap().phys.iter().collect();
        assert_eq!(values, vec![Some(19_782), None, None]);

        let instants = rfc3339_datetime(
            "t".into(),
            [Some("1970-01-01T00:00:01Z"), Some("1970-01-01"), None].into_iter(),
        );
        let values: Vec<_> = instants.datetime().unwrap().phys.iter().collect();
        assert_eq!(values, vec![Some(1_000_000), None, None]);
    }

    #[test]
    fn list_helpers_keep_null_rows_and_dtypes() {
        let rows = || [Some(vec![Some(0), Some(1)]), None].into_iter();
        let datetimes = datetime_list("t".into(), EpochUnit::Seconds, true, rows());
        assert_eq!(datetimes.dtype(), &DataType::List(Box::new(utc())));
        assert_eq!(datetimes.null_count(), 1);
        let dates = date_list("d".into(), EpochUnit::Seconds, false, rows());
        assert_eq!(dates.dtype(), &DataType::List(Box::new(DataType::Date)));
        assert_eq!(dates.null_count(), 1);
    }
}

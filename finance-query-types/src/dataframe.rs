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
        ChunkedArray, Column, DataFrame, DataType, Int32Type, Int64Chunked, Int64Type, IntoColumn,
        IntoSeries, ListBuilderTrait, ListStringChunkedBuilder, NamedFrom, NewChunkedArray,
        PlSmallStr, PolarsNumericType, PolarsResult, Series, TimeUnit, TimeZone,
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

    /// The dtype of every `Datetime` column: microseconds, UTC.
    pub fn datetime_dtype() -> DataType {
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

    /// Pushes into a plain `Vec` until the first null, which is the common case and avoids
    /// the per-value builder; only a column with a null pays for the validity mask.
    fn numeric<T: PolarsNumericType>(
        name: String,
        mut values: impl Iterator<Item = Option<T::Native>>,
    ) -> ChunkedArray<T> {
        let mut dense = Vec::with_capacity(values.size_hint().0);
        while let Some(value) = values.next() {
            match value {
                Some(value) => dense.push(value),
                None => {
                    let rest = dense
                        .into_iter()
                        .map(Some)
                        .chain(std::iter::once(None))
                        .chain(values);
                    return ChunkedArray::from_iter_options(name.into(), rest);
                }
            }
        }
        ChunkedArray::from_vec(name.into(), dense)
    }

    fn datetime_column(micros: Int64Chunked) -> Column {
        micros
            .into_datetime(DATETIME_UNIT, Some(DATETIME_ZONE))
            .into_column()
    }

    fn date_column(name: String, days: impl Iterator<Item = Option<i32>>) -> Column {
        numeric::<Int32Type>(name, days).into_date().into_column()
    }

    /// A `Datetime(us, UTC)` column from epoch integers; overflow and, with `nonzero`, `0` become null.
    pub fn epoch_datetime(
        name: String,
        unit: EpochUnit,
        nonzero: bool,
        values: impl Iterator<Item = Option<i64>>,
    ) -> Column {
        // Epoch columns back long candle and stream frames, where building from a slice is far
        // cheaper per row than the dense push loop, at one extra `Vec` per column.
        let micros: Vec<Option<i64>> = values
            .map(|value| epoch_micros(value, unit, nonzero))
            .collect();
        datetime_column(Int64Chunked::new(name.into(), &micros))
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
        let bytes = value.as_bytes().get(..10)?;
        if bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let number = |digits: &[u8]| {
            digits.iter().try_fold(0u32, |acc, &b| {
                b.is_ascii_digit().then(|| acc * 10 + u32::from(b - b'0'))
            })
        };
        let year = i32::try_from(number(&bytes[..4])?).ok()?;
        let day = NaiveDate::from_ymd_opt(year, number(&bytes[5..7])?, number(&bytes[8..10])?)?;
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
        datetime_column(numeric::<Int64Type>(
            name,
            values.map(|value| value.and_then(parse_rfc3339)),
        ))
    }

    /// A `List(Datetime(us, UTC))` column from lists of epoch integers.
    pub fn datetime_list<'a, S: 'a>(
        name: String,
        unit: EpochUnit,
        nonzero: bool,
        values: impl Iterator<Item = Option<&'a [S]>>,
        each: impl Fn(&S) -> Option<i64>,
    ) -> Column {
        let mut builder =
            ListPrimitiveChunkedBuilder::<Int64Type>::new(name.into(), 0, 0, datetime_dtype());
        for list in values {
            match list {
                Some(list) => builder.append_iter(
                    list.iter()
                        .map(|value| epoch_micros(each(value), unit, nonzero)),
                ),
                None => builder.append_null(),
            }
        }
        Column::from(builder.finish().into_series())
    }

    /// A `List(Date)` column from lists of epoch integers.
    pub fn date_list<'a, S: 'a>(
        name: String,
        unit: EpochUnit,
        nonzero: bool,
        values: impl Iterator<Item = Option<&'a [S]>>,
        each: impl Fn(&S) -> Option<i64>,
    ) -> Column {
        let mut builder =
            ListPrimitiveChunkedBuilder::<Int32Type>::new(name.into(), 0, 0, DataType::Date);
        for list in values {
            match list {
                Some(list) => builder.append_iter(
                    list.iter()
                        .map(|value| epoch_day(each(value), unit, nonzero)),
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

    const STRONG_TIME_TOKENS: &[&str] = &["date", "time", "timestamp", "expiration"];
    const WEAK_TIME_TOKENS: &[&str] = &[
        "start",
        "end",
        "ending",
        "filed",
        "published",
        "updated",
        "quarter",
        "year",
        "period",
    ];

    fn is_time_dtype(dtype: &DataType) -> bool {
        match dtype {
            DataType::List(inner) => is_time_dtype(inner),
            _ => *dtype == DataType::Date || *dtype == datetime_dtype(),
        }
    }

    /// Asserts that a column is time-typed exactly when its name says it is a point in time.
    ///
    /// A name ending in `date`, `time`, `timestamp`, `expiration` or `at` must be `Date` or
    /// `Datetime` (or a list of either). A `Date` or `Datetime` column must have one of those
    /// tokens anywhere, or one of `start`, `end`, `ending`, `filed`, `published`, `updated`,
    /// `quarter`, `year` or `period`. `exempt` names columns checked by neither rule.
    pub fn assert_time_naming(ty: &str, frame: &DataFrame, exempt: &[&str]) {
        let violations: Vec<String> = frame
            .schema()
            .iter()
            .filter(|(name, _)| !exempt.contains(&name.as_str()))
            .filter_map(|(name, dtype)| {
                let tokens: Vec<&str> = name.split('_').collect();
                let strong = tokens
                    .last()
                    .is_some_and(|t| STRONG_TIME_TOKENS.contains(t) || *t == "at");
                let weak = strong
                    || tokens
                        .iter()
                        .any(|t| STRONG_TIME_TOKENS.contains(t) || WEAK_TIME_TOKENS.contains(t));
                let time = is_time_dtype(dtype);
                (strong && !time)
                    .then(|| {
                        format!("{ty}.{name}: {dtype} is named like a time but is not typed as one")
                    })
                    .or_else(|| {
                        (time && !weak).then(|| {
                            format!(
                                "{ty}.{name}: {dtype} is a time column without a time-like name"
                            )
                        })
                    })
            })
            .collect();
        assert!(violations.is_empty(), "{}", violations.join("\n"));
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
        EpochUnit, assert_time_naming, date_list, datetime_dtype, datetime_list, epoch_date,
        epoch_datetime, rfc3339_datetime, str_date,
    };
    use polars::prelude::{DataFrame, DataType, NamedFrom, Series};

    #[test]
    fn epoch_units_scale_to_microseconds() {
        let column = epoch_datetime(
            "t".into(),
            EpochUnit::Nanos,
            false,
            [Some(1_500), Some(-1), None].into_iter(),
        );
        assert_eq!(column.dtype(), &datetime_dtype());
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
        let lists = [Some(vec![0_i64, 1]), None];
        let rows = || lists.iter().map(|list| list.as_deref());
        let datetimes = datetime_list("t".into(), EpochUnit::Seconds, true, rows(), |v| Some(*v));
        assert_eq!(
            datetimes.dtype(),
            &DataType::List(Box::new(datetime_dtype()))
        );
        assert_eq!(datetimes.null_count(), 1);
        let dates = date_list("d".into(), EpochUnit::Seconds, false, rows(), |v| Some(*v));
        assert_eq!(dates.dtype(), &DataType::List(Box::new(DataType::Date)));
        assert_eq!(dates.null_count(), 1);
    }

    fn frame(name: &str, dtype: &DataType) -> DataFrame {
        let column = Series::new(name.into(), Vec::<i64>::new())
            .cast(dtype)
            .unwrap();
        DataFrame::new(0, vec![column.into()]).unwrap()
    }

    #[test]
    fn time_naming_accepts_agreeing_names_and_dtypes() {
        for (name, dtype) in [
            ("filing_date", DataType::Date),
            ("observation_start", DataType::Date),
            ("updated", datetime_dtype()),
            ("earnings_date", DataType::List(Box::new(DataType::Date))),
            ("all_time_high", DataType::Float64),
            ("count", DataType::Int64),
        ] {
            assert_time_naming("T", &frame(name, &dtype), &[]);
        }
    }

    #[test]
    #[should_panic(expected = "named like a time")]
    fn time_naming_rejects_untyped_time_names() {
        assert_time_naming("T", &frame("ipo_date", &DataType::Int64), &[]);
    }

    #[test]
    #[should_panic(expected = "without a time-like name")]
    fn time_naming_rejects_time_columns_with_other_names() {
        assert_time_naming("T", &frame("day", &DataType::Date), &[]);
    }

    #[test]
    fn time_naming_skips_exempt_columns() {
        assert_time_naming("T", &frame("time", &DataType::Int64), &["time"]);
    }
}

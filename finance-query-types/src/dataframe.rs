use polars::prelude::{DataFrame, PolarsResult};

/// Conversion of a model into a polars [`DataFrame`].
///
/// Implemented by every type with `#[derive(ToDataFrame)]` and by the
/// collection types that build a frame from their items (for example
/// `Chart` and `Options`). Generic code can take `T: ToDataFrame` instead of
/// naming each model.
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

/// Support code for `#[derive(ToDataFrame)]`. Not public API.
#[doc(hidden)]
pub mod derive {
    use std::fmt::Display;
    use std::marker::PhantomData;

    use polars::prelude::{
        Column, IntoSeries, ListBuilderTrait, ListStringChunkedBuilder, NamedFrom, PlSmallStr,
        PolarsResult, Series,
    };

    use polars::chunked_array::builder::get_list_builder;

    use super::ToDataFrame;

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

    /// One `List(Struct)` column holding each parent's child rows as a list of structs.
    ///
    /// An empty slice is an empty list. Child rows use their own nested frame,
    /// so nesting recurses.
    pub fn nested_column<'a, T: ToDataFrame + 'a>(
        name: &str,
        rows: impl ExactSizeIterator<Item = &'a [T]>,
    ) -> PolarsResult<Column> {
        let dtype = T::vec_to_nested_dataframe(&[])?
            .into_struct(PlSmallStr::EMPTY)
            .into_series()
            .dtype()
            .clone();
        let mut builder = get_list_builder(&dtype, rows.len(), rows.len(), name.into());
        for items in rows {
            let child = T::vec_to_nested_dataframe(items)?;
            builder.append_series(&child.into_struct(PlSmallStr::EMPTY).into_series())?;
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

//! Polars conversions for the batch response types.

use super::{
    BatchCapitalGainsResponse, BatchChartsResponse, BatchDividendsResponse, BatchNewsResponse,
    BatchOptionsResponse, BatchQuotesResponse, BatchRecommendationsResponse, BatchSplitsResponse,
};
use crate::models::chart::{Candle, CapitalGain, Dividend, Split};
use crate::models::corporate::news::News;
use crate::models::corporate::recommendation::SimilarSymbol;
use crate::models::options::Contracts;
use crate::models::quote::Quote;
use polars::prelude::{DataFrame, NamedFrom, PolarsResult, Series};
use std::collections::HashMap;

fn stack_by_symbol<T>(
    items: &HashMap<String, T>,
    convert: impl Fn(&T) -> PolarsResult<DataFrame>,
    empty: impl FnOnce() -> PolarsResult<DataFrame>,
) -> PolarsResult<DataFrame> {
    let mut symbols: Vec<&String> = items.keys().collect();
    symbols.sort();

    let mut stacked = with_symbol(empty()?, "")?;
    for symbol in symbols {
        stacked.vstack_mut(&with_symbol(convert(&items[symbol])?, symbol)?)?;
    }
    stacked.rechunk_mut();
    Ok(stacked)
}

fn with_symbol(mut frame: DataFrame, symbol: &str) -> PolarsResult<DataFrame> {
    let column = Series::new("symbol".into(), vec![symbol; frame.height()]);
    frame.insert_column(0, column.into())?;
    Ok(frame)
}

fn rename_symbol(mut frame: DataFrame) -> PolarsResult<DataFrame> {
    frame.rename("symbol", "recommended_symbol".into())?;
    Ok(frame)
}

impl BatchQuotesResponse {
    /// Converts the quotes to a polars DataFrame, one row per symbol.
    ///
    /// Columns are `symbol` followed by the columns of `Quote::to_dataframe`.
    /// Rows are sorted by symbol. Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.quotes,
            |quote| quote.to_dataframe()?.drop("symbol"),
            || <Quote>::vec_to_dataframe(&[])?.drop("symbol"),
        )
    }
}

impl BatchChartsResponse {
    /// Converts the charts to a long-format polars DataFrame, one row per candle.
    ///
    /// Columns are `symbol` followed by the columns of `Chart::to_dataframe`.
    /// Rows are grouped by symbol in ascending order, candles in chart order.
    /// Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.charts,
            |chart| chart.to_dataframe(),
            || Candle::vec_to_dataframe(&[]),
        )
    }
}

impl BatchDividendsResponse {
    /// Converts the dividends to a long-format polars DataFrame, one row per dividend.
    ///
    /// Columns are `symbol` followed by the columns of `Dividend::vec_to_dataframe`.
    /// Rows are grouped by symbol in ascending order, dividends in stored order.
    /// Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.dividends,
            |items| Dividend::vec_to_dataframe(items),
            || Dividend::vec_to_dataframe(&[]),
        )
    }
}

impl BatchSplitsResponse {
    /// Converts the splits to a long-format polars DataFrame, one row per split.
    ///
    /// Columns are `symbol` followed by the columns of `Split::vec_to_dataframe`.
    /// Rows are grouped by symbol in ascending order, splits in stored order.
    /// Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.splits,
            |items| Split::vec_to_dataframe(items),
            || Split::vec_to_dataframe(&[]),
        )
    }
}

impl BatchCapitalGainsResponse {
    /// Converts the capital gains to a long-format polars DataFrame, one row per gain.
    ///
    /// Columns are `symbol` followed by the columns of `CapitalGain::vec_to_dataframe`.
    /// Rows are grouped by symbol in ascending order, gains in stored order.
    /// Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.capital_gains,
            |items| CapitalGain::vec_to_dataframe(items),
            || CapitalGain::vec_to_dataframe(&[]),
        )
    }
}

impl BatchNewsResponse {
    /// Converts the news to a long-format polars DataFrame, one row per article.
    ///
    /// Columns are `symbol` followed by the columns of `News::vec_to_dataframe`.
    /// Rows are grouped by symbol in ascending order, articles in stored order.
    /// Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.news,
            |items| News::vec_to_dataframe(items),
            || News::vec_to_dataframe(&[]),
        )
    }
}

impl BatchRecommendationsResponse {
    /// Converts the recommendations to a long-format polars DataFrame, one row per similar symbol.
    ///
    /// Columns are `symbol` (the queried symbol), `recommended_symbol` and `score`.
    /// Rows are grouped by queried symbol in ascending order, recommendations in
    /// stored order. Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.recommendations,
            |recommendation| rename_symbol(recommendation.to_dataframe()?),
            || rename_symbol(SimilarSymbol::vec_to_dataframe(&[])?),
        )
    }
}

impl BatchOptionsResponse {
    /// Converts the option chains to a long-format polars DataFrame, one row per contract.
    ///
    /// Columns are `symbol` followed by the columns of `Options::to_dataframe`,
    /// including `option_type`. Rows are grouped by symbol in ascending order.
    /// Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.options,
            |options| options.to_dataframe(),
            || {
                let mut frame = Contracts(Vec::new()).to_dataframe()?;
                let option_type = Series::new("option_type".into(), Vec::<&str>::new());
                frame.with_column(option_type.into())?;
                Ok(frame)
            },
        )
    }
}

#[cfg(feature = "indicators")]
impl super::BatchIndicatorsResponse {
    /// Converts the indicator summaries to a polars DataFrame, one row per symbol.
    ///
    /// Columns are `symbol` followed by the columns of `IndicatorsSummary::to_dataframe`.
    /// Rows are sorted by symbol. Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.indicators,
            |summary| summary.to_dataframe(),
            || crate::indicators::IndicatorsSummary::vec_to_dataframe(&[]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::DataType;

    fn response<T: serde::de::DeserializeOwned>(json: serde_json::Value) -> T {
        serde_json::from_value(json).unwrap()
    }

    fn names(df: &DataFrame) -> Vec<String> {
        df.get_column_names()
            .iter()
            .map(|n| n.to_string())
            .collect()
    }

    #[test]
    fn dividends_sorted_by_symbol_with_symbol_first() {
        let batch: BatchDividendsResponse = response(serde_json::json!({
            "dividends": {
                "MSFT": [{"timestamp": 5, "amount": 0.5}],
                "AAPL": [{"timestamp": 1, "amount": 0.1}, {"timestamp": 2, "amount": 0.2}]
            },
            "errors": {"BAD": "boom"}
        }));
        let df = batch.to_dataframe().unwrap();

        assert_eq!(names(&df)[..3], ["symbol", "timestamp", "amount"]);
        assert_eq!(df.column("symbol").unwrap().dtype(), &DataType::String);
        assert_eq!(df.column("timestamp").unwrap().dtype(), &DataType::Int64);
        assert_eq!(df.column("amount").unwrap().dtype(), &DataType::Float64);
        let symbols: Vec<_> = df
            .column("symbol")
            .unwrap()
            .str()
            .unwrap()
            .into_no_null_iter()
            .collect();
        assert_eq!(symbols, ["AAPL", "AAPL", "MSFT"]);
        let timestamps: Vec<_> = df
            .column("timestamp")
            .unwrap()
            .i64()
            .unwrap()
            .into_no_null_iter()
            .collect();
        assert_eq!(timestamps, [1, 2, 5]);
    }

    #[test]
    fn splits_keep_value_columns() {
        let batch: BatchSplitsResponse = response(serde_json::json!({
            "splits": {
                "NVDA": [{"timestamp": 9, "numerator": 10.0, "denominator": 1.0, "ratio": "10:1"}]
            },
            "errors": {}
        }));
        let df = batch.to_dataframe().unwrap();

        assert_eq!(
            names(&df)[..5],
            ["symbol", "timestamp", "numerator", "denominator", "ratio"]
        );
        assert_eq!(df.column("ratio").unwrap().dtype(), &DataType::String);
        assert_eq!(df.height(), 1);
    }

    #[test]
    fn recommendations_rename_inner_symbol() {
        let batch: BatchRecommendationsResponse = response(serde_json::json!({
            "recommendations": {
                "B": {"symbol": "B", "recommendations": [{"symbol": "X", "score": 0.5}]},
                "A": {"symbol": "A", "recommendations": [
                    {"symbol": "Y", "score": 0.9}, {"symbol": "Z", "score": 0.1}
                ]}
            },
            "errors": {}
        }));
        let df = batch.to_dataframe().unwrap();

        assert_eq!(names(&df), ["symbol", "recommended_symbol", "score"]);
        assert_eq!(df.column("score").unwrap().dtype(), &DataType::Float64);
        let recommended: Vec<_> = df
            .column("recommended_symbol")
            .unwrap()
            .str()
            .unwrap()
            .into_no_null_iter()
            .collect();
        assert_eq!(recommended, ["Y", "Z", "X"]);
    }

    #[test]
    fn all_errors_keep_typed_empty_schema() {
        let batch: BatchDividendsResponse = response(serde_json::json!({
            "dividends": {},
            "errors": {"AAPL": "boom"}
        }));
        let df = batch.to_dataframe().unwrap();

        assert_eq!(names(&df)[..3], ["symbol", "timestamp", "amount"]);
        assert_eq!(df.column("timestamp").unwrap().dtype(), &DataType::Int64);
        assert_eq!(df.height(), 0);

        let charts: BatchChartsResponse = response(serde_json::json!({
            "charts": {},
            "errors": {"AAPL": "boom"}
        }));
        let df = charts.to_dataframe().unwrap();

        assert_eq!(names(&df)[..2], ["symbol", "timestamp"]);
        assert_eq!(df.column("close").unwrap().dtype(), &DataType::Float64);
        assert_eq!(df.height(), 0);
    }
}

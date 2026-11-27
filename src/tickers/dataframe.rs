//! Polars conversions for the batch response types.

use super::{
    BatchCapitalGainsResponse, BatchChartsResponse, BatchDividendsResponse,
    BatchFinancialsResponse, BatchNewsResponse, BatchOptionsResponse, BatchQuotesAs,
    BatchQuotesResponse, BatchRecommendationsResponse, BatchSparksResponse, BatchSplitsResponse,
    SymbolMap,
};
use crate::__dataframe::Columns;
use crate::models::chart::spark::Spark;
use crate::models::chart::{Candle, CapitalGain, Chart, Dividend, Split};
use crate::models::corporate::news::News;
use crate::models::corporate::recommendation::SimilarSymbol;
use crate::models::fundamentals::FinancialStatement;
use crate::models::options::{Contracts, Options};
use polars::prelude::{DataFrame, NamedFrom, PolarsResult, Series};
use std::collections::BTreeMap;

fn stack_by_symbol<T>(
    items: &SymbolMap<T>,
    convert: impl Fn(&T) -> PolarsResult<DataFrame>,
    empty: impl FnOnce() -> PolarsResult<DataFrame>,
) -> PolarsResult<DataFrame> {
    let mut frames = items
        .iter()
        .map(|(symbol, item)| with_symbol(convert(item)?, symbol));
    let Some(first) = frames.next() else {
        return with_symbol(empty()?, "");
    };
    let mut stacked = first?;
    for frame in frames {
        stacked.vstack_mut(&frame?)?;
    }
    stacked.rechunk_mut();
    Ok(stacked)
}

/// One row per symbol for single-row item types, built in one pass with no per-item frames to stack.
fn rows_by_symbol<T: Columns>(items: &SymbolMap<T>) -> PolarsResult<DataFrame> {
    let rows: Vec<Option<&T>> = items.values().map(Some).collect();
    let mut frame = DataFrame::new_infer_height(T::columns(&rows, ""))?.drop("symbol")?;
    let symbols: Vec<&str> = items.keys().map(String::as_str).collect();
    frame.insert_column(0, Series::new("symbol".into(), symbols).into())?;
    Ok(frame)
}

fn with_symbol(frame: DataFrame, symbol: &str) -> PolarsResult<DataFrame> {
    crate::__dataframe::with_leading_column(frame, "symbol", symbol)
}

fn rename_symbol(mut frame: DataFrame) -> PolarsResult<DataFrame> {
    frame.rename("symbol", "recommended_symbol".into())?;
    Ok(frame)
}

impl BatchQuotesResponse {
    /// Converts the quotes to a polars DataFrame, one row per symbol.
    ///
    /// Columns are `symbol` followed by the columns of `Quote::to_dataframe`.
    /// Rows follow input symbol order. Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        rows_by_symbol(&self.quotes)
    }
}

impl BatchQuotesAs<crate::format::Both> {
    /// Converts the quotes to a polars DataFrame, one row per symbol.
    ///
    /// Columns are `symbol` followed by the columns of `Quote::to_dataframe`.
    /// Rows are sorted by symbol. Failed symbols in `errors` are omitted.
    /// Only the `Both` format has a frame; `Raw` and `Pretty` quotes do not.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        rows_by_symbol(&self.quotes)
    }
}

impl BatchChartsResponse {
    /// Converts the charts to a long-format polars DataFrame, one row per candle.
    ///
    /// Columns are `symbol` followed by the columns of `Chart::to_dataframe`.
    /// Rows are grouped by symbol in input order, candles in chart order.
    /// Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(&self.charts, Chart::to_dataframe, || {
            Candle::vec_to_dataframe(&[])
        })
    }
}

impl BatchSparksResponse {
    /// Converts the sparks to a long-format polars DataFrame, one row per point.
    ///
    /// Columns are `symbol` followed by the columns of `Spark::to_dataframe`.
    /// Rows are grouped by symbol in input order, points in stored order.
    /// Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.sparks,
            |spark| spark.to_dataframe()?.drop("symbol"),
            || Spark::vec_to_dataframe(&[])?.drop("symbol"),
        )
    }
}

impl BatchDividendsResponse {
    /// Converts the dividends to a long-format polars DataFrame, one row per dividend.
    ///
    /// Columns are `symbol` followed by the columns of `Dividend::vec_to_dataframe`.
    /// Rows are grouped by symbol in input order, dividends in stored order.
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
    /// Rows are grouped by symbol in input order, splits in stored order.
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
    /// Rows are grouped by symbol in input order, gains in stored order.
    /// Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.capital_gains,
            |items| CapitalGain::vec_to_dataframe(items),
            || CapitalGain::vec_to_dataframe(&[]),
        )
    }
}

impl BatchFinancialsResponse {
    /// Converts the statements to a long-format polars DataFrame, one row per symbol, metric and date.
    ///
    /// Columns are `symbol` followed by the columns of `FinancialStatement::to_dataframe`.
    /// Rows are grouped by symbol in input order, then sorted by metric and date.
    /// Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.financials,
            |statement| statement.to_dataframe()?.drop("symbol"),
            || {
                FinancialStatement {
                    symbol: String::new(),
                    statement_type: String::new(),
                    frequency: String::new(),
                    statement: BTreeMap::new(),
                    provider_id: None,
                }
                .to_dataframe()?
                .drop("symbol")
            },
        )
    }
}

impl BatchNewsResponse {
    /// Converts the news to a long-format polars DataFrame, one row per article.
    ///
    /// Columns are `symbol` followed by the columns of `News::vec_to_dataframe`.
    /// Rows are grouped by symbol in input order, articles in stored order.
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
    /// Rows are grouped by queried symbol in input order, recommendations in
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
    /// including `option_type`. Rows are grouped by symbol in input order.
    /// Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(&self.options, Options::to_dataframe, || {
            let mut frame = Contracts(Vec::new()).to_dataframe()?;
            let option_type = Series::new("option_type".into(), Vec::<&str>::new());
            frame.with_column(option_type.into())?;
            Ok(frame)
        })
    }
}

#[cfg(feature = "indicators")]
impl super::BatchIndicatorsResponse {
    /// Converts the indicator summaries to a polars DataFrame, one row per symbol.
    ///
    /// Columns are `symbol` followed by the columns of `IndicatorsSummary::to_dataframe`.
    /// Rows follow input symbol order. Failed symbols in `errors` are omitted.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        stack_by_symbol(
            &self.indicators,
            crate::indicators::IndicatorsSummary::to_dataframe,
            || crate::indicators::IndicatorsSummary::vec_to_dataframe(&[]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::quote::Quote;
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
    fn quotes_as_both_frame_matches_batch_quotes_frame() {
        let summary = crate::models::quote::QuoteSummaryResponse {
            symbol: "AAPL".into(),
            ..Default::default()
        };
        let mut batch = BatchQuotesResponse::with_capacity(1);
        batch
            .quotes
            .insert("AAPL".into(), Quote::from_response(&summary, None, None));
        let expected = batch.to_dataframe().unwrap();
        let generic: BatchQuotesAs<crate::format::Both> = batch.into();
        assert_eq!(generic.to_dataframe().unwrap(), expected);
    }

    #[test]
    fn dividends_follow_input_order_with_symbol_first() {
        let mut batch = BatchDividendsResponse::with_capacity(2);
        batch.dividends.insert(
            "MSFT".into(),
            response(serde_json::json!([{"timestamp": 5, "amount": 0.5}])),
        );
        batch.dividends.insert(
            "AAPL".into(),
            response(serde_json::json!([
                {"timestamp": 1, "amount": 0.1},
                {"timestamp": 2, "amount": 0.2}
            ])),
        );
        batch.errors.insert("BAD".into(), "boom".into());
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
        assert_eq!(symbols, ["MSFT", "AAPL", "AAPL"]);
        let timestamps: Vec<_> = df
            .column("timestamp")
            .unwrap()
            .i64()
            .unwrap()
            .into_no_null_iter()
            .collect();
        assert_eq!(timestamps, [5, 1, 2]);
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
    fn financials_stack_statements_by_symbol() {
        let statement = |symbol: &str, revenue: f64| {
            serde_json::json!({
                "symbol": symbol,
                "statementType": "income",
                "frequency": "annual",
                "statement": {
                    "TotalRevenue": {"2024-09-30": revenue, "2023-09-30": revenue / 2.0},
                    "NetIncome": {"2024-09-30": revenue / 4.0}
                }
            })
        };
        let batch: BatchFinancialsResponse = response(serde_json::json!({
            "financials": {"MSFT": statement("MSFT", 8.0), "AAPL": statement("AAPL", 4.0)},
            "errors": {"BAD": "boom"}
        }));
        let df = batch.to_dataframe().unwrap();

        assert_eq!(
            names(&df),
            [
                "symbol",
                "statement_type",
                "frequency",
                "metric",
                "date",
                "value"
            ]
        );
        assert_eq!(df.column("value").unwrap().dtype(), &DataType::Float64);
        let column = |name: &str| -> Vec<String> {
            df.column(name)
                .unwrap()
                .str()
                .unwrap()
                .into_no_null_iter()
                .map(str::to_string)
                .collect()
        };
        assert_eq!(
            column("symbol"),
            ["AAPL", "AAPL", "AAPL", "MSFT", "MSFT", "MSFT"]
        );
        assert_eq!(
            column("metric")[..3],
            ["net_income", "total_revenue", "total_revenue"]
        );
        assert_eq!(df.column("date").unwrap().dtype(), &DataType::Date);
    }

    #[test]
    fn financials_with_only_errors_keep_the_schema() {
        let batch: BatchFinancialsResponse = response(serde_json::json!({
            "financials": {},
            "errors": {"AAPL": "boom"}
        }));
        let df = batch.to_dataframe().unwrap();

        assert_eq!(
            names(&df),
            [
                "symbol",
                "statement_type",
                "frequency",
                "metric",
                "date",
                "value"
            ]
        );
        assert_eq!(df.column("value").unwrap().dtype(), &DataType::Float64);
        assert_eq!(df.height(), 0);
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

    #[test]
    fn options_stack_onto_the_empty_schema() {
        let contract =
            |symbol: &str| serde_json::json!({"contractSymbol": symbol, "strike": 100.0});
        let batch: BatchOptionsResponse = response(serde_json::json!({
            "options": {
                "AAPL": {"optionChain": {"result": [{"options": [{
                    "expirationDate": 1,
                    "calls": [contract("AAPL1C")],
                    "puts": [contract("AAPL1P")]
                }]}]}}
            },
            "errors": {}
        }));
        let df = batch.to_dataframe().unwrap();

        let empty: BatchOptionsResponse =
            response(serde_json::json!({"options": {}, "errors": {}}));
        assert_eq!(names(&df), names(&empty.to_dataframe().unwrap()));
        assert_eq!(names(&df)[..2], ["symbol", "contract_symbol"]);
        assert_eq!(names(&df).last().unwrap(), "option_type");
        assert_eq!(df.height(), 2);
    }

    #[test]
    fn sparks_stack_by_symbol_with_one_symbol_column() {
        let spark = |symbol: &str, ts: i64| {
            serde_json::json!({
                "symbol": symbol, "meta": {"symbol": "AAPL"}, "timestamps": [ts], "closes": [1.5],
                "interval": "5m", "range": "1d"
            })
        };
        let batch: BatchSparksResponse = response(serde_json::json!({
            "sparks": {"B": spark("B", 2), "A": spark("A", 1)},
            "errors": {}
        }));
        let df = batch.to_dataframe().unwrap();
        assert_eq!(
            names(&df),
            ["symbol", "timestamp", "close", "interval", "range"]
        );
        assert_eq!(df.column("timestamp").unwrap().dtype(), &DataType::Int64);
        assert_eq!(df.height(), 2);

        let empty: BatchSparksResponse = response(serde_json::json!({"sparks": {}, "errors": {}}));
        let empty = empty.to_dataframe().unwrap();
        assert_eq!(names(&empty), names(&df));
        assert_eq!(empty.height(), 0);
    }

    #[test]
    fn capital_gains_follow_input_order_with_symbol_first() {
        let mut batch = BatchCapitalGainsResponse::with_capacity(2);
        batch.capital_gains.insert(
            "VTSAX".into(),
            response(serde_json::json!([{"timestamp": 30, "amount": 0.75}])),
        );
        batch.capital_gains.insert(
            "FXAIX".into(),
            response(serde_json::json!([
                {"timestamp": 10, "amount": 0.1},
                {"timestamp": 20, "amount": 0.2}
            ])),
        );
        batch.errors.insert("BAD".into(), "boom".into());
        let df = batch.to_dataframe().unwrap();

        assert_eq!(names(&df)[..3], ["symbol", "timestamp", "amount"]);
        assert_eq!(df.column("timestamp").unwrap().dtype(), &DataType::Int64);
        assert_eq!(df.column("amount").unwrap().dtype(), &DataType::Float64);
        assert_eq!(df.height(), 3);
        let symbols: Vec<_> = df
            .column("symbol")
            .unwrap()
            .str()
            .unwrap()
            .into_no_null_iter()
            .collect();
        assert_eq!(symbols, ["VTSAX", "FXAIX", "FXAIX"]);
        let amounts: Vec<_> = df
            .column("amount")
            .unwrap()
            .f64()
            .unwrap()
            .into_no_null_iter()
            .collect();
        assert_eq!(amounts, [0.75, 0.1, 0.2]);

        let empty: BatchCapitalGainsResponse =
            response(serde_json::json!({"capitalGains": {}, "errors": {"X": "boom"}}));
        let empty = empty.to_dataframe().unwrap();
        assert_eq!(names(&empty), names(&df));
        assert_eq!(empty.height(), 0);
    }

    #[test]
    fn news_follow_input_order_with_symbol_first() {
        let article = |title: &str| {
            serde_json::json!({
                "title": title, "link": format!("https://example.com/{title}"),
                "source": "Reuters", "img": "", "time": "1 hour ago"
            })
        };
        let mut batch = BatchNewsResponse::with_capacity(2);
        batch
            .news
            .insert("TSLA".into(), response(serde_json::json!([article("t1")])));
        batch.news.insert(
            "AAPL".into(),
            response(serde_json::json!([article("a1"), article("a2")])),
        );
        let df = batch.to_dataframe().unwrap();

        assert_eq!(
            names(&df)[..6],
            ["symbol", "title", "link", "source", "img", "time"]
        );
        assert_eq!(df.height(), 3);
        let column = |name: &str| -> Vec<String> {
            df.column(name)
                .unwrap()
                .str()
                .unwrap()
                .into_no_null_iter()
                .map(str::to_string)
                .collect()
        };
        assert_eq!(column("symbol"), ["TSLA", "AAPL", "AAPL"]);
        assert_eq!(column("title"), ["t1", "a1", "a2"]);

        let empty: BatchNewsResponse = response(serde_json::json!({"news": {}, "errors": {}}));
        let empty = empty.to_dataframe().unwrap();
        assert_eq!(names(&empty), names(&df));
        assert_eq!(empty.height(), 0);
    }

    #[cfg(feature = "indicators")]
    #[test]
    fn indicators_one_row_per_symbol_in_input_order() {
        use crate::indicators::IndicatorsSummary;
        use crate::tickers::BatchIndicatorsResponse;

        let mut batch = BatchIndicatorsResponse::with_capacity(2);
        batch.indicators.insert(
            "MSFT".into(),
            IndicatorsSummary {
                sma_10: Some(2.0),
                ..Default::default()
            },
        );
        batch.indicators.insert(
            "AAPL".into(),
            IndicatorsSummary {
                sma_10: Some(1.0),
                ..Default::default()
            },
        );
        let df = batch.to_dataframe().unwrap();

        assert_eq!(names(&df)[..2], ["symbol", "sma_10"]);
        assert_eq!(df.column("sma_10").unwrap().dtype(), &DataType::Float64);
        assert_eq!(df.height(), 2);
        let symbols: Vec<_> = df
            .column("symbol")
            .unwrap()
            .str()
            .unwrap()
            .into_no_null_iter()
            .collect();
        assert_eq!(symbols, ["MSFT", "AAPL"]);
        let sma: Vec<_> = df.column("sma_10").unwrap().f64().unwrap().iter().collect();
        assert_eq!(sma, [Some(2.0), Some(1.0)]);
        assert_eq!(df.column("ema_10").unwrap().null_count(), 2);

        let empty: BatchIndicatorsResponse =
            response(serde_json::json!({"indicators": {}, "errors": {}}));
        let empty = empty.to_dataframe().unwrap();
        assert_eq!(names(&empty), names(&df));
        assert_eq!(empty.height(), 0);
    }

    #[test]
    fn quote_rows_match_stacked_quote_frames() {
        let quote = |symbol: &str, time: i64| -> Quote {
            response(serde_json::json!({
                "symbol": symbol,
                "regularMarketTime": time,
                "exDividendDate": {"raw": 1794528000, "fmt": "2026-11-09"}
            }))
        };
        let mut batch = BatchQuotesResponse::with_capacity(3);
        batch
            .quotes
            .insert("MSFT".into(), quote("MSFT", 1794603601));
        batch
            .quotes
            .insert("AAPL".into(), quote("AAPL", 1794603602));
        batch.quotes.insert(
            "BRK-B".into(),
            response(serde_json::json!({"symbol": "BRK-B"})),
        );

        let stacked = stack_by_symbol(
            &batch.quotes,
            |quote| quote.to_dataframe()?.drop("symbol"),
            || <Quote>::vec_to_dataframe(&[])?.drop("symbol"),
        )
        .unwrap();
        assert!(batch.to_dataframe().unwrap().equals_missing(&stacked));

        let empty = BatchQuotesResponse::with_capacity(0)
            .to_dataframe()
            .unwrap();
        assert_eq!(empty.height(), 0);
        assert_eq!(empty.get_column_names(), stacked.get_column_names());
    }
}

//! Polars conversions for backtest results.

use super::{BacktestResult, EquityPoint};
use finance_query_types::__dataframe::{EpochUnit, epoch_datetime};
use polars::prelude::{Column, DataFrame, NamedFrom, PolarsResult, Series};

fn series<T, U>(name: &str, items: &[T], get: impl Fn(&T) -> U) -> Column
where
    Series: NamedFrom<Vec<U>, [U]>,
{
    Series::new(name.into(), items.iter().map(get).collect::<Vec<_>>()).into()
}

pub(crate) fn datetimes<T>(name: &str, items: &[T], get: impl Fn(&T) -> i64) -> Column {
    epoch_datetime(
        name.to_string(),
        EpochUnit::Seconds,
        false,
        items.iter().map(|item| Some(get(item))),
    )
}

impl BacktestResult {
    /// Converts the equity curve to a polars DataFrame, one row per bar.
    ///
    /// Columns are `timestamp` (datetime, UTC), `equity` (f64) and `drawdown` (f64, a fraction).
    pub fn equity_dataframe(&self) -> PolarsResult<DataFrame> {
        EquityPoint::vec_to_dataframe(&self.equity_curve)
    }

    /// Converts the trade log to a polars DataFrame, one row per trade, in log order.
    ///
    /// Columns are `side` (string, `LONG` or `SHORT`), `entry_timestamp`, `exit_timestamp`
    /// (datetime, UTC), `entry_price`, `exit_price`, `quantity`, `entry_quantity`, `commission`,
    /// `transaction_tax`, `pnl`, `net_return`, `dividend_income`, `unreinvested_dividends`,
    /// `financing_cost` (f64), `is_partial` (bool) and `scale_sequence` (u64). Entry and exit
    /// signals and tags are not included.
    pub fn trades_dataframe(&self) -> PolarsResult<DataFrame> {
        let t = &self.trades;
        DataFrame::new(
            t.len(),
            vec![
                series("side", t, |x| x.side.as_str()),
                datetimes("entry_timestamp", t, |x| x.entry_timestamp),
                datetimes("exit_timestamp", t, |x| x.exit_timestamp),
                series("entry_price", t, |x| x.entry_price),
                series("exit_price", t, |x| x.exit_price),
                series("quantity", t, |x| x.quantity),
                series("entry_quantity", t, |x| x.entry_quantity),
                series("commission", t, |x| x.commission),
                series("transaction_tax", t, |x| x.transaction_tax),
                series("pnl", t, |x| x.pnl),
                series("net_return", t, |x| x.net_return),
                series("dividend_income", t, |x| x.dividend_income),
                series("unreinvested_dividends", t, |x| x.unreinvested_dividends),
                series("financing_cost", t, |x| x.financing_cost),
                series("is_partial", t, |x| x.is_partial),
                series("scale_sequence", t, |x| x.scale_sequence as u64),
            ],
        )
    }

    /// Converts the generated signals to a polars DataFrame, one row per signal.
    ///
    /// Columns are `timestamp` (datetime, UTC), `price` (f64), `direction` (string, the
    /// `SignalDirection` display name), `strength` (f64), `reason` (nullable string)
    /// and `executed` (bool). Tags are not included.
    pub fn signals_dataframe(&self) -> PolarsResult<DataFrame> {
        let s = &self.signals;
        DataFrame::new(
            s.len(),
            vec![
                datetimes("timestamp", s, |x| x.timestamp),
                series("price", s, |x| x.price),
                series("direction", s, |x| x.direction.as_str()),
                series("strength", s, |x| x.strength),
                Series::new(
                    "reason".into(),
                    s.iter().map(|x| x.reason.as_deref()).collect::<Vec<_>>(),
                )
                .into(),
                series("executed", s, |x| x.executed),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::PerformanceMetrics;
    use super::super::fixtures::{equity_point, make_result, make_trade};
    use super::*;
    use polars::prelude::DataType;

    fn names(df: &DataFrame) -> Vec<String> {
        df.get_column_names()
            .iter()
            .map(|n| n.to_string())
            .collect()
    }

    #[test]
    fn equity_dataframe_columns_and_order() {
        let curve = vec![
            equity_point(1, 100.0, 0.0),
            equity_point(2, 90.0, 0.1),
            equity_point(3, 95.0, 0.05),
        ];
        let df = make_result(vec![], curve).equity_dataframe().unwrap();

        assert_eq!(names(&df), ["timestamp", "equity", "drawdown"]);
        assert_eq!(
            df.column("timestamp").unwrap().dtype(),
            &finance_query_types::__dataframe::datetime_dtype()
        );
        let micros = df
            .column("timestamp")
            .unwrap()
            .cast(&DataType::Int64)
            .unwrap();
        assert_eq!(micros.i64().unwrap().get(0), Some(1_000_000));
        assert_eq!(df.column("equity").unwrap().dtype(), &DataType::Float64);
        let equity: Vec<_> = df
            .column("equity")
            .unwrap()
            .f64()
            .unwrap()
            .into_no_null_iter()
            .collect();
        assert_eq!(equity, [100.0, 90.0, 95.0]);
    }

    #[test]
    fn trades_dataframe_maps_side_to_string() {
        let trades = vec![make_trade(50.0, 5.0, true), make_trade(-20.0, -2.0, false)];
        let df = make_result(trades, vec![]).trades_dataframe().unwrap();

        assert_eq!(
            names(&df),
            [
                "side",
                "entry_timestamp",
                "exit_timestamp",
                "entry_price",
                "exit_price",
                "quantity",
                "entry_quantity",
                "commission",
                "transaction_tax",
                "pnl",
                "net_return",
                "dividend_income",
                "unreinvested_dividends",
                "financing_cost",
                "is_partial",
                "scale_sequence",
            ]
        );
        assert_eq!(df.column("side").unwrap().dtype(), &DataType::String);
        assert_eq!(
            df.column("entry_timestamp").unwrap().dtype(),
            &finance_query_types::__dataframe::datetime_dtype()
        );
        assert_eq!(df.column("pnl").unwrap().dtype(), &DataType::Float64);
        assert_eq!(df.column("is_partial").unwrap().dtype(), &DataType::Boolean);
        assert_eq!(
            df.column("scale_sequence").unwrap().dtype(),
            &DataType::UInt64
        );
        let sides: Vec<_> = df
            .column("side")
            .unwrap()
            .str()
            .unwrap()
            .iter()
            .map(Option::unwrap)
            .collect();
        assert_eq!(sides, ["LONG", "SHORT"]);
    }

    #[test]
    fn signals_dataframe_maps_direction_and_nullable_reason() {
        let mut result = make_result(vec![], vec![]);
        result.signals = serde_json::from_value(serde_json::json!([
            {"timestamp": 1, "price": 10.0, "direction": "Long", "strength": 0.8,
             "reason": "cross", "executed": true},
            {"timestamp": 2, "price": 11.0, "direction": "Exit", "strength": 0.5,
             "reason": null, "executed": false}
        ]))
        .unwrap();
        let df = result.signals_dataframe().unwrap();

        assert_eq!(
            names(&df),
            [
                "timestamp",
                "price",
                "direction",
                "strength",
                "reason",
                "executed"
            ]
        );
        assert_eq!(df.column("direction").unwrap().dtype(), &DataType::String);
        assert_eq!(df.column("executed").unwrap().dtype(), &DataType::Boolean);
        let reason = df.column("reason").unwrap().str().unwrap();
        assert_eq!(reason.get(0), Some("cross"));
        assert_eq!(reason.get(1), None);
        let direction = df.column("direction").unwrap().str().unwrap();
        assert_eq!(direction.get(1), Some("EXIT"));
    }

    #[test]
    fn empty_result_keeps_schema() {
        let result = make_result(vec![], vec![]);

        assert_eq!(result.trades_dataframe().unwrap().height(), 0);
        assert_eq!(result.trades_dataframe().unwrap().width(), 16);
        assert_eq!(result.equity_dataframe().unwrap().height(), 0);
        assert_eq!(result.signals_dataframe().unwrap().width(), 6);
    }

    #[test]
    fn metrics_dataframe_is_single_row_with_typed_columns() {
        let result = make_result(vec![make_trade(50.0, 5.0, true)], vec![]);
        let df = result.metrics.to_dataframe().unwrap();

        assert_eq!(df.height(), 1);
        assert_eq!(df.width(), 39);
        assert_eq!(
            names(&df)[..3],
            ["total_return", "annualized_return", "sharpe_ratio"]
        );
        assert_eq!(
            df.column("total_trades").unwrap().dtype(),
            &DataType::UInt64
        );
        assert_eq!(
            df.column("max_drawdown_duration").unwrap().dtype(),
            &DataType::Int64
        );
        assert_eq!(df.column("win_rate").unwrap().dtype(), &DataType::Float64);
        assert_eq!(
            df.column("total_trades").unwrap().u64().unwrap().get(0),
            Some(1)
        );
    }

    #[test]
    fn metrics_rows_stack_through_the_trait() {
        let metrics = make_result(vec![make_trade(50.0, 5.0, true)], vec![]).metrics;
        let single = metrics.to_dataframe().unwrap();
        let df = <PerformanceMetrics as finance_query_types::ToDataFrame>::vec_to_dataframe(&[
            metrics.clone(),
            metrics,
        ])
        .unwrap();

        assert_eq!(df.height(), 2);
        assert_eq!(df.schema(), single.schema());
    }

    fn assert_covers_fields(frame: &DataFrame, value: &impl serde::Serialize, skipped: &[&str]) {
        let mut fields: Vec<String> = serde_json::to_value(value)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .filter(|k| !skipped.contains(&k.as_str()))
            .cloned()
            .collect();
        fields.sort();
        let mut columns = names(frame);
        columns.sort();
        assert_eq!(columns, fields);
    }

    #[test]
    fn metrics_frame_covers_every_field() {
        let result = make_result(vec![make_trade(50.0, 5.0, true)], vec![]);
        let df = result.metrics.to_dataframe().unwrap();

        assert_covers_fields(&df, &result.metrics, &[]);
    }

    #[test]
    fn trades_frame_covers_every_scalar_field() {
        let trade = make_trade(50.0, 5.0, true);
        let df = make_result(vec![trade.clone()], vec![])
            .trades_dataframe()
            .unwrap();

        assert_covers_fields(&df, &trade, &["entry_signal", "exit_signal", "tags"]);
    }

    #[test]
    fn signals_frame_covers_every_scalar_field() {
        let mut result = make_result(vec![], vec![]);
        result.signals = serde_json::from_value(serde_json::json!([
            {"timestamp": 1, "price": 10.0, "direction": "Long", "strength": 0.8,
             "reason": null, "executed": true}
        ]))
        .unwrap();
        let df = result.signals_dataframe().unwrap();

        assert_covers_fields(&df, &result.signals[0], &["tags"]);
    }
}

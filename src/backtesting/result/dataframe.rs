//! Polars conversions for backtest results.

use super::{BacktestResult, EquityPoint, PerformanceMetrics};
use polars::prelude::{Column, DataFrame, NamedFrom, PolarsResult, Series};

fn column<T>(name: &str, value: T) -> Column
where
    Series: NamedFrom<Vec<T>, [T]>,
{
    Series::new(name.into(), vec![value]).into()
}

impl PerformanceMetrics {
    /// Converts the metrics to a single-row polars DataFrame.
    ///
    /// Columns follow the field order of [`PerformanceMetrics`], named in snake case.
    /// Counts are `u64`, `max_drawdown_duration` and `max_idle_period` are `i64`, and
    /// everything else is `f64`.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        DataFrame::new(
            1,
            vec![
                column("total_return_pct", self.total_return_pct),
                column("annualized_return_pct", self.annualized_return_pct),
                column("sharpe_ratio", self.sharpe_ratio),
                column("sortino_ratio", self.sortino_ratio),
                column("max_drawdown_pct", self.max_drawdown_pct),
                column("max_drawdown_duration", self.max_drawdown_duration),
                column("win_rate", self.win_rate),
                column("profit_factor", self.profit_factor),
                column("avg_trade_return_pct", self.avg_trade_return_pct),
                column("avg_win_pct", self.avg_win_pct),
                column("avg_loss_pct", self.avg_loss_pct),
                column("avg_trade_duration", self.avg_trade_duration),
                column("total_trades", self.total_trades as u64),
                column("winning_trades", self.winning_trades as u64),
                column("losing_trades", self.losing_trades as u64),
                column("largest_win", self.largest_win),
                column("largest_loss", self.largest_loss),
                column("max_consecutive_wins", self.max_consecutive_wins as u64),
                column("max_consecutive_losses", self.max_consecutive_losses as u64),
                column("calmar_ratio", self.calmar_ratio),
                column("total_commission", self.total_commission),
                column("total_financing_cost", self.total_financing_cost),
                column("long_trades", self.long_trades as u64),
                column("short_trades", self.short_trades as u64),
                column("total_signals", self.total_signals as u64),
                column("executed_signals", self.executed_signals as u64),
                column("avg_win_duration", self.avg_win_duration),
                column("avg_loss_duration", self.avg_loss_duration),
                column("time_in_market_pct", self.time_in_market_pct),
                column("max_idle_period", self.max_idle_period),
                column("total_dividend_income", self.total_dividend_income),
                column("kelly_criterion", self.kelly_criterion),
                column("sqn", self.sqn),
                column("expectancy", self.expectancy),
                column("omega_ratio", self.omega_ratio),
                column("tail_ratio", self.tail_ratio),
                column("recovery_factor", self.recovery_factor),
                column("ulcer_index", self.ulcer_index),
                column("serenity_ratio", self.serenity_ratio),
            ],
        )
    }
}

impl BacktestResult {
    /// Converts the equity curve to a polars DataFrame, one row per bar.
    ///
    /// Columns are `timestamp` (i64), `equity` (f64) and `drawdown_pct` (f64, a fraction).
    pub fn equity_dataframe(&self) -> PolarsResult<DataFrame> {
        EquityPoint::vec_to_dataframe(&self.equity_curve)
    }

    /// Converts the trade log to a polars DataFrame, one row per trade, in log order.
    ///
    /// Columns are `side` (string, `LONG` or `SHORT`), `entry_timestamp`, `exit_timestamp`
    /// (i64), `entry_price`, `exit_price`, `quantity`, `entry_quantity`, `commission`,
    /// `transaction_tax`, `pnl`, `return_pct`, `dividend_income`, `unreinvested_dividends`,
    /// `financing_cost` (f64), `is_partial` (bool) and `scale_sequence` (u64). Entry and exit
    /// signals and tags are not included.
    pub fn trades_dataframe(&self) -> PolarsResult<DataFrame> {
        let trades = &self.trades;
        let floats = |name: &str, get: fn(&crate::backtesting::Trade) -> f64| {
            Series::new(name.into(), trades.iter().map(get).collect::<Vec<_>>()).into()
        };
        let timestamps = |name: &str, get: fn(&crate::backtesting::Trade) -> i64| {
            Series::new(name.into(), trades.iter().map(get).collect::<Vec<_>>()).into()
        };

        DataFrame::new(
            trades.len(),
            vec![
                Series::new(
                    "side".into(),
                    trades
                        .iter()
                        .map(|t| t.side.to_string())
                        .collect::<Vec<_>>(),
                )
                .into(),
                timestamps("entry_timestamp", |t| t.entry_timestamp),
                timestamps("exit_timestamp", |t| t.exit_timestamp),
                floats("entry_price", |t| t.entry_price),
                floats("exit_price", |t| t.exit_price),
                floats("quantity", |t| t.quantity),
                floats("entry_quantity", |t| t.entry_quantity),
                floats("commission", |t| t.commission),
                floats("transaction_tax", |t| t.transaction_tax),
                floats("pnl", |t| t.pnl),
                floats("return_pct", |t| t.return_pct),
                floats("dividend_income", |t| t.dividend_income),
                floats("unreinvested_dividends", |t| t.unreinvested_dividends),
                floats("financing_cost", |t| t.financing_cost),
                Series::new(
                    "is_partial".into(),
                    trades.iter().map(|t| t.is_partial).collect::<Vec<_>>(),
                )
                .into(),
                Series::new(
                    "scale_sequence".into(),
                    trades
                        .iter()
                        .map(|t| t.scale_sequence as u64)
                        .collect::<Vec<_>>(),
                )
                .into(),
            ],
        )
    }

    /// Converts the generated signals to a polars DataFrame, one row per signal.
    ///
    /// Columns are `timestamp` (i64), `price` (f64), `direction` (string, the
    /// `SignalDirection` display name), `strength` (f64), `reason` (nullable string)
    /// and `executed` (bool). Tags are not included.
    pub fn signals_dataframe(&self) -> PolarsResult<DataFrame> {
        let signals = &self.signals;
        DataFrame::new(
            signals.len(),
            vec![
                Series::new(
                    "timestamp".into(),
                    signals.iter().map(|s| s.timestamp).collect::<Vec<_>>(),
                )
                .into(),
                Series::new(
                    "price".into(),
                    signals.iter().map(|s| s.price).collect::<Vec<_>>(),
                )
                .into(),
                Series::new(
                    "direction".into(),
                    signals
                        .iter()
                        .map(|s| s.direction.to_string())
                        .collect::<Vec<_>>(),
                )
                .into(),
                Series::new(
                    "strength".into(),
                    signals.iter().map(|s| s.strength).collect::<Vec<_>>(),
                )
                .into(),
                Series::new(
                    "reason".into(),
                    signals
                        .iter()
                        .map(|s| s.reason.as_deref())
                        .collect::<Vec<_>>(),
                )
                .into(),
                Series::new(
                    "executed".into(),
                    signals.iter().map(|s| s.executed).collect::<Vec<_>>(),
                )
                .into(),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
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

        assert_eq!(names(&df), ["timestamp", "equity", "drawdown_pct"]);
        assert_eq!(df.column("timestamp").unwrap().dtype(), &DataType::Int64);
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
                "return_pct",
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
            &DataType::Int64
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
            .into_no_null_iter()
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
            ["total_return_pct", "annualized_return_pct", "sharpe_ratio"]
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

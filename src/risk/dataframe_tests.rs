use super::RiskSummary;
use super::drawdown::DrawdownResult;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[test]
fn risk_summary_columns() {
    let row: RiskSummary = serde_json::from_value(json!({"var_95": 1.5, "var_99": 1.5, "parametric_var_95": 1.5, "cvar_95": 1.5, "cvar_99": 1.5, "parametric_cvar_95": 1.5, "omega": 1.5, "kelly": 1.5, "max_drawdown": 1.5, "ulcer_index": 1.5})).unwrap();
    let expected = [
        "var_95:f64",
        "var_99:f64",
        "parametric_var_95:f64",
        "cvar_95:f64",
        "cvar_99:f64",
        "parametric_cvar_95:f64",
        "omega:f64",
        "kelly:f64",
        "sharpe:f64",
        "sortino:f64",
        "calmar:f64",
        "beta:f64",
        "max_drawdown:f64",
        "max_drawdown_recovery_periods:u64",
        "ulcer_index:f64",
        "information_ratio:f64",
        "tracking_error:f64",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <RiskSummary>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn drawdown_result_columns() {
    let row = DrawdownResult {
        max_drawdown: 0.3,
        recovery_periods: None,
    };
    let expected = ["max_drawdown:f64", "recovery_periods:u64"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = DrawdownResult::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

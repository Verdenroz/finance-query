use thiserror::Error;

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Finance query error: {0}")]
    FinanceQuery(#[from] finance_query::YahooError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV serialization error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Indicator calculation error: {0}")]
    Indicator(#[from] finance_query::indicators::IndicatorError),

    #[error("Backtest error: {0}")]
    Backtest(#[from] finance_query::backtesting::BacktestError),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

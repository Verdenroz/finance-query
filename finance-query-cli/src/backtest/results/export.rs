use finance_query::backtesting::BacktestResult;
use finance_query::backtesting::portfolio::PortfolioResult;
use std::path::PathBuf;

use super::format::format_timestamp;

// CSV export

pub(super) fn export_trades_csv(result: &BacktestResult) -> Result<PathBuf, String> {
    use std::io::Write;

    let filename = format!(
        "backtest_{}_{}.csv",
        result.symbol.to_lowercase(),
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let export_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fq")
        .join("exports");
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;
    let path = export_dir.join(&filename);

    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;

    writeln!(file, "side,entry_date,exit_date,entry_price,exit_price,quantity,pnl,return_pct,commission,dividend_income")
        .map_err(|e| e.to_string())?;

    for trade in &result.trades {
        let side = if trade.is_long() { "LONG" } else { "SHORT" };
        writeln!(
            file,
            "{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
            side,
            format_timestamp(trade.entry_timestamp),
            format_timestamp(trade.exit_timestamp),
            trade.entry_price,
            trade.exit_price,
            trade.quantity,
            trade.pnl,
            trade.return_pct,
            trade.commission,
            trade.dividend_income,
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(path)
}

pub(super) fn export_portfolio_csv(portfolio: &PortfolioResult) -> Result<PathBuf, String> {
    use std::io::Write;

    let filename = format!(
        "portfolio_backtest_{}.csv",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let export_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fq")
        .join("exports");
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;
    let path = export_dir.join(&filename);

    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;

    writeln!(
        file,
        "symbol,side,entry_date,exit_date,entry_price,exit_price,quantity,pnl,return_pct,commission,dividend_income"
    )
    .map_err(|e| e.to_string())?;

    let mut symbols: Vec<&str> = portfolio.symbols.keys().map(|s| s.as_str()).collect();
    symbols.sort();

    for sym in symbols {
        if let Some(result) = portfolio.symbols.get(sym) {
            for trade in &result.trades {
                let side = if trade.is_long() { "LONG" } else { "SHORT" };
                writeln!(
                    file,
                    "{},{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
                    sym,
                    side,
                    format_timestamp(trade.entry_timestamp),
                    format_timestamp(trade.exit_timestamp),
                    trade.entry_price,
                    trade.exit_price,
                    trade.quantity,
                    trade.pnl,
                    trade.return_pct,
                    trade.commission,
                    trade.dividend_income,
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(path)
}

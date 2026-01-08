use crate::error::Result;
use crate::output;
use clap::Parser;
use finance_query::{Ticker, TimeRange};

#[derive(Parser)]
pub struct DividendsArgs {
    /// Stock symbol to get dividends for
    #[arg(required = true)]
    symbol: String,

    /// Time range for dividend history
    #[arg(short, long, default_value = "1y")]
    range: String,
}

pub async fn execute(args: DividendsArgs) -> Result<()> {
    let range = parse_range(&args.range)?;
    let ticker = Ticker::new(&args.symbol).await?;
    let mut dividends = ticker.dividends(range).await?;

    if dividends.is_empty() {
        output::print_info(&format!(
            "No dividends found for {} in range {}",
            args.symbol, args.range
        ));
        return Ok(());
    }

    // Reverse to show oldest first (most recent at bottom)
    dividends.sort_by_key(|d| d.timestamp);

    output::print_success(&format!(
        "Dividends for {} ({} total)",
        args.symbol,
        dividends.len()
    ));
    println!();

    // Header
    println!("{:<12} {:<15}", "Date", "Amount");
    println!("{}", "-".repeat(30));

    // Data rows
    for dividend in &dividends {
        let date = chrono::DateTime::from_timestamp(dividend.timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        println!("{:<12} ${:<14.4}", date, dividend.amount);
    }

    Ok(())
}

fn parse_range(s: &str) -> Result<TimeRange> {
    match s.to_lowercase().as_str() {
        "1d" => Ok(TimeRange::OneDay),
        "5d" => Ok(TimeRange::FiveDays),
        "1mo" => Ok(TimeRange::OneMonth),
        "3mo" => Ok(TimeRange::ThreeMonths),
        "6mo" => Ok(TimeRange::SixMonths),
        "1y" => Ok(TimeRange::OneYear),
        "2y" => Ok(TimeRange::TwoYears),
        "5y" => Ok(TimeRange::FiveYears),
        "10y" => Ok(TimeRange::TenYears),
        "ytd" => Ok(TimeRange::YearToDate),
        "max" => Ok(TimeRange::Max),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid range '{}'. Valid ranges: 1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max",
            s
        ))),
    }
}

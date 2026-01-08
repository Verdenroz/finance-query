use crate::error::Result;
use crate::output;
use clap::Parser;
use finance_query::{Ticker, TimeRange};

#[derive(Parser)]
pub struct SplitsArgs {
    /// Stock symbol to get splits for
    #[arg(required = true)]
    symbol: String,

    /// Time range for split history
    #[arg(short, long, default_value = "10y")]
    range: String,
}

pub async fn execute(args: SplitsArgs) -> Result<()> {
    let range = parse_range(&args.range)?;
    let ticker = Ticker::new(&args.symbol).await?;
    let mut splits = ticker.splits(range).await?;

    if splits.is_empty() {
        output::print_info(&format!(
            "No splits found for {} in range {}",
            args.symbol, args.range
        ));
        return Ok(());
    }

    // Sort by timestamp ascending (oldest first, most recent at bottom)
    splits.sort_by_key(|s| s.timestamp);

    output::print_success(&format!(
        "Stock splits for {} ({} total)",
        args.symbol,
        splits.len()
    ));
    println!();

    // Header
    println!("{:<12} {:<15} {:<10}", "Date", "Ratio", "Details");
    println!("{}", "-".repeat(40));

    // Data rows
    for split in &splits {
        let date = chrono::DateTime::from_timestamp(split.timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        println!(
            "{:<12} {:<15} {}:{}",
            date, split.ratio, split.numerator, split.denominator
        );
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

use crate::error::Result;
use crate::output;
use crate::parse::parse_range;
use clap::Parser;

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
    let ticker = crate::lang::ticker(&args.symbol).await?;
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

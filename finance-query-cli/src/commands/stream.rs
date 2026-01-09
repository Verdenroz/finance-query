use crate::error::Result;
use crate::output;
use clap::Parser;
use colored::Colorize;
use finance_query::streaming::{MarketHoursType, PriceStream};
use futures::StreamExt;

#[derive(Parser)]
pub struct StreamArgs {
    /// Stock symbols to stream prices for (e.g., AAPL MSFT TSLA)
    #[arg(required = true)]
    symbols: Vec<String>,
}

pub async fn execute(args: StreamArgs) -> Result<()> {
    output::print_info(&format!(
        "Streaming prices for: {}",
        args.symbols.join(", ")
    ));
    output::print_info("Press Ctrl+C to stop");
    println!();

    // Subscribe to price stream
    let mut stream =
        PriceStream::subscribe(&args.symbols.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .await
            .map_err(|e| crate::error::CliError::Other(e.into()))?;

    // Process price updates as they arrive
    while let Some(price) = stream.next().await {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();

        // Color code the change
        let change_str = if price.change >= 0.0 {
            format!("+{:.2}", price.change).green()
        } else {
            format!("{:.2}", price.change).red()
        };

        let change_pct_str = if price.change_percent >= 0.0 {
            format!("+{:.2}%", price.change_percent).green()
        } else {
            format!("{:.2}%", price.change_percent).red()
        };

        let market_label = format_market_hours(price.market_hours);
        println!(
            "{} {} ${:.2} {} {} Vol: {} {}",
            timestamp.bright_black(),
            price.id.bright_white().bold(),
            price.price,
            change_str,
            change_pct_str,
            format_volume(price.day_volume),
            market_label
        );
    }

    Ok(())
}

/// Format volume with abbreviations (K, M, B)
fn format_volume(volume: i64) -> String {
    match volume {
        v if v >= 1_000_000_000 => format!("{:.2}B", v as f64 / 1_000_000_000.0),
        v if v >= 1_000_000 => format!("{:.2}M", v as f64 / 1_000_000.0),
        v if v >= 1_000 => format!("{:.2}K", v as f64 / 1_000.0),
        _ => volume.to_string(),
    }
}

/// Format market hours type with colored output
fn format_market_hours(market_hours: MarketHoursType) -> colored::ColoredString {
    match market_hours {
        MarketHoursType::RegularMarket => "Regular".green(),
        MarketHoursType::PreMarket => {
            // Distinguish overnight (8 PM - 4 AM ET) from pre-market (4 AM - 9:30 AM ET)
            if is_overnight_session() {
                "Overnight".cyan()
            } else {
                "Pre-Market".yellow()
            }
        }
        MarketHoursType::PostMarket => "After-Hours".yellow(),
        MarketHoursType::ExtendedHoursMarket => "Extended".cyan(),
    }
}

/// Check if current time is during overnight trading session (8 PM - 4 AM ET)
fn is_overnight_session() -> bool {
    use chrono::{Timelike, Utc};
    use chrono_tz::America::New_York;

    let now_et = Utc::now().with_timezone(&New_York);
    let hour = now_et.hour();

    // Overnight: 8 PM (20:00) to 4 AM (04:00) ET
    hour >= 20 || hour < 4
}

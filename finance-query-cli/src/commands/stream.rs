use crate::error::Result;
use crate::output;
use clap::Parser;
use colored::Colorize;
use finance_query::streaming::PriceStream;
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
        let timestamp = chrono::DateTime::from_timestamp(price.time, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "N/A".to_string());

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

        println!(
            "{} {} ${:.2} {} {} Vol: {} Market: {:?}",
            timestamp.bright_black(),
            price.id.bright_white().bold(),
            price.price,
            change_str,
            change_pct_str,
            format_volume(price.day_volume),
            price.market_hours
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

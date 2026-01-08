use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use colored::Colorize;
use finance_query::Tickers;
use serde::Serialize;
use tabled::Tabled;

#[derive(Parser)]
pub struct QuoteArgs {
    /// Stock symbols to get quotes for (e.g., AAPL MSFT TSLA)
    #[arg(required = true)]
    symbols: Vec<String>,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,

    /// Include company logo URLs
    #[arg(short, long)]
    logo: bool,
}

#[derive(Debug, Serialize, Tabled)]
struct QuoteDisplay {
    #[tabled(rename = "Symbol")]
    symbol: String,

    #[tabled(rename = "Name")]
    name: String,

    #[tabled(rename = "Price")]
    price: String,

    #[tabled(rename = "Change")]
    change: String,

    #[tabled(rename = "Change %")]
    change_percent: String,

    #[tabled(rename = "Volume")]
    volume: String,

    #[tabled(rename = "Market Cap")]
    market_cap: String,

    #[tabled(rename = "Exchange")]
    exchange: String,
}

pub async fn execute(args: QuoteArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;

    // Use Tickers for efficient batch fetching
    let tickers = Tickers::new(&args.symbols).await?;
    let response = tickers.quotes(args.logo).await?;

    // Convert successful quotes to display format
    let mut quotes = Vec::new();
    for (symbol, quote) in &response.quotes {
        let display = QuoteDisplay {
            symbol: symbol.clone(),
            name: quote
                .short_name
                .clone()
                .or_else(|| quote.long_name.clone())
                .unwrap_or_else(|| "N/A".to_string()),
            price: quote
                .regular_market_price
                .as_ref()
                .and_then(|v| v.raw.map(|r| format!("{:.2}", r)))
                .unwrap_or_else(|| "N/A".to_string()),
            change: quote
                .regular_market_change
                .as_ref()
                .and_then(|v| {
                    v.raw.map(|r| {
                        if r >= 0.0 {
                            format!("+{:.2}", r)
                        } else {
                            format!("{:.2}", r)
                        }
                    })
                })
                .unwrap_or_else(|| "N/A".to_string()),
            change_percent: quote
                .regular_market_change_percent
                .as_ref()
                .and_then(|v| {
                    v.raw.map(|r| {
                        if r >= 0.0 {
                            format!("+{:.2}%", r)
                        } else {
                            format!("{:.2}%", r)
                        }
                    })
                })
                .unwrap_or_else(|| "N/A".to_string()),
            volume: quote
                .regular_market_volume
                .as_ref()
                .and_then(|v| v.raw.map(format_volume))
                .unwrap_or_else(|| "N/A".to_string()),
            market_cap: quote
                .market_cap
                .as_ref()
                .and_then(|v| v.raw.map(format_market_cap))
                .unwrap_or_else(|| "N/A".to_string()),
            exchange: quote.exchange.clone().unwrap_or_else(|| "N/A".to_string()),
        };

        quotes.push(display);
    }

    // Print successful quotes
    if !quotes.is_empty() {
        output::print_many(&quotes, format)?;
    }

    // Print errors if any
    if !response.errors.is_empty() && format == OutputFormat::Table {
        eprintln!();
        eprintln!("{}", "Errors:".red().bold());
        for (symbol, error) in &response.errors {
            eprintln!("  {} {}: {}", "✗".red(), symbol.yellow(), error);
        }
    }

    // Print summary for table format (only if multiple symbols or errors)
    if format == OutputFormat::Table && (args.symbols.len() > 1 || !response.errors.is_empty()) {
        eprintln!();
        eprintln!(
            "{} {} successful, {} failed",
            "Summary:".blue().bold(),
            response.success_count(),
            response.error_count()
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

/// Format market cap with abbreviations (M, B, T)
fn format_market_cap(market_cap: i64) -> String {
    match market_cap {
        v if v >= 1_000_000_000_000 => format!("{:.2}T", v as f64 / 1_000_000_000_000.0),
        v if v >= 1_000_000_000 => format!("{:.2}B", v as f64 / 1_000_000_000.0),
        v if v >= 1_000_000 => format!("{:.2}M", v as f64 / 1_000_000.0),
        _ => market_cap.to_string(),
    }
}

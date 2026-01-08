use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use finance_query::ScreenerType;
use serde::Serialize;
use tabled::Tabled;

#[derive(Parser)]
pub struct ScreenerArgs {
    /// Screener type (most-actives, day-gainers, day-losers, etc.)
    #[arg(required = true)]
    screener_type: String,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,

    /// Maximum number of results to return
    #[arg(short, long, default_value = "25")]
    limit: u32,
}

#[derive(Debug, Serialize, Tabled)]
struct ScreenerDisplay {
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

    #[tabled(rename = "Exchange")]
    exchange: String,
}

pub async fn execute(args: ScreenerArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;

    // Parse screener type
    let screener_type = parse_screener_type(&args.screener_type)?;

    // Fetch screener results
    let results = finance_query::finance::screener(screener_type, args.limit).await?;

    let mut screener_items = Vec::new();
    for quote in &results.quotes {
        let price = quote
            .regular_market_price
            .raw
            .map(|v| format!("{:.2}", v))
            .unwrap_or_else(|| "N/A".to_string());

        let change = quote
            .regular_market_change
            .raw
            .map(|v| match v {
                c if c >= 0.0 => format!("+{:.2}", c),
                c => format!("{:.2}", c),
            })
            .unwrap_or_else(|| "N/A".to_string());

        let change_percent = quote
            .regular_market_change_percent
            .raw
            .map(|v| match v {
                c if c >= 0.0 => format!("+{:.2}%", c),
                c => format!("{:.2}%", c),
            })
            .unwrap_or_else(|| "N/A".to_string());

        let volume = quote
            .regular_market_volume
            .as_ref()
            .and_then(|v| v.raw.map(format_volume))
            .unwrap_or_else(|| "N/A".to_string());

        screener_items.push(ScreenerDisplay {
            symbol: quote.symbol.clone(),
            name: quote.short_name.clone(),
            price,
            change,
            change_percent,
            volume,
            exchange: quote.exchange.clone(),
        });
    }

    output::print_many(&screener_items, format)?;

    Ok(())
}

fn parse_screener_type(s: &str) -> Result<ScreenerType> {
    ScreenerType::parse(s).ok_or_else(|| {
        crate::error::CliError::InvalidArgument(format!(
            "Invalid screener type '{}'. Valid types: {}",
            s,
            ScreenerType::valid_types()
        ))
    })
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

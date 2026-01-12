use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use serde::Serialize;
use tabled::Tabled;

#[derive(Parser)]
pub struct MarketArgs {
    /// Region code (US, GB, JP, etc.) - optional
    #[arg(short, long)]
    region: Option<String>,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,
}

#[derive(Debug, Serialize, Tabled)]
struct MarketDisplay {
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

    #[tabled(rename = "State")]
    state: String,
}

pub async fn execute(args: MarketArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;

    // Parse region if provided
    let region = args.region.as_deref().and_then(parse_region);

    // Fetch market summary
    let markets = finance_query::finance::market_summary(region).await?;

    let mut market_items = Vec::new();
    for quote in &markets {
        let price = quote
            .regular_market_price
            .as_ref()
            .and_then(|v| v.raw.map(|p| format!("{:.2}", p)))
            .unwrap_or_else(|| "N/A".to_string());

        let change = quote
            .regular_market_change
            .as_ref()
            .and_then(|v| {
                v.raw.map(|c| match c {
                    val if val >= 0.0 => format!("+{:.2}", val),
                    val => format!("{:.2}", val),
                })
            })
            .unwrap_or_else(|| "N/A".to_string());

        let change_percent = quote
            .regular_market_change_percent
            .as_ref()
            .and_then(|v| {
                v.raw.map(|c| match c {
                    val if val >= 0.0 => format!("+{:.2}%", val),
                    val => format!("{:.2}%", val),
                })
            })
            .unwrap_or_else(|| "N/A".to_string());

        market_items.push(MarketDisplay {
            symbol: quote.symbol.clone(),
            name: quote
                .short_name
                .clone()
                .unwrap_or_else(|| "N/A".to_string()),
            price,
            change,
            change_percent,
            state: quote
                .market_state
                .clone()
                .unwrap_or_else(|| "N/A".to_string()),
        });
    }

    output::print_many(&market_items, format)?;

    Ok(())
}

fn parse_region(s: &str) -> Option<finance_query::Region> {
    use finance_query::Region;
    match s.to_uppercase().as_str() {
        "US" => Some(Region::UnitedStates),
        "GB" | "UK" => Some(Region::UnitedKingdom),
        "CA" => Some(Region::Canada),
        "AU" => Some(Region::Australia),
        "DE" => Some(Region::Germany),
        "FR" => Some(Region::France),
        "IT" => Some(Region::Italy),
        "ES" => Some(Region::Spain),
        "HK" => Some(Region::HongKong),
        "SG" => Some(Region::Singapore),
        "IN" => Some(Region::India),
        "BR" => Some(Region::Brazil),
        "CN" => Some(Region::China),
        "TW" => Some(Region::Taiwan),
        _ => None,
    }
}

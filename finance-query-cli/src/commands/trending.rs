use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use serde::Serialize;
use tabled::Tabled;

#[derive(Parser)]
pub struct TrendingArgs {
    /// Region code (US, GB, JP, etc.)
    #[arg(short, long, default_value = "US")]
    region: String,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,
}

#[derive(Debug, Serialize, Tabled)]
struct TrendingDisplay {
    #[tabled(rename = "Symbol")]
    symbol: String,
}

pub async fn execute(args: TrendingArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;

    // Parse region
    let region = parse_region(&args.region);

    // Fetch trending symbols
    let trending = finance_query::finance::trending(region).await?;

    let trending_items: Vec<TrendingDisplay> = trending
        .iter()
        .map(|quote| TrendingDisplay {
            symbol: quote.symbol.clone(),
        })
        .collect();

    if trending_items.is_empty() {
        output::print_info("No trending symbols found for this region");
        return Ok(());
    }

    output::print_many(&trending_items, format)?;

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

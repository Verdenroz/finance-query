use crate::error::Result;
use crate::output;
use clap::Parser;
use finance_query::{IndicesRegion, finance};

#[derive(Parser)]
pub struct IndicesArgs {
    /// Region filter (americas, europe, asia, africa-middle-east)
    #[arg(short, long)]
    region: Option<String>,
}

pub async fn execute(args: IndicesArgs) -> Result<()> {
    let region = if let Some(ref region_str) = args.region {
        Some(parse_region(region_str)?)
    } else {
        None
    };

    let response = finance::indices(region).await?;

    let success_count = response.success_count();
    if success_count == 0 {
        output::print_info("No indices data available");
        return Ok(());
    }

    let region_name = args.region.as_deref().unwrap_or("All");
    output::print_success(&format!(
        "World Market Indices - {} ({} indices)",
        region_name, success_count
    ));
    println!();

    // Header
    println!(
        "{:<15} {:<35} {:<12} {:<12} {:<12}",
        "Symbol", "Name", "Price", "Change", "Change %"
    );
    println!("{}", "-".repeat(95));

    // Data rows
    for quote in response.quotes.values() {
        let price = quote
            .regular_market_price
            .as_ref()
            .and_then(|p| p.raw)
            .map(|p| format!("${:.2}", p))
            .unwrap_or_else(|| "N/A".to_string());

        let change = quote
            .regular_market_change
            .as_ref()
            .and_then(|c| c.raw)
            .map(|c| {
                if c >= 0.0 {
                    format!("+${:.2}", c)
                } else {
                    format!("-${:.2}", c.abs())
                }
            })
            .unwrap_or_else(|| "N/A".to_string());

        let change_pct = quote
            .regular_market_change_percent
            .as_ref()
            .and_then(|p| p.raw)
            .map(|p| {
                if p >= 0.0 {
                    format!("+{:.2}%", p)
                } else {
                    format!("{:.2}%", p)
                }
            })
            .unwrap_or_else(|| "N/A".to_string());

        let name = quote
            .short_name
            .as_deref()
            .or(quote.long_name.as_deref())
            .unwrap_or("N/A");
        let name_truncated = truncate(name, 35);

        println!(
            "{:<15} {:<35} {:<12} {:<12} {:<12}",
            quote.symbol, name_truncated, price, change, change_pct
        );
    }

    Ok(())
}

fn parse_region(s: &str) -> Result<IndicesRegion> {
    match s.to_lowercase().as_str() {
        "americas" | "america" | "am" => Ok(IndicesRegion::Americas),
        "europe" | "eu" => Ok(IndicesRegion::Europe),
        "asia" | "asiapacific" | "asia-pacific" | "apac" => Ok(IndicesRegion::AsiaPacific),
        "mea" | "middleeastafrica" | "middle-east-africa" | "emea" => {
            Ok(IndicesRegion::MiddleEastAfrica)
        }
        "currencies" | "currency" | "fx" => Ok(IndicesRegion::Currencies),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid region '{}'. Valid regions: americas, europe, asia, mea, currencies",
            s
        ))),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

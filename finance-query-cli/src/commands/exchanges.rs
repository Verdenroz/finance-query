use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use serde::Serialize;
use tabled::Tabled;

#[derive(Parser)]
pub struct ExchangesArgs {
    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,
}

#[derive(Debug, Serialize, Tabled)]
struct ExchangeDisplay {
    #[tabled(rename = "Country")]
    country: String,
    #[tabled(rename = "Market")]
    market: String,
    #[tabled(rename = "Suffix")]
    suffix: String,
    #[tabled(rename = "Delay")]
    delay: String,
    #[tabled(rename = "Data Provider")]
    data_provider: String,
}

pub async fn execute(args: ExchangesArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;

    // Fetch exchanges
    let exchanges = finance_query::finance::exchanges().await?;

    let display: Vec<ExchangeDisplay> = exchanges
        .iter()
        .map(|e| ExchangeDisplay {
            country: e.country.clone(),
            market: e.market.clone(),
            suffix: e.suffix.clone(),
            delay: e.delay.clone(),
            data_provider: e.data_provider.clone(),
        })
        .collect();

    output::print_many(&display, format)?;

    Ok(())
}

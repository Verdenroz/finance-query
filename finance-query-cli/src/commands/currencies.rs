use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use serde::Serialize;
use tabled::Tabled;

#[derive(Parser)]
pub struct CurrenciesArgs {
    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,
}

#[derive(Debug, Serialize, Tabled)]
struct CurrencyDisplay {
    #[tabled(rename = "Symbol")]
    symbol: String,
    #[tabled(rename = "Name")]
    name: String,
}

pub async fn execute(args: CurrenciesArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;

    // Fetch currencies
    let currencies = finance_query::finance::currencies().await?;

    let display: Vec<CurrencyDisplay> = currencies
        .iter()
        .map(|c| CurrencyDisplay {
            symbol: c.symbol.clone().unwrap_or_else(|| "N/A".to_string()),
            name: c
                .long_name
                .clone()
                .or_else(|| c.short_name.clone())
                .unwrap_or_else(|| "N/A".to_string()),
        })
        .collect();

    output::print_many(&display, format)?;

    Ok(())
}

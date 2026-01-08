use crate::error::Result;
use crate::output;
use clap::Parser;
use finance_query::{LookupOptions, LookupType, finance};

#[derive(Parser)]
pub struct LookupArgs {
    /// Search query (company name, symbol, etc.)
    #[arg(required = true)]
    query: String,

    /// Asset type filter (equity, etf, mutualfund, index, future, currency, cryptocurrency)
    #[arg(short = 't', long)]
    asset_type: Option<String>,

    /// Maximum number of results to return
    #[arg(short, long, default_value = "10")]
    limit: u32,

    /// Include logo URLs (requires additional API call)
    #[arg(long)]
    include_logo: bool,
}

pub async fn execute(args: LookupArgs) -> Result<()> {
    let lookup_type = if let Some(ref type_str) = args.asset_type {
        Some(parse_lookup_type(type_str)?)
    } else {
        None
    };

    let mut options = LookupOptions::new().count(args.limit);

    if let Some(ltype) = lookup_type {
        options = options.lookup_type(ltype);
    }

    if args.include_logo {
        options = options.include_logo(true);
    }

    let results = finance::lookup(&args.query, &options).await?;

    if results.is_empty() {
        output::print_info(&format!("No results found for '{}'", args.query));
        return Ok(());
    }

    output::print_success(&format!(
        "Found {} results for '{}'",
        results.quotes().len(),
        args.query
    ));
    println!();

    // Header
    println!(
        "{:<10} {:<30} {:<15} {:<12} {:<15}",
        "Symbol", "Name", "Type", "Exchange", "Price"
    );
    println!("{}", "-".repeat(85));

    // Data rows
    for quote in results.quotes() {
        let name = quote
            .short_name
            .as_deref()
            .or(quote.long_name.as_deref())
            .unwrap_or("N/A");
        let name_truncated = if name.len() > 28 {
            format!("{}...", &name[..28])
        } else {
            name.to_string()
        };

        let quote_type = quote.quote_type.as_deref().unwrap_or("N/A");

        let exchange = quote.exchange.as_deref().unwrap_or("N/A");

        let price = quote
            .regular_market_price
            .map(|p| format!("${:.2}", p))
            .unwrap_or_else(|| "N/A".to_string());

        println!(
            "{:<10} {:<30} {:<15} {:<12} {:<15}",
            quote.symbol, name_truncated, quote_type, exchange, price
        );
    }

    Ok(())
}

fn parse_lookup_type(s: &str) -> Result<LookupType> {
    match s.to_lowercase().as_str() {
        "equity" | "stock" | "stocks" => Ok(LookupType::Equity),
        "etf" | "etfs" => Ok(LookupType::Etf),
        "mutualfund" | "fund" | "funds" => Ok(LookupType::MutualFund),
        "index" | "indices" | "indexes" => Ok(LookupType::Index),
        "future" | "futures" => Ok(LookupType::Future),
        "currency" | "currencies" | "forex" => Ok(LookupType::Currency),
        "cryptocurrency" | "crypto" => Ok(LookupType::Cryptocurrency),
        "all" => Ok(LookupType::All),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid asset type '{}'. Valid types: equity, etf, mutualfund, index, future, currency, cryptocurrency, all",
            s
        ))),
    }
}

use crate::error::Result;
use crate::options::run_options_tui;
use crate::output;
use clap::Parser;
use finance_query::Ticker;

#[derive(Parser)]
pub struct OptionsArgs {
    /// Stock symbol to get options for
    #[arg(required = true)]
    symbol: String,

    /// Expiration date (Unix timestamp) for non-interactive mode
    #[arg(short, long)]
    date: Option<i64>,

    /// Show only call options (non-interactive mode)
    #[arg(long, conflicts_with = "puts")]
    calls: bool,

    /// Show only put options (non-interactive mode)
    #[arg(long, conflicts_with = "calls")]
    puts: bool,

    /// Maximum number of contracts to display (non-interactive mode)
    #[arg(short, long, default_value = "50")]
    limit: usize,

    /// Use non-interactive plain text output instead of TUI
    #[arg(long)]
    plain: bool,

    /// List available expirations only (non-interactive)
    #[arg(long)]
    list_expirations: bool,
}

pub async fn execute(args: OptionsArgs) -> Result<()> {
    // Use TUI by default unless --plain or --list-expirations is specified
    if !args.plain && !args.list_expirations && args.date.is_none() {
        return run_options_tui(&args.symbol).await;
    }

    let ticker = Ticker::new(&args.symbol).await?;
    let options = ticker.options(args.date).await?;

    // List expirations mode
    if args.list_expirations || (args.date.is_none() && args.plain) {
        let expirations = options.expiration_dates();
        if expirations.is_empty() {
            output::print_info(&format!("No options available for {}", args.symbol));
            return Ok(());
        }

        output::print_success(&format!(
            "Available option expirations for {} ({} total)",
            args.symbol,
            expirations.len()
        ));
        println!();
        println!(
            "{:<15} {:<20} {:<15}",
            "Date", "Unix Timestamp", "Days to Exp"
        );
        println!("{}", "-".repeat(55));

        let now = chrono::Utc::now().timestamp();
        for exp in expirations.iter() {
            let date = chrono::DateTime::from_timestamp(*exp, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "N/A".to_string());
            let days = (*exp - now) / 86400;
            println!("{:<15} {:<20} {:<15}", date, exp, format!("{} days", days));
        }

        if !args.list_expirations {
            println!("\nUse --date <timestamp> to view contracts for a specific expiration");
            println!("Or run without --plain for interactive TUI mode");
        }

        return Ok(());
    }

    // Show option contracts in plain mode with a date
    let show_calls = args.calls || !args.puts;
    let show_puts = args.puts || !args.calls;

    if show_calls {
        let calls = options.calls();
        if !calls.is_empty() {
            output::print_success(&format!("Call options ({} total)", calls.len()));
            println!();
            print_contracts(&calls, args.limit);
        }
    }

    if show_calls && show_puts && !options.calls().is_empty() && !options.puts().is_empty() {
        println!();
    }

    if show_puts {
        let puts = options.puts();
        if !puts.is_empty() {
            output::print_success(&format!("Put options ({} total)", puts.len()));
            println!();
            print_contracts(&puts, args.limit);
        }
    }

    Ok(())
}

fn print_contracts(contracts: &finance_query::Contracts, limit: usize) {
    // Header
    println!(
        "{:<25} {:<10} {:<12} {:<10} {:<10} {:<10}",
        "Contract", "Strike", "Last Price", "Bid", "Ask", "Volume"
    );
    println!("{}", "-".repeat(85));

    // Data rows
    for contract in contracts.iter().take(limit) {
        println!(
            "{:<25} ${:<9.2} ${:<11.2} ${:<9.2} ${:<9.2} {:>10}",
            contract.contract_symbol,
            contract.strike,
            contract.last_price.unwrap_or(0.0),
            contract.bid.unwrap_or(0.0),
            contract.ask.unwrap_or(0.0),
            contract.volume.unwrap_or(0)
        );
    }

    if contracts.len() > limit {
        println!(
            "\n... and {} more (use --limit to show more)",
            contracts.len() - limit
        );
    }
}

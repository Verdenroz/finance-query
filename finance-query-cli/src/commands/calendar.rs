use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use colored::Colorize;
use finance_query::{EventKind, TimeRange};

#[derive(Parser)]
pub struct CalendarArgs {
    /// One or more stock symbols (e.g. AAPL MSFT TSLA)
    #[arg(required = true, num_args = 1..)]
    symbols: Vec<String>,

    /// Forward time window (1d, 5d, 1mo, 3mo, 6mo, 1y, …)
    #[arg(short, long, default_value = "1mo")]
    range: String,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,
}

pub async fn execute(args: CalendarArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;
    let range = parse_range(&args.range)?;

    let tickers = crate::lang::tickers(args.symbols.clone()).await?;
    let events = tickers.calendar(range).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&events)?);
            return Ok(());
        }
        OutputFormat::Csv => {
            println!("date,symbol,type,details");
            for e in &events {
                let (_, details) = describe(&e.event);
                println!(
                    "{},{},{},{}",
                    e.date,
                    e.symbol.as_deref().unwrap_or(""),
                    kind_str(&e.event),
                    details.replace(',', ";"),
                );
            }
            return Ok(());
        }
        OutputFormat::Table => {}
    }

    output::print_success(&format!(
        "Calendar for {}",
        args.symbols
            .iter()
            .map(|s| s.to_uppercase())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    println!();

    if events.is_empty() {
        println!("  {}", "No upcoming events in this window".dimmed());
        return Ok(());
    }

    println!(
        "{:<12} {:<8} {:<22} {}",
        "DATE".bold(),
        "SYMBOL".bold(),
        "EVENT".bold(),
        "DETAILS".bold()
    );
    println!("{}", "─".repeat(70));

    for e in &events {
        let (label, details) = describe(&e.event);
        let symbol = e.symbol.as_deref().unwrap_or("—");
        println!("{:<12} {:<8} {:<22} {}", e.date, symbol, label, details);
    }

    let symbol_count = events
        .iter()
        .filter_map(|e| e.symbol.as_deref())
        .collect::<std::collections::HashSet<_>>()
        .len();
    println!();
    println!(
        "{}",
        format!(
            "{} event{} across {} symbol{} over the next {}.",
            events.len(),
            if events.len() == 1 { "" } else { "s" },
            symbol_count,
            if symbol_count == 1 { "" } else { "s" },
            args.range
        )
        .dimmed()
    );

    Ok(())
}

/// A human-readable `(label, details)` pair for an event kind.
fn describe(event: &EventKind) -> (String, String) {
    match event {
        EventKind::Earnings {
            eps_estimate_low,
            eps_estimate_avg,
            eps_estimate_high,
            ..
        } => {
            let details = match (eps_estimate_low, eps_estimate_avg, eps_estimate_high) {
                (Some(lo), Some(avg), Some(hi)) => {
                    format!("EPS: ${:.2}–${:.2} avg ${:.2}", lo, hi, avg)
                }
                (_, Some(avg), _) => format!("EPS avg ${:.2}", avg),
                _ => String::new(),
            };
            ("Earnings (est.)".to_string(), details)
        }
        EventKind::ExDividend { amount } => (
            "Ex-Dividend".to_string(),
            amount
                .map(|a| format!("${:.2}/share", a))
                .unwrap_or_default(),
        ),
        EventKind::DividendPayment { amount } => (
            "Dividend Payment".to_string(),
            amount
                .map(|a| format!("${:.2}/share", a))
                .unwrap_or_default(),
        ),
        EventKind::OptionsExpiration { contract_count } => (
            "Options Expiration".to_string(),
            match contract_count {
                Some(n) => format!("{} contracts", n),
                None => "monthly".to_string(),
            },
        ),
        // Catch-all covers feature-gated/future variants (e.g. economic_release).
        _ => ("Event".to_string(), String::new()),
    }
}

fn kind_str(event: &EventKind) -> &'static str {
    match event {
        EventKind::Earnings { .. } => "earnings",
        EventKind::ExDividend { .. } => "ex_dividend",
        EventKind::DividendPayment { .. } => "dividend_payment",
        EventKind::OptionsExpiration { .. } => "options_expiration",
        _ => "other",
    }
}

fn parse_range(s: &str) -> Result<TimeRange> {
    match s.to_lowercase().as_str() {
        "1d" => Ok(TimeRange::OneDay),
        "5d" | "1wk" => Ok(TimeRange::FiveDays),
        "1mo" => Ok(TimeRange::OneMonth),
        "3mo" => Ok(TimeRange::ThreeMonths),
        "6mo" => Ok(TimeRange::SixMonths),
        "1y" => Ok(TimeRange::OneYear),
        "2y" => Ok(TimeRange::TwoYears),
        "5y" => Ok(TimeRange::FiveYears),
        "10y" => Ok(TimeRange::TenYears),
        "ytd" => Ok(TimeRange::YearToDate),
        "max" => Ok(TimeRange::Max),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid range '{}'. Valid ranges: 1d, 5d/1wk, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max",
            s
        ))),
    }
}

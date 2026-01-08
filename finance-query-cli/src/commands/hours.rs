use crate::error::Result;
use crate::output;
use clap::Parser;
use finance_query::finance;

#[derive(Parser)]
pub struct HoursArgs {
    /// Region code (e.g., US, JP, GB). Defaults to US.
    #[arg(short, long)]
    region: Option<String>,
}

pub async fn execute(args: HoursArgs) -> Result<()> {
    let hours = finance::hours(args.region.as_deref()).await?;

    if hours.markets.is_empty() {
        output::print_info("No market hours data available");
        return Ok(());
    }

    output::print_success(&format!("Market Hours ({} markets)", hours.markets.len()));
    println!();

    // Header
    println!(
        "{:<15} {:<25} {:<10} {:<20} {:<20}",
        "Market", "Name", "Status", "Open", "Close"
    );
    println!("{}", "-".repeat(95));

    // Data rows
    for market in &hours.markets {
        let status_display = match market.status.as_str() {
            "open" => "🟢 OPEN".to_string(),
            "closed" => "🔴 CLOSED".to_string(),
            _ => market.status.clone(),
        };

        let open_time = market
            .open
            .as_ref()
            .and_then(|s| parse_time_short(s))
            .unwrap_or_else(|| "N/A".to_string());

        let close_time = market
            .close
            .as_ref()
            .and_then(|s| parse_time_short(s))
            .unwrap_or_else(|| "N/A".to_string());

        println!(
            "{:<15} {:<25} {:<10} {:<20} {:<20}",
            market.id,
            truncate(&market.name, 25),
            status_display,
            open_time,
            close_time
        );
    }

    Ok(())
}

/// Parse ISO 8601 time to human-readable format (time only)
fn parse_time_short(s: &str) -> Option<String> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.format("%H:%M %Z").to_string())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use colored::Colorize;
use finance_query::Ticker;
use serde::Serialize;

#[derive(Parser)]
pub struct HoldersArgs {
    /// Stock symbol to get holder information for
    #[arg(required = true)]
    symbol: String,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,
}

#[derive(Debug, Serialize)]
struct HoldersJson {
    symbol: String,
    insiders_percent: Option<f64>,
    institutions_percent: Option<f64>,
    institutions_float_percent: Option<f64>,
    institutions_count: Option<i64>,
    top_institutions: Vec<InstitutionJson>,
    top_insiders: Vec<InsiderJson>,
}

#[derive(Debug, Serialize)]
struct InstitutionJson {
    organization: String,
    shares: Option<i64>,
    value: Option<i64>,
    percent_held: Option<f64>,
    percent_change: Option<f64>,
}

#[derive(Debug, Serialize)]
struct InsiderJson {
    name: String,
    relation: Option<String>,
    shares_direct: Option<i64>,
    shares_indirect: Option<i64>,
    latest_transaction: Option<String>,
}

pub async fn execute(args: HoldersArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;
    let ticker = Ticker::new(&args.symbol).await?;

    // Fetch all holder data in parallel
    let (major_holders, institutions, insiders) = tokio::join!(
        ticker.major_holders(),
        ticker.institution_ownership(),
        ticker.insider_holders()
    );

    let major = major_holders.ok().flatten();
    let inst = institutions.ok().flatten();
    let insider = insiders.ok().flatten();

    // For JSON/CSV output
    if format != OutputFormat::Table {
        let holders_json = HoldersJson {
            symbol: args.symbol.clone(),
            insiders_percent: major
                .as_ref()
                .and_then(|m| m.insiders_percent_held.as_ref()?.raw),
            institutions_percent: major
                .as_ref()
                .and_then(|m| m.institutions_percent_held.as_ref()?.raw),
            institutions_float_percent: major
                .as_ref()
                .and_then(|m| m.institutions_float_percent_held.as_ref()?.raw),
            institutions_count: major
                .as_ref()
                .and_then(|m| m.institutions_count.as_ref()?.raw),
            top_institutions: inst
                .as_ref()
                .map(|i| {
                    i.ownership_list
                        .iter()
                        .take(10)
                        .map(|o| InstitutionJson {
                            organization: o.organization.clone().unwrap_or_default(),
                            shares: o.position.as_ref().and_then(|v| v.raw),
                            value: o.value.as_ref().and_then(|v| v.raw),
                            percent_held: o.pct_held.as_ref().and_then(|v| v.raw),
                            percent_change: o.pct_change.as_ref().and_then(|v| v.raw),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            top_insiders: insider
                .as_ref()
                .map(|i| {
                    i.holders
                        .iter()
                        .take(10)
                        .map(|h| InsiderJson {
                            name: h.name.clone().unwrap_or_default(),
                            relation: h.relation.clone(),
                            shares_direct: h.position_direct.as_ref().and_then(|v| v.raw),
                            shares_indirect: h.position_indirect.as_ref().and_then(|v| v.raw),
                            latest_transaction: h.transaction_description.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        };

        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&holders_json)?);
            }
            OutputFormat::Csv => {
                // CSV: flatten to key ownership breakdown
                println!("metric,value");
                println!(
                    "insiders_percent,{}",
                    holders_json
                        .insiders_percent
                        .map(|v| format!("{:.2}", v * 100.0))
                        .unwrap_or_else(|| "N/A".to_string())
                );
                println!(
                    "institutions_percent,{}",
                    holders_json
                        .institutions_percent
                        .map(|v| format!("{:.2}", v * 100.0))
                        .unwrap_or_else(|| "N/A".to_string())
                );
                println!(
                    "institutions_count,{}",
                    holders_json
                        .institutions_count
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "N/A".to_string())
                );
            }
            _ => {}
        }
        return Ok(());
    }

    // Table output
    output::print_success(&format!("Holders for {}", args.symbol.to_uppercase()));
    println!();

    // Major Holders Breakdown
    println!("{}", "Ownership Breakdown".blue().bold());
    println!("{}", "─".repeat(50));

    if let Some(m) = &major {
        print_row(
            "Insiders",
            m.insiders_percent_held
                .as_ref()
                .and_then(|v| v.raw)
                .map(|v| format!("{:.2}%", v * 100.0)),
        );
        print_row(
            "Institutions",
            m.institutions_percent_held
                .as_ref()
                .and_then(|v| v.raw)
                .map(|v| format!("{:.2}%", v * 100.0)),
        );
        print_row(
            "Institutions (Float)",
            m.institutions_float_percent_held
                .as_ref()
                .and_then(|v| v.raw)
                .map(|v| format!("{:.2}%", v * 100.0)),
        );
        print_row(
            "# of Institutions",
            m.institutions_count
                .as_ref()
                .and_then(|v| v.raw)
                .map(format_number),
        );
    } else {
        println!("  {}", "No ownership data available".dimmed());
    }

    // Top Institutional Holders
    println!();
    println!("{}", "Top Institutional Holders".blue().bold());
    println!("{}", "─".repeat(80));

    if let Some(inst) = &inst {
        if inst.ownership_list.is_empty() {
            println!("  {}", "No institutional holders found".dimmed());
        } else {
            println!(
                "{:<35} {:>12} {:>15} {:>10}",
                "Organization", "Shares", "Value", "% Held"
            );
            println!("{}", "─".repeat(80));

            for owner in inst.ownership_list.iter().take(10) {
                let org = owner
                    .organization
                    .as_ref()
                    .map(|s| truncate(s, 34))
                    .unwrap_or_else(|| "Unknown".to_string());
                let shares = owner
                    .position
                    .as_ref()
                    .and_then(|v| v.raw)
                    .map(format_number)
                    .unwrap_or_else(|| "N/A".to_string());
                let value = owner
                    .value
                    .as_ref()
                    .and_then(|v| v.raw)
                    .map(format_currency)
                    .unwrap_or_else(|| "N/A".to_string());
                let pct = owner
                    .pct_held
                    .as_ref()
                    .and_then(|v| v.raw)
                    .map(|v| format!("{:.2}%", v * 100.0))
                    .unwrap_or_else(|| "N/A".to_string());

                println!("{:<35} {:>12} {:>15} {:>10}", org, shares, value, pct);
            }
        }
    } else {
        println!("  {}", "No institutional holder data available".dimmed());
    }

    // Top Insider Holders
    println!();
    println!("{}", "Top Insider Holders".blue().bold());
    println!("{}", "─".repeat(80));

    if let Some(insider) = &insider {
        if insider.holders.is_empty() {
            println!("  {}", "No insider holders found".dimmed());
        } else {
            println!(
                "{:<25} {:<20} {:>15} {:>15}",
                "Name", "Relation", "Direct", "Indirect"
            );
            println!("{}", "─".repeat(80));

            for holder in insider.holders.iter().take(10) {
                let name = holder
                    .name
                    .as_ref()
                    .map(|s| truncate(s, 24))
                    .unwrap_or_else(|| "Unknown".to_string());
                let relation = holder
                    .relation
                    .as_ref()
                    .map(|s| truncate(s, 19))
                    .unwrap_or_else(|| "N/A".to_string());
                let direct = holder
                    .position_direct
                    .as_ref()
                    .and_then(|v| v.raw)
                    .map(format_number)
                    .unwrap_or_else(|| "N/A".to_string());
                let indirect = holder
                    .position_indirect
                    .as_ref()
                    .and_then(|v| v.raw)
                    .map(format_number)
                    .unwrap_or_else(|| "-".to_string());

                println!(
                    "{:<25} {:<20} {:>15} {:>15}",
                    name, relation, direct, indirect
                );
            }
        }
    } else {
        println!("  {}", "No insider holder data available".dimmed());
    }

    Ok(())
}

fn print_row(label: &str, value: Option<String>) {
    let val = value.unwrap_or_else(|| "N/A".to_string());
    println!("  {:<25} {}", label, val);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

fn format_number(n: i64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_currency(n: i64) -> String {
    if n >= 1_000_000_000 {
        format!("${:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("${:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("${:.1}K", n as f64 / 1_000.0)
    } else {
        format!("${}", n)
    }
}

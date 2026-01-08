use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use colored::Colorize;
use finance_query::Ticker;
use serde::Serialize;

#[derive(Parser)]
pub struct ProfileArgs {
    /// Stock symbol to get profile for
    #[arg(required = true)]
    symbol: String,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,
}

#[derive(Debug, Serialize)]
struct ProfileJson {
    symbol: String,
    sector: Option<String>,
    industry: Option<String>,
    employees: Option<i64>,
    website: Option<String>,
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    phone: Option<String>,
    description: Option<String>,
    governance: Option<GovernanceJson>,
    executives: Vec<ExecutiveJson>,
}

#[derive(Debug, Serialize)]
struct GovernanceJson {
    audit_risk: Option<i32>,
    board_risk: Option<i32>,
    compensation_risk: Option<i32>,
    shareholder_rights_risk: Option<i32>,
    overall_risk: Option<i32>,
}

#[derive(Debug, Serialize)]
struct ExecutiveJson {
    name: String,
    title: Option<String>,
    age: Option<i32>,
    total_pay: Option<i64>,
}

pub async fn execute(args: ProfileArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;
    let ticker = Ticker::new(&args.symbol).await?;

    let profile = ticker.asset_profile().await?;

    if profile.is_none() {
        output::print_info(&format!("No profile data available for {}", args.symbol));
        return Ok(());
    }

    let p = profile.unwrap();

    // For JSON/CSV output
    if format != OutputFormat::Table {
        let profile_json = ProfileJson {
            symbol: args.symbol.clone(),
            sector: p.sector.clone(),
            industry: p.industry.clone(),
            employees: p.full_time_employees,
            website: p.website.clone(),
            address: p.address1.clone(),
            city: p.city.clone(),
            state: p.state.clone(),
            country: p.country.clone(),
            phone: p.phone.clone(),
            description: p.long_business_summary.clone(),
            governance: Some(GovernanceJson {
                audit_risk: p.audit_risk,
                board_risk: p.board_risk,
                compensation_risk: p.compensation_risk,
                shareholder_rights_risk: p.shareholder_rights_risk,
                overall_risk: p.overall_risk,
            }),
            executives: p
                .company_officers
                .iter()
                .map(|o| ExecutiveJson {
                    name: o.name.clone().unwrap_or_default(),
                    title: o.title.clone(),
                    age: o.age,
                    total_pay: o.total_pay.as_ref().and_then(|v| v.raw),
                })
                .collect(),
        };

        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&profile_json)?);
            }
            OutputFormat::Csv => {
                println!("field,value");
                println!("symbol,{}", args.symbol);
                println!("sector,{}", profile_json.sector.as_deref().unwrap_or("N/A"));
                println!(
                    "industry,{}",
                    profile_json.industry.as_deref().unwrap_or("N/A")
                );
                println!(
                    "employees,{}",
                    profile_json
                        .employees
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "N/A".to_string())
                );
                println!(
                    "website,{}",
                    profile_json.website.as_deref().unwrap_or("N/A")
                );
                println!(
                    "country,{}",
                    profile_json.country.as_deref().unwrap_or("N/A")
                );
            }
            _ => {}
        }
        return Ok(());
    }

    // Table output
    output::print_success(&format!("Company Profile: {}", args.symbol.to_uppercase()));
    println!();

    // Basic Info
    println!("{}", "Company Information".blue().bold());
    println!("{}", "─".repeat(60));

    print_row("Sector", p.sector.as_deref());
    print_row("Industry", p.industry.as_deref());
    print_row(
        "Employees",
        p.full_time_employees.map(format_number).as_deref(),
    );
    print_row("Website", p.website.as_deref());

    // Address
    let address = build_address(p.city.as_ref(), p.state.as_ref(), p.country.as_ref());
    if !address.is_empty() {
        print_row("Location", Some(&address));
    }
    print_row("Phone", p.phone.as_deref());

    // Governance Risk Scores
    if p.overall_risk.is_some() {
        println!();
        println!("{}", "Governance Risk Scores".blue().bold());
        println!("{}", "─".repeat(60));
        println!(
            "  {} (1=low risk, 10=high risk)",
            "Lower is better".dimmed()
        );
        println!();

        print_risk_row("Overall Risk", p.overall_risk);
        print_risk_row("Audit Risk", p.audit_risk);
        print_risk_row("Board Risk", p.board_risk);
        print_risk_row("Compensation Risk", p.compensation_risk);
        print_risk_row("Shareholder Rights", p.shareholder_rights_risk);
    }

    // Business Summary
    if let Some(summary) = &p.long_business_summary {
        println!();
        println!("{}", "Business Summary".blue().bold());
        println!("{}", "─".repeat(60));

        // Word wrap the summary
        let wrapped = textwrap(summary, 58);
        for line in wrapped {
            println!("  {}", line);
        }
    }

    // Executives
    if !p.company_officers.is_empty() {
        println!();
        println!("{}", "Key Executives".blue().bold());
        println!("{}", "─".repeat(80));
        println!(
            "{:<30} {:<30} {:>8} {:>12}",
            "Name", "Title", "Age", "Total Pay"
        );
        println!("{}", "─".repeat(80));

        for officer in p.company_officers.iter().take(10) {
            let name = officer
                .name
                .as_ref()
                .map(|s| truncate(s, 29))
                .unwrap_or_else(|| "N/A".to_string());
            let title = officer
                .title
                .as_ref()
                .map(|s| truncate(s, 29))
                .unwrap_or_else(|| "N/A".to_string());
            let age = officer
                .age
                .map(|a| a.to_string())
                .unwrap_or_else(|| "-".to_string());
            let pay = officer
                .total_pay
                .as_ref()
                .and_then(|v| v.raw)
                .map(format_currency)
                .unwrap_or_else(|| "-".to_string());

            println!("{:<30} {:<30} {:>8} {:>12}", name, title, age, pay);
        }
    }

    Ok(())
}

fn print_row(label: &str, value: Option<&str>) {
    let val = value.unwrap_or("N/A");
    println!("  {:<20} {}", label, val);
}

fn print_risk_row(label: &str, value: Option<i32>) {
    let val = value
        .map(|v| {
            let color = match v {
                1..=3 => "green",
                4..=6 => "yellow",
                _ => "red",
            };
            let bar = "█".repeat(v as usize);
            let empty = "░".repeat(10 - v as usize);
            match color {
                "green" => format!("{}{} {}/10", bar.green(), empty.dimmed(), v),
                "yellow" => format!("{}{} {}/10", bar.yellow(), empty.dimmed(), v),
                _ => format!("{}{} {}/10", bar.red(), empty.dimmed(), v),
            }
        })
        .unwrap_or_else(|| "N/A".to_string());
    println!("  {:<20} {}", label, val);
}

fn build_address(
    city: Option<&String>,
    state: Option<&String>,
    country: Option<&String>,
) -> String {
    let mut parts = Vec::new();
    if let Some(c) = city {
        parts.push(c.clone());
    }
    if let Some(s) = state {
        parts.push(s.clone());
    }
    if let Some(c) = country {
        parts.push(c.clone());
    }
    parts.join(", ")
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

fn format_number(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_currency(n: i64) -> String {
    if n >= 1_000_000 {
        format!("${:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("${:.1}K", n as f64 / 1_000.0)
    } else {
        format!("${}", n)
    }
}

fn textwrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

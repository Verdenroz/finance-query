use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use colored::Colorize;
use finance_query::SectorType;
use serde::Serialize;
use tabled::Tabled;

#[derive(Parser)]
pub struct SectorArgs {
    /// Sector to get data for
    #[arg(value_enum)]
    sector: SectorTypeArg,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,

    /// Show top companies in the sector
    #[arg(long)]
    companies: bool,

    /// Show industries within the sector
    #[arg(long)]
    industries: bool,

    /// Show all details (companies, industries, ETFs, mutual funds)
    #[arg(long)]
    all: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum SectorTypeArg {
    Technology,
    FinancialServices,
    ConsumerCyclical,
    CommunicationServices,
    Healthcare,
    Industrials,
    ConsumerDefensive,
    Energy,
    BasicMaterials,
    RealEstate,
    Utilities,
}

impl From<SectorTypeArg> for SectorType {
    fn from(arg: SectorTypeArg) -> Self {
        match arg {
            SectorTypeArg::Technology => SectorType::Technology,
            SectorTypeArg::FinancialServices => SectorType::FinancialServices,
            SectorTypeArg::ConsumerCyclical => SectorType::ConsumerCyclical,
            SectorTypeArg::CommunicationServices => SectorType::CommunicationServices,
            SectorTypeArg::Healthcare => SectorType::Healthcare,
            SectorTypeArg::Industrials => SectorType::Industrials,
            SectorTypeArg::ConsumerDefensive => SectorType::ConsumerDefensive,
            SectorTypeArg::Energy => SectorType::Energy,
            SectorTypeArg::BasicMaterials => SectorType::BasicMaterials,
            SectorTypeArg::RealEstate => SectorType::RealEstate,
            SectorTypeArg::Utilities => SectorType::Utilities,
        }
    }
}

#[derive(Debug, Serialize, Tabled)]
struct CompanyDisplay {
    #[tabled(rename = "Symbol")]
    symbol: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Day %")]
    day_change: String,
    #[tabled(rename = "YTD %")]
    ytd_return: String,
    #[tabled(rename = "Market Cap")]
    market_cap: String,
    #[tabled(rename = "Rating")]
    rating: String,
}

#[derive(Debug, Serialize, Tabled)]
struct IndustryDisplay {
    #[tabled(rename = "Industry")]
    name: String,
    #[tabled(rename = "Weight %")]
    market_weight: String,
    #[tabled(rename = "Day %")]
    day_change: String,
    #[tabled(rename = "YTD %")]
    ytd_return: String,
}

pub async fn execute(args: SectorArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;
    let sector_type: SectorType = args.sector.into();

    // Fetch sector data
    let sector = finance_query::finance::sector(sector_type).await?;

    // For JSON output, return the full sector data
    if format == OutputFormat::Json {
        let json = serde_json::to_string_pretty(&sector)?;
        println!("{}", json);
        return Ok(());
    }

    // For CSV output with companies
    if format == OutputFormat::Csv {
        let companies: Vec<CompanyDisplay> = sector
            .top_companies
            .iter()
            .map(|c| CompanyDisplay {
                symbol: c.symbol.clone(),
                name: c.name.clone().unwrap_or_else(|| "N/A".to_string()),
                price: c
                    .last_price
                    .as_ref()
                    .and_then(|v| v.raw.map(|r| format!("${:.2}", r)))
                    .unwrap_or_else(|| "N/A".to_string()),
                day_change: c
                    .day_change_percent
                    .as_ref()
                    .and_then(|v| v.raw.map(|r| format!("{:+.2}%", r * 100.0)))
                    .unwrap_or_else(|| "N/A".to_string()),
                ytd_return: c
                    .ytd_return
                    .as_ref()
                    .and_then(|v| v.raw.map(|r| format!("{:+.2}%", r * 100.0)))
                    .unwrap_or_else(|| "N/A".to_string()),
                market_cap: c
                    .market_cap
                    .as_ref()
                    .and_then(|v| v.raw.map(format_market_cap))
                    .unwrap_or_else(|| "N/A".to_string()),
                rating: c.rating.clone().unwrap_or_else(|| "N/A".to_string()),
            })
            .collect();
        output::print_many(&companies, OutputFormat::Csv)?;
        return Ok(());
    }

    // Table output - show summary with optional details
    println!("{}", "━".repeat(80).blue());
    println!("{} {}", "Sector:".bold(), sector.name.green());
    if let Some(ref symbol) = sector.symbol {
        println!("{} {}", "Symbol:".bold(), symbol);
    }
    println!("{}", "━".repeat(80).blue());

    // Overview
    if let Some(ref overview) = sector.overview {
        println!("\n{}", "Overview".bold().underline());
        if let Some(desc) = &overview.description {
            println!("{}", desc);
        }
        if let Some(companies) = overview.companies_count {
            println!("  {} {}", "Companies:".bold(), companies);
        }
        if let Some(industries) = overview.industries_count {
            println!("  {} {}", "Industries:".bold(), industries);
        }
        if let Some(ref market_cap) = overview.market_cap
            && let Some(val) = market_cap.raw
        {
            println!(
                "  {} ${:.2}T",
                "Market Cap:".bold(),
                val / 1_000_000_000_000.0
            );
        }
        if let Some(ref market_weight) = overview.market_weight
            && let Some(val) = market_weight.raw
        {
            // Raw value is decimal fraction, multiply by 100 for percentage
            println!("  {} {:.2}%", "Market Weight:".bold(), val * 100.0);
        }
    }

    // Performance
    if let Some(ref performance) = sector.performance {
        println!("\n{}", "Performance".bold().underline());
        if let Some(ref day) = performance.day_change_percent
            && let Some(val) = day.raw
        {
            let formatted = format_change_percent(val);
            println!("  {} {}", "Day:".bold(), formatted);
        }
        if let Some(ref ytd) = performance.ytd_change_percent
            && let Some(val) = ytd.raw
        {
            let formatted = format_change_percent(val);
            println!("  {} {}", "YTD:".bold(), formatted);
        }
        if let Some(ref one_year) = performance.one_year_change_percent
            && let Some(val) = one_year.raw
        {
            let formatted = format_change_percent(val);
            println!("  {} {}", "1 Year:".bold(), formatted);
        }
        if let Some(ref three_year) = performance.three_year_change_percent
            && let Some(val) = three_year.raw
        {
            let formatted = format_change_percent(val);
            println!("  {} {}", "3 Year:".bold(), formatted);
        }
        if let Some(ref five_year) = performance.five_year_change_percent
            && let Some(val) = five_year.raw
        {
            let formatted = format_change_percent(val);
            println!("  {} {}", "5 Year:".bold(), formatted);
        }
    }

    // Benchmark comparison
    if let (Some(benchmark), Some(name)) = (&sector.benchmark, &sector.benchmark_name) {
        println!(
            "\n{} ({})",
            "Benchmark Performance".bold().underline(),
            name
        );
        if let Some(ref ytd) = benchmark.ytd_change_percent
            && let Some(val) = ytd.raw
        {
            let formatted = format_change_percent(val);
            println!("  {} {}", "YTD:".bold(), formatted);
        }
        if let Some(ref one_year) = benchmark.one_year_change_percent
            && let Some(val) = one_year.raw
        {
            let formatted = format_change_percent(val);
            println!("  {} {}", "1 Year:".bold(), formatted);
        }
    }

    // Top companies (show by default or with --companies flag)
    if (args.companies || args.all || !args.industries) && !sector.top_companies.is_empty() {
        println!("\n{}", "Top Companies".bold().underline());
        let companies: Vec<CompanyDisplay> = sector
            .top_companies
            .iter()
            .map(|c| CompanyDisplay {
                symbol: c.symbol.clone(),
                name: c.name.clone().unwrap_or_else(|| "N/A".to_string()),
                price: c
                    .last_price
                    .as_ref()
                    .and_then(|v| v.raw.map(|r| format!("${:.2}", r)))
                    .unwrap_or_else(|| "N/A".to_string()),
                day_change: c
                    .day_change_percent
                    .as_ref()
                    .and_then(|v| v.raw.map(|r| format!("{:+.2}%", r * 100.0)))
                    .unwrap_or_else(|| "N/A".to_string()),
                ytd_return: c
                    .ytd_return
                    .as_ref()
                    .and_then(|v| v.raw.map(|r| format!("{:+.2}%", r * 100.0)))
                    .unwrap_or_else(|| "N/A".to_string()),
                market_cap: c
                    .market_cap
                    .as_ref()
                    .and_then(|v| v.raw.map(format_market_cap))
                    .unwrap_or_else(|| "N/A".to_string()),
                rating: c.rating.clone().unwrap_or_else(|| "N/A".to_string()),
            })
            .collect();
        output::print_many(&companies, OutputFormat::Table)?;
    }

    // Industries
    if (args.industries || args.all) && !sector.industries.is_empty() {
        println!("\n{}", "Industries".bold().underline());
        let industries: Vec<IndustryDisplay> = sector
            .industries
            .iter()
            .map(|i| IndustryDisplay {
                name: i.name.clone(),
                market_weight: i
                    .market_weight
                    .as_ref()
                    .and_then(|v| v.raw.map(|r| format!("{:.2}%", r * 100.0)))
                    .unwrap_or_else(|| "N/A".to_string()),
                day_change: i
                    .day_change_percent
                    .as_ref()
                    .and_then(|v| v.raw.map(|r| format!("{:+.2}%", r * 100.0)))
                    .unwrap_or_else(|| "N/A".to_string()),
                ytd_return: i
                    .ytd_return
                    .as_ref()
                    .and_then(|v| v.raw.map(|r| format!("{:+.2}%", r * 100.0)))
                    .unwrap_or_else(|| "N/A".to_string()),
            })
            .collect();
        output::print_many(&industries, OutputFormat::Table)?;
    }

    println!();
    Ok(())
}

fn format_change_percent(val: f64) -> String {
    // Raw values from Yahoo are decimal fractions (0.01 = 1%), multiply by 100
    let pct = val * 100.0;
    let formatted = if pct >= 0.0 {
        format!("+{:.2}%", pct).green()
    } else {
        format!("{:.2}%", pct).red()
    };
    formatted.to_string()
}

fn format_market_cap(market_cap: f64) -> String {
    match market_cap as i64 {
        v if v >= 1_000_000_000_000 => format!("{:.2}T", market_cap / 1_000_000_000_000.0),
        v if v >= 1_000_000_000 => format!("{:.2}B", market_cap / 1_000_000_000.0),
        v if v >= 1_000_000 => format!("{:.2}M", market_cap / 1_000_000.0),
        _ => market_cap.to_string(),
    }
}

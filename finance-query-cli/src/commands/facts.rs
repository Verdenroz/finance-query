use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use colored::Colorize;
use finance_query::{CompanyFacts, EdgarClientBuilder};
use serde::Serialize;
use tabled::Tabled;

#[derive(Parser)]
pub struct FactsArgs {
    /// Stock symbol to get XBRL facts for
    #[arg(required = true)]
    symbol: String,

    /// Email address for SEC EDGAR User-Agent (required by SEC)
    #[arg(short, long, env = "EDGAR_EMAIL")]
    email: Option<String>,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,

    /// Specific concept to display (e.g., "Revenue", "Assets", "NetIncomeLoss")
    #[arg(short, long)]
    concept: Option<String>,

    /// Unit to filter by (e.g., "USD", "shares")
    #[arg(short, long)]
    unit: Option<String>,

    /// Maximum number of data points to show per concept
    #[arg(short, long, default_value = "10")]
    limit: usize,
}

#[derive(Debug, Serialize, Tabled)]
struct FactRow {
    concept: String,
    fiscal_year: String,
    fiscal_period: String,
    value: String,
    unit: String,
    form: String,
    filed: String,
}

pub async fn execute(args: FactsArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;

    let email = args.email.ok_or_else(|| {
        crate::error::CliError::InvalidArgument(
            "EDGAR email required. Provide via --email or set EDGAR_EMAIL environment variable"
                .to_string(),
        )
    })?;

    let edgar_client = EdgarClientBuilder::new(email)
        .app_name("finance-query-cli")
        .build()?;

    // Resolve symbol to CIK
    let cik = edgar_client.resolve_cik(&args.symbol).await?;

    // Fetch company facts
    let facts = edgar_client.company_facts(cik).await?;

    if format == OutputFormat::Json {
        println!("{}", serde_json::to_string_pretty(&facts)?);
        return Ok(());
    }

    // Extract and display key facts
    let rows = extract_fact_rows(
        &facts,
        args.concept.as_deref(),
        args.unit.as_deref(),
        args.limit,
    );

    if rows.is_empty() {
        let msg = if args.concept.is_some() {
            format!(
                "No XBRL facts found for concept '{}' in {}",
                args.concept.unwrap(),
                args.symbol
            )
        } else {
            format!("No XBRL facts found for {}", args.symbol)
        };
        output::print_info(&msg);
        return Ok(());
    }

    match format {
        OutputFormat::Table => {
            println!(
                "\n{}",
                format!("EDGAR XBRL Facts: {}", args.symbol.to_uppercase())
                    .cyan()
                    .bold()
            );
            println!("{}", format!("CIK: {}", cik).dimmed());
            if let Some(name) = &facts.entity_name {
                println!("{}\n", name.dimmed());
            }
            output::print_many(&rows, format)?;
        }
        OutputFormat::Csv => {
            output::print_many(&rows, format)?;
        }
        _ => {}
    }

    Ok(())
}

fn extract_fact_rows(
    facts: &CompanyFacts,
    concept_filter: Option<&str>,
    unit_filter: Option<&str>,
    limit: usize,
) -> Vec<FactRow> {
    let mut rows = Vec::new();

    // Get US-GAAP facts (most common)
    let Some(gaap) = facts.us_gaap() else {
        return rows;
    };

    // Common financial concepts to display if no filter specified
    let default_concepts = [
        "Revenue",
        "GrossProfit",
        "OperatingIncomeLoss",
        "NetIncomeLoss",
        "EarningsPerShareBasic",
        "Assets",
        "Liabilities",
        "StockholdersEquity",
        "CashAndCashEquivalentsAtCarryingValue",
        "RetainedEarningsAccumulatedDeficit",
    ];

    let concepts_to_check: Vec<&str> = if let Some(concept) = concept_filter {
        vec![concept]
    } else {
        default_concepts.to_vec()
    };

    for concept_name in concepts_to_check {
        if let Some(concept) = gaap.0.get(concept_name) {
            // Iterate over units
            for (unit_name, data_points) in &concept.units {
                // Apply unit filter if specified
                if let Some(filter_unit) = unit_filter
                    && unit_name != filter_unit
                {
                    continue;
                }

                // Take most recent data points up to limit
                let mut sorted_points = data_points.clone();
                sorted_points.sort_by(|a, b| {
                    // Sort by fiscal year descending, then by fiscal period
                    b.fy.cmp(&a.fy).then_with(|| b.fp.cmp(&a.fp))
                });

                for point in sorted_points.iter().take(limit) {
                    rows.push(FactRow {
                        concept: concept_name.to_string(),
                        fiscal_year: point
                            .fy
                            .map(|y| y.to_string())
                            .unwrap_or_else(|| "N/A".to_string()),
                        fiscal_period: point.fp.clone().unwrap_or_else(|| "N/A".to_string()),
                        value: point
                            .val
                            .map(format_value)
                            .unwrap_or_else(|| "N/A".to_string()),
                        unit: unit_name.clone(),
                        form: point.form.clone().unwrap_or_else(|| "N/A".to_string()),
                        filed: point.filed.clone().unwrap_or_else(|| "N/A".to_string()),
                    });
                }
            }
        }
    }

    rows
}

fn format_value(val: f64) -> String {
    if val.abs() >= 1_000_000_000.0 {
        format!("{:.2}B", val / 1_000_000_000.0)
    } else if val.abs() >= 1_000_000.0 {
        format!("{:.2}M", val / 1_000_000.0)
    } else if val.abs() >= 1_000.0 {
        format!("{:.2}K", val / 1_000.0)
    } else {
        format!("{:.2}", val)
    }
}

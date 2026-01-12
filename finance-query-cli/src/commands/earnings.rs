use crate::error::Result;
use crate::output::{self, OutputFormat};
use chrono::{DateTime, Utc};
use clap::Parser;
use colored::Colorize;
use finance_query::Ticker;
use serde::Serialize;

#[derive(Parser)]
pub struct EarningsArgs {
    /// Stock symbol to get earnings for
    #[arg(required = true)]
    symbol: String,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,
}

#[derive(Debug, Serialize)]
struct EarningsJson {
    symbol: String,
    next_earnings_date: Option<String>,
    earnings_estimate: Option<f64>,
    revenue_estimate: Option<i64>,
    ex_dividend_date: Option<String>,
    dividend_date: Option<String>,
    quarterly_history: Vec<QuarterJson>,
}

#[derive(Debug, Serialize)]
struct QuarterJson {
    quarter: String,
    actual_eps: Option<f64>,
    estimate_eps: Option<f64>,
    surprise_percent: Option<f64>,
}

pub async fn execute(args: EarningsArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;
    let ticker = Ticker::new(&args.symbol).await?;

    // Fetch earnings data
    let (calendar, earnings, history) = tokio::join!(
        ticker.calendar_events(),
        ticker.earnings(),
        ticker.earnings_history()
    );

    let cal = calendar.ok().flatten();
    let earn = earnings.ok().flatten();
    let hist = history.ok().flatten();

    // For JSON/CSV output
    if format != OutputFormat::Table {
        let earnings_json = EarningsJson {
            symbol: args.symbol.clone(),
            next_earnings_date: cal
                .as_ref()
                .and_then(|c| c.next_earnings_date())
                .and_then(format_date),
            earnings_estimate: cal.as_ref().and_then(|c| c.earnings_estimate()),
            revenue_estimate: cal
                .as_ref()
                .and_then(|c| c.earnings.as_ref()?.revenue_average.as_ref()?.raw),
            ex_dividend_date: cal
                .as_ref()
                .and_then(|c| c.ex_dividend_timestamp())
                .and_then(format_date),
            dividend_date: cal
                .as_ref()
                .and_then(|c| c.dividend_timestamp())
                .and_then(format_date),
            quarterly_history: hist
                .as_ref()
                .map(|h| {
                    h.history
                        .iter()
                        .map(|e| QuarterJson {
                            quarter: e.period.clone().unwrap_or_else(|| "N/A".to_string()),
                            actual_eps: e.eps_actual.as_ref().and_then(|v| v.raw),
                            estimate_eps: e.eps_estimate.as_ref().and_then(|v| v.raw),
                            surprise_percent: e.surprise_percent.as_ref().and_then(|v| v.raw),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        };

        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&earnings_json)?);
            }
            OutputFormat::Csv => {
                println!("quarter,actual_eps,estimate_eps,surprise_percent");
                for q in &earnings_json.quarterly_history {
                    println!(
                        "{},{},{},{}",
                        q.quarter,
                        q.actual_eps
                            .map(|v| format!("{:.2}", v))
                            .unwrap_or_default(),
                        q.estimate_eps
                            .map(|v| format!("{:.2}", v))
                            .unwrap_or_default(),
                        q.surprise_percent
                            .map(|v| format!("{:.2}", v))
                            .unwrap_or_default(),
                    );
                }
            }
            _ => {}
        }
        return Ok(());
    }

    // Table output
    output::print_success(&format!("Earnings for {}", args.symbol.to_uppercase()));
    println!();

    // Upcoming Events
    println!("{}", "Upcoming Events".blue().bold());
    println!("{}", "─".repeat(50));

    if let Some(c) = &cal {
        // Next earnings date
        if let Some(ts) = c.next_earnings_date() {
            let date_str = format_date(ts).unwrap_or_else(|| "N/A".to_string());
            let is_estimate = c
                .earnings
                .as_ref()
                .and_then(|e| {
                    e.earnings_date
                        .as_ref()?
                        .first()?
                        .fmt
                        .as_ref()
                        .map(|_| true)
                })
                .unwrap_or(false);
            let label = if is_estimate {
                "Next Earnings (Est)"
            } else {
                "Next Earnings"
            };
            print_row(label, Some(date_str));
        }

        // Earnings estimate
        if let Some(est) = c.earnings_estimate() {
            print_row("EPS Estimate", Some(format!("${:.2}", est)));
        }

        // Revenue estimate
        if let Some(rev) = c
            .earnings
            .as_ref()
            .and_then(|e| e.revenue_average.as_ref()?.raw)
        {
            print_row("Revenue Estimate", Some(format_currency(rev)));
        }

        // Dividend dates
        if let Some(ts) = c.ex_dividend_timestamp() {
            print_row("Ex-Dividend Date", format_date(ts));
        }
        if let Some(ts) = c.dividend_timestamp() {
            print_row("Dividend Date", format_date(ts));
        }
    } else {
        println!("  {}", "No calendar events available".dimmed());
    }

    // Earnings History
    println!();
    println!("{}", "Earnings History".blue().bold());
    println!("{}", "─".repeat(70));

    if let Some(h) = &hist {
        if h.history.is_empty() {
            println!("  {}", "No earnings history available".dimmed());
        } else {
            println!(
                "{:<12} {:>12} {:>12} {:>12} {:>15}",
                "Quarter", "Actual", "Estimate", "Diff", "Surprise"
            );
            println!("{}", "─".repeat(70));

            for entry in h.history.iter().take(8) {
                let quarter = entry.period.clone().unwrap_or_else(|| "N/A".to_string());
                let actual = entry
                    .eps_actual
                    .as_ref()
                    .and_then(|v| v.raw)
                    .map(|v| format!("${:.2}", v))
                    .unwrap_or_else(|| "N/A".to_string());
                let estimate = entry
                    .eps_estimate
                    .as_ref()
                    .and_then(|v| v.raw)
                    .map(|v| format!("${:.2}", v))
                    .unwrap_or_else(|| "N/A".to_string());
                let diff = entry
                    .eps_difference
                    .as_ref()
                    .and_then(|v| v.raw)
                    .map(|v| {
                        if v >= 0.0 {
                            format!("+${:.2}", v).green().to_string()
                        } else {
                            format!("-${:.2}", v.abs()).red().to_string()
                        }
                    })
                    .unwrap_or_else(|| "N/A".to_string());
                let surprise = entry
                    .surprise_percent
                    .as_ref()
                    .and_then(|v| v.raw)
                    .map(|v| {
                        if v >= 0.0 {
                            format!("+{:.1}%", v * 100.0).green().to_string()
                        } else {
                            format!("{:.1}%", v * 100.0).red().to_string()
                        }
                    })
                    .unwrap_or_else(|| "N/A".to_string());

                println!(
                    "{:<12} {:>12} {:>12} {:>12} {:>15}",
                    quarter, actual, estimate, diff, surprise
                );
            }
        }
    } else {
        println!("  {}", "No earnings history available".dimmed());
    }

    // Quarterly Earnings Chart (from earnings module)
    if let Some(e) = &earn
        && let Some(chart) = &e.earnings_chart
        && !chart.quarterly.is_empty()
    {
        println!();
        println!("{}", "Recent Quarterly Results".blue().bold());
        println!("{}", "─".repeat(50));

        for q in chart.quarterly.iter().take(4) {
            let date = q.date.clone().unwrap_or_else(|| "N/A".to_string());
            let actual = q
                .actual
                .as_ref()
                .and_then(|v| v.raw)
                .map(|v| format!("${:.2}", v))
                .unwrap_or_else(|| "N/A".to_string());
            let estimate = q
                .estimate
                .as_ref()
                .and_then(|v| v.raw)
                .map(|v| format!("${:.2}", v))
                .unwrap_or_else(|| "N/A".to_string());

            println!("  {} - Actual: {}, Est: {}", date, actual, estimate);
        }
    }

    Ok(())
}

fn print_row(label: &str, value: Option<String>) {
    let val = value.unwrap_or_else(|| "N/A".to_string());
    println!("  {:<25} {}", label, val);
}

fn format_date(timestamp: i64) -> Option<String> {
    DateTime::<Utc>::from_timestamp(timestamp, 0).map(|dt| dt.format("%Y-%m-%d").to_string())
}

fn format_currency(n: i64) -> String {
    if n >= 1_000_000_000 {
        format!("${:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("${:.2}M", n as f64 / 1_000_000.0)
    } else {
        format!("${}", n)
    }
}

use crate::error::Result;
use crate::output::{self, OutputFormat};
use clap::Parser;
use colored::Colorize;
use finance_query::Ticker;
use serde::Serialize;

#[derive(Parser)]
pub struct GradesArgs {
    /// Stock symbol to get analyst grades for
    #[arg(required = true)]
    symbol: String,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,

    /// Filter by action type (up, down, init, main, reit)
    #[arg(short, long)]
    action: Option<String>,

    /// Maximum number of grades to show
    #[arg(short, long, default_value = "20")]
    limit: usize,
}

#[derive(Debug, Serialize)]
struct GradeJson {
    symbol: String,
    date: Option<String>,
    firm: Option<String>,
    action: Option<String>,
    from_grade: Option<String>,
    to_grade: Option<String>,
    prior_price_target: Option<f64>,
    current_price_target: Option<f64>,
    price_target_action: Option<String>,
}

pub async fn execute(args: GradesArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;
    let ticker = Ticker::new(&args.symbol).await?;

    let grading_data = ticker.grading_history().await?;

    if grading_data.is_none() {
        output::print_info(&format!(
            "No analyst grade history available for {}",
            args.symbol
        ));
        return Ok(());
    }

    let history = grading_data.unwrap();
    let mut grades = history.history;

    // Filter by action if specified
    if let Some(ref filter_action) = args.action {
        let filter_lower = filter_action.to_lowercase();
        grades.retain(|g| {
            g.action
                .as_ref()
                .map(|a| a.to_lowercase().contains(&filter_lower))
                .unwrap_or(false)
        });
    }

    // Sort by date ascending (oldest first, most recent at bottom)
    grades.sort_by_key(|g| g.epoch_grade_date.unwrap_or(0));

    // Take the last N items (most recent)
    if grades.len() > args.limit {
        grades = grades.split_off(grades.len() - args.limit);
    }

    if grades.is_empty() {
        let msg = if args.action.is_some() {
            format!(
                "No {} grade changes found for {}",
                args.action.as_ref().unwrap(),
                args.symbol
            )
        } else {
            format!("No analyst grade history found for {}", args.symbol)
        };
        output::print_info(&msg);
        return Ok(());
    }

    // For JSON/CSV output
    if format != OutputFormat::Table {
        let grades_json: Vec<GradeJson> = grades
            .iter()
            .map(|g| {
                let date = g.epoch_grade_date.and_then(|ts| {
                    chrono::DateTime::from_timestamp(ts, 0)
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                });
                GradeJson {
                    symbol: args.symbol.clone(),
                    date,
                    firm: g.firm.clone(),
                    action: g.action.clone(),
                    from_grade: g.from_grade.clone(),
                    to_grade: g.to_grade.clone(),
                    prior_price_target: g.prior_price_target,
                    current_price_target: g.current_price_target,
                    price_target_action: g.price_target_action.clone(),
                }
            })
            .collect();

        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&grades_json)?);
            }
            OutputFormat::Csv => {
                println!(
                    "symbol,date,firm,action,from_grade,to_grade,prior_pt,current_pt,pt_action"
                );
                for grade in &grades_json {
                    println!(
                        "{},{},{},{},{},{},{},{},{}",
                        grade.symbol,
                        grade.date.as_deref().unwrap_or("N/A"),
                        escape_csv(grade.firm.as_deref().unwrap_or("N/A")),
                        grade.action.as_deref().unwrap_or("N/A"),
                        grade.from_grade.as_deref().unwrap_or("N/A"),
                        grade.to_grade.as_deref().unwrap_or("N/A"),
                        grade
                            .prior_price_target
                            .map(|v| format!("{:.2}", v))
                            .unwrap_or_else(|| "N/A".to_string()),
                        grade
                            .current_price_target
                            .map(|v| format!("{:.2}", v))
                            .unwrap_or_else(|| "N/A".to_string()),
                        grade.price_target_action.as_deref().unwrap_or("N/A")
                    );
                }
            }
            _ => {}
        }
        return Ok(());
    }

    // Table output
    output::print_success(&format!(
        "Analyst Grades for {} ({} changes)",
        args.symbol.to_uppercase(),
        grades.len()
    ));
    println!();

    println!(
        "{:<12} {:<20} {:<8} {:<12} {:<12} {:<15}",
        "Date".blue().bold(),
        "Firm".blue().bold(),
        "Action".blue().bold(),
        "From".blue().bold(),
        "To".blue().bold(),
        "Price Target".blue().bold()
    );
    println!("{}", "─".repeat(85));

    for grade in &grades {
        let date = grade
            .epoch_grade_date
            .and_then(|ts| {
                chrono::DateTime::from_timestamp(ts, 0).map(|dt| dt.format("%Y-%m-%d").to_string())
            })
            .unwrap_or_else(|| "N/A".to_string());

        let firm = grade
            .firm
            .as_ref()
            .map(|f| truncate(f, 18))
            .unwrap_or_else(|| "N/A".to_string());

        let action = grade.action.as_deref().unwrap_or("N/A");
        let from_grade = grade.from_grade.as_deref().unwrap_or("-");
        let to_grade = grade.to_grade.as_deref().unwrap_or("-");

        // Color code the action
        let action_colored = match action {
            a if a.contains("up") => action.green().to_string(),
            a if a.contains("down") => action.red().to_string(),
            "init" => action.cyan().to_string(),
            "main" => action.white().to_string(),
            _ => action.to_string(),
        };

        // Format price target change
        let price_target = format_price_target(
            grade.prior_price_target,
            grade.current_price_target,
            grade.price_target_action.as_deref(),
        );

        println!(
            "{:<12} {:<20} {:<8} {:<12} {:<12} {:<15}",
            date, firm, action_colored, from_grade, to_grade, price_target
        );
    }

    // Summary stats
    let upgrades = grades.iter().filter(|g| g.is_upgrade()).count();
    let downgrades = grades.iter().filter(|g| g.is_downgrade()).count();

    if upgrades > 0 || downgrades > 0 {
        println!();
        println!(
            "Summary: {} {}, {} {}",
            upgrades,
            "upgrades".green(),
            downgrades,
            "downgrades".red()
        );
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn format_price_target(prior: Option<f64>, current: Option<f64>, action: Option<&str>) -> String {
    match (prior, current, action) {
        (Some(p), Some(c), Some(act)) => {
            let arrow = match act {
                "Raises" => "↑".green().to_string(),
                "Lowers" => "↓".red().to_string(),
                "Maintains" => "→".yellow().to_string(),
                _ => "".to_string(),
            };
            format!("${:.0} {} ${:.0}", p, arrow, c)
        }
        (None, Some(c), _) => format!("→ ${:.0}", c),
        (Some(p), None, _) => format!("${:.0} →", p),
        _ => "-".to_string(),
    }
}

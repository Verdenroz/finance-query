use crate::error::Result;
use colored::Colorize;
use serde::Serialize;
use tabled::{Table, Tabled, settings::Style};

/// Output format for CLI commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable table format (default)
    Table,
    /// JSON output
    Json,
    /// CSV output
    Csv,
}

impl OutputFormat {
    /// Parse output format from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            _ => Err(crate::error::CliError::InvalidArgument(format!(
                "Invalid output format '{}'. Valid formats: table, json, csv",
                s
            ))),
        }
    }
}

/// Print multiple items in the specified format
pub fn print_many<T>(data: &[T], format: OutputFormat) -> Result<()>
where
    T: Serialize + Tabled,
{
    match format {
        OutputFormat::Json => print_json(data),
        OutputFormat::Csv => print_csv_many(data),
        OutputFormat::Table => print_table_many(data),
    }
}

/// Print data as JSON
fn print_json<T: Serialize + ?Sized>(data: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{}", json);
    Ok(())
}

/// Print multiple items as CSV
fn print_csv_many<T: Serialize>(data: &[T]) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    for item in data {
        wtr.serialize(item)?;
    }
    wtr.flush()?;
    Ok(())
}

/// Print multiple items as a table
fn print_table_many<T: Tabled>(data: &[T]) -> Result<()> {
    if data.is_empty() {
        println!("{}", "No results found.".yellow());
        return Ok(());
    }

    let mut table = Table::new(data);
    table.with(Style::rounded());
    println!("{}", table);
    Ok(())
}

/// Print an info message
pub fn print_info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

/// Print a success message
pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

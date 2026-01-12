use crate::error::Result;
use crate::output;
use clap::Parser;
use finance_query::finance;

#[derive(Parser)]
pub struct TranscriptArgs {
    /// Stock symbol to get earnings transcript for
    #[arg(required = true)]
    symbol: String,

    /// Fiscal quarter (Q1, Q2, Q3, Q4). If not specified, gets latest.
    #[arg(short, long)]
    quarter: Option<String>,

    /// Fiscal year. If not specified, gets latest.
    #[arg(short, long)]
    year: Option<i32>,

    /// Show full transcript text
    #[arg(short, long)]
    full: bool,

    /// Number of lines to preview (if not showing full)
    #[arg(short, long, default_value = "20")]
    preview: usize,
}

pub async fn execute(args: TranscriptArgs) -> Result<()> {
    let transcript =
        finance::earnings_transcript(&args.symbol, args.quarter.as_deref(), args.year).await?;

    // Print metadata
    output::print_success(&format!(
        "Earnings Transcript: {} {} {}",
        args.symbol,
        transcript.quarter(),
        transcript.year()
    ));
    println!();

    let date = chrono::DateTime::from_timestamp(transcript.transcript_metadata.date, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "N/A".to_string());

    println!("Date:  {}", date);
    println!("Title: {}", transcript.transcript_metadata.title);
    println!("Type:  {}", transcript.transcript_metadata.event_type);
    println!();

    // Print transcript
    let text = transcript.text();
    if text.is_empty() {
        output::print_info("No transcript text available");
        return Ok(());
    }

    if args.full {
        println!("{}", text);
    } else {
        // Show preview
        let lines: Vec<&str> = text.lines().take(args.preview).collect();
        for line in lines {
            println!("{}", line);
        }

        let total_lines = text.lines().count();
        if total_lines > args.preview {
            println!();
            println!(
                "... {} more lines (use --full to see complete transcript)",
                total_lines - args.preview
            );
        }
    }

    Ok(())
}

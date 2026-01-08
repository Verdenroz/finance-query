use crate::error::Result;
use crate::output;
use clap::Parser;
use finance_query::Ticker;

#[derive(Parser)]
pub struct RecommendationsArgs {
    /// Stock symbol to get recommendations for
    #[arg(required = true)]
    symbol: String,

    /// Maximum number of recommendations to fetch
    #[arg(short, long, default_value = "10")]
    limit: u32,
}

pub async fn execute(args: RecommendationsArgs) -> Result<()> {
    let ticker = Ticker::new(&args.symbol).await?;
    let recommendations = ticker.recommendations(args.limit).await?;

    if recommendations.recommendations.is_empty() {
        output::print_info(&format!("No recommendations found for {}", args.symbol));
        return Ok(());
    }

    output::print_success(&format!(
        "Similar/Recommended symbols for {} ({} total)",
        recommendations.symbol,
        recommendations.count()
    ));
    println!();

    // Header
    println!("{:<10} {:<10}", "Symbol", "Score");
    println!("{}", "-".repeat(25));

    // Data rows
    for rec in &recommendations.recommendations {
        println!("{:<10} {:<10.4}", rec.symbol, rec.score);
    }

    Ok(())
}

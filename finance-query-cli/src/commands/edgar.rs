use crate::config::resolve_edgar_email;
use crate::error::Result;
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FormType {
    /// Annual report (10-K)
    #[value(name = "10-K")]
    TenK,
    /// Quarterly report (10-Q)
    #[value(name = "10-Q")]
    TenQ,
    /// Current report (8-K)
    #[value(name = "8-K")]
    EightK,
    /// Statement of changes in beneficial ownership (Form 4)
    #[value(name = "4")]
    Four,
    /// Registration statement for IPOs (S-1)
    #[value(name = "S-1")]
    SOne,
    /// Proxy statement (DEF 14A)
    #[value(name = "DEF 14A")]
    Def14A,
    /// Amendment to annual report (10-K/A)
    #[value(name = "10-K/A")]
    TenKA,
    /// Amendment to quarterly report (10-Q/A)
    #[value(name = "10-Q/A")]
    TenQA,
    /// Registration statement (S-3)
    #[value(name = "S-3")]
    SThree,
    /// Foreign private issuer annual report (20-F)
    #[value(name = "20-F")]
    TwentyF,
}

impl FormType {
    fn as_str(&self) -> &'static str {
        match self {
            FormType::TenK => "10-K",
            FormType::TenQ => "10-Q",
            FormType::EightK => "8-K",
            FormType::Four => "4",
            FormType::SOne => "S-1",
            FormType::Def14A => "DEF 14A",
            FormType::TenKA => "10-K/A",
            FormType::TenQA => "10-Q/A",
            FormType::SThree => "S-3",
            FormType::TwentyF => "20-F",
        }
    }
}

#[derive(Parser)]
pub struct EdgarArgs {
    /// Stock symbol to browse SEC filings for (optional - omit to start in search mode)
    pub symbol: Option<String>,

    /// Full-text search query across all SEC filings
    #[arg(short, long, conflicts_with = "symbol")]
    pub search: Option<String>,

    /// Filter by form type (e.g., 10-K, 10-Q, 8-K)
    #[arg(short = 'f', long, value_enum)]
    pub form_type: Option<Vec<FormType>>,

    /// Start date for search (YYYY-MM-DD, only with --search)
    #[arg(long, requires = "search")]
    pub start_date: Option<String>,

    /// End date for search (YYYY-MM-DD, only with --search)
    #[arg(long, requires = "search")]
    pub end_date: Option<String>,

    /// Email address for SEC EDGAR User-Agent (required by SEC)
    #[arg(short, long, env = "EDGAR_EMAIL")]
    pub email: Option<String>,
}

pub async fn execute(args: EdgarArgs) -> Result<()> {
    let email = resolve_edgar_email(args.email)?;

    // Initialize EDGAR singleton
    finance_query::edgar::init_with_config(
        email.clone(),
        "finance-query-cli",
        std::time::Duration::from_secs(30),
    )?;

    // Branch based on symbol vs search mode vs empty
    match (args.symbol, args.search) {
        (Some(symbol), None) => {
            // Symbol mode: browse filings for a specific company
            let cik = finance_query::edgar::resolve_cik(&symbol).await?;
            let submissions = finance_query::edgar::submissions(cik).await?;
            crate::edgar::run_symbol(symbol, submissions)?;
        }
        (None, Some(query)) => {
            // Search mode: full-text search across all filings
            let forms: Option<Vec<&str>> = args
                .form_type
                .as_ref()
                .map(|types| types.iter().map(|ft| ft.as_str()).collect());

            let results = finance_query::edgar::search(
                &query,
                forms.as_deref(),
                args.start_date.as_deref(),
                args.end_date.as_deref(),
                None, // from
                None, // size
            )
            .await?;

            crate::edgar::run_search(query, results)?;
        }
        (None, None) => {
            // Empty mode: start TUI ready for search
            crate::edgar::run_empty()?;
        }
        (Some(_), Some(_)) => {
            // This shouldn't happen due to conflicts_with, but handle it
            return Err(crate::error::CliError::InvalidArgument(
                "Cannot specify both symbol and search".to_string(),
            ));
        }
    }

    Ok(())
}

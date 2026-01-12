use crate::dashboard::run_dashboard;
use crate::error::Result;
use clap::Parser;

#[derive(Parser)]
pub struct DashboardArgs {
    // No args needed - TUI is interactive
}

pub async fn execute(_args: DashboardArgs) -> Result<()> {
    run_dashboard().await?;
    Ok(())
}

use crate::backtest::BacktestOptions;
use crate::error::Result;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct BacktestArgs {
    /// Stock symbol to backtest (optional, can be entered in TUI)
    pub symbol: Option<String>,

    /// Output JSON instead of TUI
    #[arg(long)]
    pub json: bool,

    /// Skip interactive TUI and run directly with preset
    #[arg(long)]
    pub no_tui: bool,

    /// Use a preset strategy: swing, day, trend, mean-reversion, conservative, aggressive
    #[arg(short, long)]
    pub preset: Option<String>,
}

pub async fn execute(args: BacktestArgs) -> Result<()> {
    let opts = BacktestOptions {
        symbol: args.symbol,
        json: args.json,
        no_tui: args.no_tui,
        preset: args.preset,
    };

    crate::backtest::execute(opts).await
}

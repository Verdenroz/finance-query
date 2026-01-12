// Thin wrapper for indicator command - TUI logic is in src/indicator/
pub use crate::indicator::IndicatorArgs;

use crate::error::Result;

pub async fn execute(args: IndicatorArgs) -> Result<()> {
    crate::indicator::execute(args).await
}

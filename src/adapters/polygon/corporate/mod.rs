//! Corporate data: news, dividends, splits, analyst ratings, earnings events.

#[allow(dead_code)] // unrouted: Benzinga premium surface has no capability route yet
pub(crate) mod benzinga;
mod corporate_actions;
mod news;

pub use corporate_actions::*;
pub use news::*;

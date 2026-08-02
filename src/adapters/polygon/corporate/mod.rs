//! Corporate data: news, dividends, splits, analyst ratings, earnings events.

#[allow(dead_code)] // unrouted: Benzinga premium surface has no capability route yet
mod benzinga;
mod corporate_actions;
#[allow(dead_code)] // unrouted: corporate-events feed has no capability route yet
mod corporate_events;
mod news;

pub use corporate_actions::*;
pub use news::*;

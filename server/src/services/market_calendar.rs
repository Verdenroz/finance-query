use std::sync::Arc;

use crate::cache::{self, Cache};
use finance_query::{CalendarKind, Providers};

use super::{ServiceError, ServiceResult};

/// Market-wide event calendar of `kind` over `[from, to]`, via
/// `Capability::CALENDAR` (Yahoo market status; the always-on local holiday
/// calendar; Nasdaq earnings/IPO/dividend/split; FRED economic releases when
/// configured).
pub async fn get_market_calendar(
    cache: &Cache,
    providers: &Arc<Providers>,
    kind: CalendarKind,
    from: &str,
    to: &str,
) -> ServiceResult {
    let cache_key = Cache::key("market-calendar", &[&format!("{kind:?}"), from, to]);
    let providers = Arc::clone(providers);
    let (from, to) = (from.to_string(), to.to_string());

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let entries = providers.calendar().fetch(kind, &from, &to).await?;
                serde_json::to_value(&entries).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

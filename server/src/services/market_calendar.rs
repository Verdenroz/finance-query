use std::sync::Arc;

use chrono::{Days, NaiveDate, Utc};

use crate::cache::{self, Cache};
use finance_query::{CalendarKind, Providers};

use super::{ServiceError, ServiceResult};

const DEFAULT_WINDOW_DAYS: u64 = 6;

/// An omitted `from` becomes `today`; an omitted `to` becomes six days after `from`.
fn window(from: &str, to: &str, today: NaiveDate) -> (String, String) {
    let from = if from.is_empty() {
        today.to_string()
    } else {
        from.to_string()
    };
    let to = if to.is_empty() {
        NaiveDate::parse_from_str(&from, "%Y-%m-%d")
            .map(|start| (start + Days::new(DEFAULT_WINDOW_DAYS)).to_string())
            .unwrap_or_default()
    } else {
        to.to_string()
    };
    (from, to)
}

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
    let (from, to) = window(from, to, Utc::now().date_naive());
    let cache_key = Cache::key("market-calendar", &[&format!("{kind:?}"), &from, &to]);
    let providers = Arc::clone(providers);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 15).unwrap()
    }

    #[test]
    fn omitted_dates_default_to_the_week_ahead() {
        assert_eq!(
            window("", "", today()),
            ("2026-11-15".to_string(), "2026-11-21".to_string())
        );
    }

    #[test]
    fn omitted_to_follows_the_given_from() {
        assert_eq!(
            window("2026-12-30", "", today()),
            ("2026-12-30".to_string(), "2027-01-05".to_string())
        );
    }

    #[test]
    fn given_dates_pass_through_unchanged() {
        assert_eq!(
            window("2026-01-01", "2026-01-31", today()),
            ("2026-01-01".to_string(), "2026-01-31".to_string())
        );
    }
}

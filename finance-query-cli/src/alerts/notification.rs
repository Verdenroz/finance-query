use notify_rust::{Notification, Timeout};

use super::Alert;

/// Send a desktop notification for a triggered alert
pub fn send_alert_notification(alert: &Alert, current_value: Option<f64>) {
    let title = format!("{} Alert", alert.symbol);

    let body = if let Some(val) = current_value {
        format!(
            "{}\nCurrent: {} | Target: {}",
            alert.alert_type.display(),
            alert.alert_type.format_current_value(val),
            alert.alert_type.format_threshold(alert.threshold)
        )
    } else {
        format!(
            "{}\nTarget: {}",
            alert.alert_type.display(),
            alert.alert_type.format_threshold(alert.threshold)
        )
    };

    // Fire and forget - don't block on notification errors
    let _ = Notification::new()
        .summary(&title)
        .body(&body)
        .appname("Finance Query")
        .timeout(Timeout::Milliseconds(8000)) // 8 seconds
        .show();
}

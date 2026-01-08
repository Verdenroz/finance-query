use crate::alerts::{Alert, AlertStore, AlertType};
use crate::error::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use finance_query::Tickers;

#[derive(Parser)]
pub struct AlertsArgs {
    #[command(subcommand)]
    command: Option<AlertCommand>,
}

#[derive(Subcommand)]
enum AlertCommand {
    /// Add a new alert
    Add {
        /// Stock symbol to monitor
        symbol: String,

        /// Alert type and threshold (e.g., price-above:150, volume-spike:2.0, change-above:5)
        #[arg(value_name = "TYPE:VALUE")]
        alert: String,

        /// Optional label/note for this alert
        #[arg(short, long)]
        label: Option<String>,
    },

    /// List all active alerts
    List {
        /// Filter by symbol
        #[arg(short, long)]
        symbol: Option<String>,

        /// Show only enabled alerts
        #[arg(short, long)]
        enabled: bool,
    },

    /// Remove an alert by ID
    Remove {
        /// Alert ID to remove
        id: i64,
    },

    /// Check all alerts and show which ones triggered
    Check {
        /// Only show triggered alerts
        #[arg(short, long)]
        triggered: bool,
    },

    /// Enable/disable an alert
    Toggle {
        /// Alert ID to toggle
        id: i64,
    },

    /// Clear all alerts
    Clear {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Show alert type examples and usage
    Examples,

    /// Watch alerts continuously in the background (for use with systemd/launchd)
    Watch {
        /// Polling interval in seconds (default: 60)
        #[arg(short, long, default_value = "60")]
        interval: u64,

        /// Run in foreground with verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Install/manage the alert watcher as a system service
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
}

#[derive(Subcommand)]
enum ServiceAction {
    /// Install the alert watcher as a user service
    Install {
        /// Polling interval in seconds (default: 60)
        #[arg(short, long, default_value = "60")]
        interval: u64,
    },
    /// Uninstall the alert watcher service
    Uninstall,
    /// Show service status
    Status,
    /// Start the service
    Start,
    /// Stop the service
    Stop,
}

pub async fn execute(args: AlertsArgs) -> Result<()> {
    match args.command {
        None => {
            // No subcommand - launch TUI
            crate::alerts::run_alerts_tui().await.map_err(Into::into)
        }
        Some(AlertCommand::Add {
            symbol,
            alert,
            label,
        }) => add_alert(symbol, alert, label).await,
        Some(AlertCommand::List { symbol, enabled }) => list_alerts(symbol, enabled).await,
        Some(AlertCommand::Remove { id }) => remove_alert(id).await,
        Some(AlertCommand::Check { triggered }) => check_alerts(triggered).await,
        Some(AlertCommand::Toggle { id }) => toggle_alert(id).await,
        Some(AlertCommand::Clear { force }) => clear_alerts(force).await,
        Some(AlertCommand::Examples) => show_examples(),
        Some(AlertCommand::Watch { interval, verbose }) => watch_alerts(interval, verbose).await,
        Some(AlertCommand::Service { action }) => manage_service(action).await,
    }
}

async fn add_alert(symbol: String, alert_spec: String, label: Option<String>) -> Result<()> {
    // Parse alert specification: type:threshold
    let parts: Vec<&str> = alert_spec.split(':').collect();
    if parts.len() != 2 {
        eprintln!(
            "{}",
            "Error: Alert must be in format TYPE:VALUE (e.g., price-above:150)".red()
        );
        eprintln!("Run 'fq alerts examples' for usage examples");
        return Ok(());
    }

    let alert_type: AlertType = parts[0].parse().map_err(|e| {
        anyhow::anyhow!(
            "Unknown alert type: {}. Run 'fq alerts examples' for valid types\n{}",
            parts[0],
            e
        )
    })?;

    let threshold: f64 = parts[1]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid threshold value: {}. Must be a number", parts[1]))?;

    // Create alert
    let store = AlertStore::new()?;
    let id = store.create_alert(
        &symbol.to_uppercase(),
        alert_type,
        threshold,
        label.as_deref(),
    )?;
    let alerts = store.get_alerts()?;
    let alert = alerts.iter().find(|a| a.id == id).unwrap();

    println!("{}", "✓ Alert added successfully!".green().bold());
    println!();
    println!("{} {}", "ID:".bold(), id.to_string().cyan());
    println!("{} {}", "Symbol:".bold(), alert.symbol.yellow());
    println!("{} {}", "Type:".bold(), alert.alert_type.display());
    println!(
        "{} {}",
        "Threshold:".bold(),
        alert.alert_type.format_threshold(alert.threshold)
    );
    if let Some(lbl) = &alert.label {
        println!("{} {}", "Label:".bold(), lbl.dimmed());
    }

    Ok(())
}

async fn list_alerts(symbol: Option<String>, enabled_only: bool) -> Result<()> {
    let store = AlertStore::new()?;

    let alerts = if let Some(ref sym) = symbol {
        store.get_alerts_for_symbol(sym)?
    } else if enabled_only {
        store.get_enabled_alerts()?
    } else {
        store.get_alerts()?
    };

    if alerts.is_empty() {
        println!("{}", "No alerts found.".yellow());
        println!("Run {} to add your first alert.", "fq alerts add".cyan());
        return Ok(());
    }

    println!("{}", "Active Alerts".bold().blue());
    println!("{}", "━".repeat(80).blue());
    println!();

    for alert in &alerts {
        print_alert(alert);
    }

    println!(
        "\n{} Run {} to check current data against alerts",
        "Tip:".bold(),
        "fq alerts check".cyan()
    );

    Ok(())
}

async fn remove_alert(id: i64) -> Result<()> {
    let store = AlertStore::new()?;

    if store.delete_alert(id)? {
        println!("{} Alert {} removed.", "✓".green(), id.to_string().cyan());
    } else {
        println!("{} Alert {} not found.", "✗".red(), id.to_string().yellow());
    }

    Ok(())
}

async fn toggle_alert(id: i64) -> Result<()> {
    let store = AlertStore::new()?;
    let alerts = store.get_alerts()?;

    if let Some(alert) = alerts.iter().find(|a| a.id == id) {
        let new_state = !alert.enabled;
        store.set_enabled(id, new_state)?;
        let status = if new_state { "enabled" } else { "disabled" };
        println!("{} Alert {} {}", "✓".green(), id.to_string().cyan(), status);
    } else {
        println!("{} Alert {} not found.", "✗".red(), id.to_string().yellow());
    }

    Ok(())
}

async fn check_alerts(show_only_triggered: bool) -> Result<()> {
    use crate::alerts::send_alert_notification;

    let store = AlertStore::new()?;
    let alerts = store.get_enabled_alerts()?;

    if alerts.is_empty() {
        println!("{}", "No alerts configured.".yellow());
        println!("Run {} to add your first alert.", "fq alerts add".cyan());
        return Ok(());
    }

    // Get unique symbols
    let symbols: Vec<String> = alerts.iter().map(|a| a.symbol.clone()).collect();
    let unique_symbols: Vec<String> = symbols
        .into_iter()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    println!("{}", "Checking alerts...".blue());
    println!();

    // Fetch quotes
    let tickers = Tickers::new(&unique_symbols).await?;
    let response = tickers.quotes(false).await?;

    let mut triggered_count = 0;
    let mut triggered_ids = Vec::new();

    // Check each alert
    for alert in &alerts {
        if let Some(quote) = response.quotes.get(&alert.symbol) {
            let triggered = alert.check(quote);

            if triggered {
                triggered_count += 1;
                triggered_ids.push((alert.id, alert.clone(), alert.get_current_value(quote)));
            }

            if !show_only_triggered || triggered {
                print_alert_with_quote(alert, quote, triggered);
            }
        }
    }

    // Mark triggered alerts and send notifications
    for (id, alert, current_value) in &triggered_ids {
        let _ = store.mark_triggered(*id);
        send_alert_notification(alert, *current_value);
    }

    // Summary
    println!();
    println!("{}", "━".repeat(80).blue());
    if triggered_count > 0 {
        println!(
            "🔔 {} alert(s) triggered!",
            triggered_count.to_string().red().bold()
        );
    } else {
        println!("{}", "✓ No alerts triggered.".green());
    }

    Ok(())
}

async fn clear_alerts(force: bool) -> Result<()> {
    if !force {
        println!(
            "{}",
            "Are you sure you want to clear all alerts? [y/N]".yellow()
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    let store = AlertStore::new()?;
    let count = store.clear_all()?;

    println!("{} Cleared {} alert(s).", "✓".green(), count);

    Ok(())
}

fn show_examples() -> Result<()> {
    println!("{}", "Finance Query Alerts - Help".bold().blue());
    println!("{}", "━".repeat(80).blue());
    println!();
    println!("{}", "Alert Types:".bold());
    println!();

    let examples = vec![
        (
            "price-above:VALUE",
            "Alert when price goes above VALUE",
            "price-above:150",
        ),
        (
            "price-below:VALUE",
            "Alert when price goes below VALUE",
            "price-below:100",
        ),
        (
            "change-above:PERCENT",
            "Alert when daily % change exceeds PERCENT",
            "change-above:5",
        ),
        (
            "change-below:PERCENT",
            "Alert when daily % change drops below PERCENT",
            "change-below:-3",
        ),
        (
            "volume-spike:MULTIPLIER",
            "Alert when volume is MULTIPLIER times average",
            "volume-spike:2.0",
        ),
        (
            "52w-high:PERCENT",
            "Alert when within PERCENT of 52-week high",
            "52w-high:2",
        ),
        (
            "52w-low:PERCENT",
            "Alert when within PERCENT of 52-week low",
            "52w-low:5",
        ),
        (
            "mcap-above:BILLIONS",
            "Alert when market cap above BILLIONS",
            "mcap-above:100",
        ),
        (
            "mcap-below:BILLIONS",
            "Alert when market cap below BILLIONS",
            "mcap-below:50",
        ),
        (
            "div-yield-above:PERCENT",
            "Alert when dividend yield above PERCENT",
            "div-yield-above:4",
        ),
        (
            "pe-above:RATIO",
            "Alert when P/E ratio above RATIO",
            "pe-above:30",
        ),
        (
            "pe-below:RATIO",
            "Alert when P/E ratio below RATIO",
            "pe-below:15",
        ),
    ];

    for (alert_type, description, example) in examples {
        println!("  {} {}", alert_type.cyan().bold(), "-".dimmed());
        println!("    {}", description);
        println!("    {} {}", "Example:".bold(), example.yellow());
        println!();
    }

    println!("{}", "Usage Examples:".bold());
    println!();
    println!("  {} Add an alert when AAPL goes above $150", "#".dimmed());
    println!("  {}", "fq alerts add AAPL price-above:150".green());
    println!();
    println!("  {} Add an alert with a custom label", "#".dimmed());
    println!(
        "  {}",
        "fq alerts add TSLA change-above:5 --label \"Big move day\"".green()
    );
    println!();
    println!("  {} Check all alerts against current prices", "#".dimmed());
    println!("  {}", "fq alerts check".green());
    println!();
    println!("  {} List all configured alerts", "#".dimmed());
    println!("  {}", "fq alerts list".green());
    println!();
    println!("  {} Remove an alert by ID", "#".dimmed());
    println!("  {}", "fq alerts remove 42".green());
    println!();
    println!("  {} Disable an alert without deleting", "#".dimmed());
    println!("  {}", "fq alerts toggle 42".green());

    Ok(())
}

fn print_alert(alert: &Alert) {
    let status = if !alert.enabled {
        "Disabled".yellow()
    } else if alert.last_triggered.is_some() {
        format!("Triggered ({}x)", alert.trigger_count)
            .yellow()
            .bold()
    } else {
        "Active".green()
    };

    println!("{} {}", "ID:".bold(), alert.id.to_string().cyan());
    println!("{} {}", "Symbol:".bold(), alert.symbol.yellow().bold());
    println!("{} {}", "Type:".bold(), alert.alert_type.display());
    println!(
        "{} {}",
        "Threshold:".bold(),
        alert.alert_type.format_threshold(alert.threshold)
    );
    println!("{} {}", "Status:".bold(), status);

    if let Some(label) = &alert.label {
        println!("{} {}", "Label:".bold(), label.dimmed());
    }

    if let Some(last_triggered) = alert.last_triggered {
        println!(
            "{} {}",
            "Last Triggered:".bold(),
            last_triggered
                .format("%Y-%m-%d %H:%M UTC")
                .to_string()
                .dimmed()
        );
    }

    println!();
}

fn print_alert_with_quote(alert: &Alert, quote: &finance_query::Quote, triggered: bool) {
    let status_icon = if triggered { "🔔" } else { "○" };
    let status_color = if triggered {
        alert.symbol.red().bold()
    } else {
        alert.symbol.green()
    };

    println!(
        "{} {} - {}",
        status_icon,
        status_color,
        alert.alert_type.display()
    );

    // Show current value vs threshold
    println!(
        "  {} {} {} {}",
        "Threshold:".bold(),
        alert.alert_type.format_threshold(alert.threshold),
        "|".dimmed(),
        alert.format_current_value(quote)
    );

    if let Some(label) = &alert.label {
        println!("  {} {}", "Label:".bold(), label.dimmed());
    }

    println!();
}

/// Continuously watch and check alerts
async fn watch_alerts(interval_secs: u64, verbose: bool) -> Result<()> {
    use crate::alerts::send_alert_notification;
    use std::collections::HashSet;
    use tokio::time::{Duration, interval};

    if verbose {
        println!("{}", "Starting alert watcher...".green().bold());
        println!("Polling interval: {} seconds", interval_secs);
        println!("Press Ctrl+C to stop\n");
    }

    let mut poll_interval = interval(Duration::from_secs(interval_secs));

    loop {
        poll_interval.tick().await;

        let store = match AlertStore::new() {
            Ok(s) => s,
            Err(e) => {
                if verbose {
                    eprintln!("{} Failed to open alert store: {}", "Error:".red(), e);
                }
                continue;
            }
        };

        let alerts = match store.get_enabled_alerts() {
            Ok(a) => a,
            Err(e) => {
                if verbose {
                    eprintln!("{} Failed to load alerts: {}", "Error:".red(), e);
                }
                continue;
            }
        };

        if alerts.is_empty() {
            if verbose {
                println!("{} No enabled alerts", "[check]".dimmed());
            }
            continue;
        }

        // Get unique symbols
        let symbols: HashSet<String> = alerts.iter().map(|a| a.symbol.clone()).collect();
        let unique_symbols: Vec<String> = symbols.into_iter().collect();

        if verbose {
            println!(
                "{} Checking {} alerts for {} symbols...",
                "[check]".blue(),
                alerts.len(),
                unique_symbols.len()
            );
        }

        // Fetch quotes
        let tickers = match Tickers::new(&unique_symbols).await {
            Ok(t) => t,
            Err(e) => {
                if verbose {
                    eprintln!("{} Failed to create tickers: {}", "Error:".red(), e);
                }
                continue;
            }
        };

        let response = match tickers.quotes(false).await {
            Ok(r) => r,
            Err(e) => {
                if verbose {
                    eprintln!("{} Failed to fetch quotes: {}", "Error:".red(), e);
                }
                continue;
            }
        };

        // Check each alert
        let mut triggered_count = 0;
        for alert in &alerts {
            if let Some(quote) = response.quotes.get(&alert.symbol)
                && alert.check(quote)
            {
                triggered_count += 1;
                let _ = store.mark_triggered(alert.id);

                // Send desktop notification
                let current_value = alert.get_current_value(quote);
                send_alert_notification(alert, current_value);

                if verbose {
                    println!(
                        "{} {} {} - {}",
                        "🔔".red().bold(),
                        alert.symbol.yellow().bold(),
                        alert.alert_type.display(),
                        alert.alert_type.format_threshold(alert.threshold)
                    );
                }
            }
        }

        if verbose && triggered_count == 0 {
            println!("{} No alerts triggered", "[check]".dimmed());
        }
    }
}

/// Manage the alert watcher as a system service
async fn manage_service(action: ServiceAction) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        manage_systemd_service(action).await
    }

    #[cfg(target_os = "macos")]
    {
        return manage_launchd_service(action).await;
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        println!(
            "{}",
            "Service management not supported on this platform.".yellow()
        );
        println!(
            "You can run {} manually or use your OS task scheduler.",
            "fq alerts watch".cyan()
        );
        Ok(())
    }
}

#[cfg(target_os = "linux")]
async fn manage_systemd_service(action: ServiceAction) -> Result<()> {
    use std::fs;
    use std::process::Command;

    let service_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("systemd/user");

    let service_file = service_dir.join("fq-alerts.service");

    // Get the path to fq executable
    let fq_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "fq".to_string());

    match action {
        ServiceAction::Install { interval } => {
            // Create directory if needed
            fs::create_dir_all(&service_dir)?;

            let service_content = format!(
                r#"[Unit]
Description=Finance Query Alert Watcher
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart={} alerts watch --interval {}
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
"#,
                fq_path, interval
            );

            fs::write(&service_file, service_content)?;

            println!("{}", "Service file created.".green());
            println!("Location: {}", service_file.display().to_string().cyan());

            // Reload systemd
            let _ = Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .status();

            // Enable the service
            let status = Command::new("systemctl")
                .args(["--user", "enable", "fq-alerts.service"])
                .status()?;

            if status.success() {
                println!("{}", "Service enabled.".green());
                println!();
                println!("To start the service now, run:");
                println!("  {}", "fq alerts service start".cyan());
                println!();
                println!("To check status:");
                println!("  {}", "fq alerts service status".cyan());
            } else {
                println!("{}", "Failed to enable service.".red());
            }

            Ok(())
        }
        ServiceAction::Uninstall => {
            // Stop and disable the service first
            let _ = Command::new("systemctl")
                .args(["--user", "stop", "fq-alerts.service"])
                .status();
            let _ = Command::new("systemctl")
                .args(["--user", "disable", "fq-alerts.service"])
                .status();

            // Remove the service file
            if service_file.exists() {
                fs::remove_file(&service_file)?;
                println!("{}", "Service uninstalled.".green());

                // Reload systemd
                let _ = Command::new("systemctl")
                    .args(["--user", "daemon-reload"])
                    .status();
            } else {
                println!("{}", "Service was not installed.".yellow());
            }

            Ok(())
        }
        ServiceAction::Status => {
            let output = Command::new("systemctl")
                .args(["--user", "status", "fq-alerts.service"])
                .output()?;

            println!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }

            Ok(())
        }
        ServiceAction::Start => {
            let status = Command::new("systemctl")
                .args(["--user", "start", "fq-alerts.service"])
                .status()?;

            if status.success() {
                println!("{}", "Service started.".green());
            } else {
                println!("{}", "Failed to start service.".red());
                println!("Check status with: {}", "fq alerts service status".cyan());
            }

            Ok(())
        }
        ServiceAction::Stop => {
            let status = Command::new("systemctl")
                .args(["--user", "stop", "fq-alerts.service"])
                .status()?;

            if status.success() {
                println!("{}", "Service stopped.".green());
            } else {
                println!("{}", "Failed to stop service.".red());
            }

            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
async fn manage_launchd_service(action: ServiceAction) -> Result<()> {
    use std::fs;
    use std::process::Command;

    let plist_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join("Library/LaunchAgents");

    let plist_file = plist_dir.join("com.finance-query.alerts.plist");

    // Get the path to fq executable
    let fq_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "/usr/local/bin/fq".to_string());

    match action {
        ServiceAction::Install { interval } => {
            // Create directory if needed
            fs::create_dir_all(&plist_dir)?;

            let plist_content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.finance-query.alerts</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>alerts</string>
        <string>watch</string>
        <string>--interval</string>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardErrorPath</key>
    <string>/tmp/fq-alerts.err</string>
    <key>StandardOutPath</key>
    <string>/tmp/fq-alerts.out</string>
</dict>
</plist>
"#,
                fq_path, interval
            );

            fs::write(&plist_file, plist_content)?;

            println!("{}", "Service plist created.".green());
            println!("Location: {}", plist_file.display().to_string().cyan());

            // Load the service
            let status = Command::new("launchctl")
                .args(["load", &plist_file.display().to_string()])
                .status()?;

            if status.success() {
                println!("{}", "Service loaded and started.".green());
            } else {
                println!("{}", "Failed to load service.".red());
            }

            Ok(())
        }
        ServiceAction::Uninstall => {
            if plist_file.exists() {
                // Unload first
                let _ = Command::new("launchctl")
                    .args(["unload", &plist_file.display().to_string()])
                    .status();

                fs::remove_file(&plist_file)?;
                println!("{}", "Service uninstalled.".green());
            } else {
                println!("{}", "Service was not installed.".yellow());
            }

            Ok(())
        }
        ServiceAction::Status => {
            let output = Command::new("launchctl")
                .args(["list", "com.finance-query.alerts"])
                .output()?;

            if output.status.success() {
                println!("{}", "Service is running.".green());
                println!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                println!("{}", "Service is not running.".yellow());
            }

            Ok(())
        }
        ServiceAction::Start => {
            let status = Command::new("launchctl")
                .args(["start", "com.finance-query.alerts"])
                .status()?;

            if status.success() {
                println!("{}", "Service started.".green());
            } else {
                println!("{}", "Failed to start service.".red());
            }

            Ok(())
        }
        ServiceAction::Stop => {
            let status = Command::new("launchctl")
                .args(["stop", "com.finance-query.alerts"])
                .status()?;

            if status.success() {
                println!("{}", "Service stopped.".green());
            } else {
                println!("{}", "Failed to stop service.".red());
            }

            Ok(())
        }
    }
}

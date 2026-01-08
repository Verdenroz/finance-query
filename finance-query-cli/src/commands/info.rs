use crate::error::Result;
use crate::output::OutputFormat;
use clap::Parser;
use colored::Colorize;
use finance_query::Ticker;
use serde::Serialize;

#[derive(Parser)]
pub struct InfoArgs {
    /// Stock symbol to get information for
    #[arg(required = true)]
    symbol: String,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    output: String,

    /// Show compact view (less details)
    #[arg(short, long)]
    compact: bool,
}

#[derive(Debug, Serialize)]
struct InfoJson {
    symbol: String,
    name: Option<String>,
    exchange: Option<String>,
    quote_type: Option<String>,
    currency: Option<String>,
    price: Option<f64>,
    change: Option<f64>,
    change_percent: Option<f64>,
    open: Option<f64>,
    day_high: Option<f64>,
    day_low: Option<f64>,
    previous_close: Option<f64>,
    volume: Option<i64>,
    market_cap: Option<i64>,
    pe_ratio: Option<f64>,
    forward_pe: Option<f64>,
    eps: Option<f64>,
    dividend_yield: Option<f64>,
    beta: Option<f64>,
    week_52_low: Option<f64>,
    week_52_high: Option<f64>,
    fifty_day_avg: Option<f64>,
    two_hundred_day_avg: Option<f64>,
    profit_margin: Option<f64>,
    operating_margin: Option<f64>,
    return_on_equity: Option<f64>,
    revenue_growth: Option<f64>,
    earnings_growth: Option<f64>,
    analyst_rating: Option<String>,
    target_price: Option<f64>,
    sector: Option<String>,
    industry: Option<String>,
    employees: Option<i64>,
    website: Option<String>,
    description: Option<String>,
}

pub async fn execute(args: InfoArgs) -> Result<()> {
    let format = OutputFormat::from_str(&args.output)?;

    // Fetch full quote data
    let ticker = Ticker::new(&args.symbol).await?;
    let quote = ticker.quote(false).await?;

    // For JSON/CSV output, return structured data
    if format != OutputFormat::Table {
        let info_json = InfoJson {
            symbol: quote.symbol.clone(),
            name: quote.long_name.clone().or(quote.short_name.clone()),
            exchange: quote.exchange_name.clone().or(quote.exchange.clone()),
            quote_type: quote.quote_type.clone(),
            currency: quote.currency.clone(),
            price: quote.regular_market_price.as_ref().and_then(|v| v.raw),
            change: quote.regular_market_change.as_ref().and_then(|v| v.raw),
            change_percent: quote
                .regular_market_change_percent
                .as_ref()
                .and_then(|v| v.raw),
            open: quote.regular_market_open.as_ref().and_then(|v| v.raw),
            day_high: quote.regular_market_day_high.as_ref().and_then(|v| v.raw),
            day_low: quote.regular_market_day_low.as_ref().and_then(|v| v.raw),
            previous_close: quote
                .regular_market_previous_close
                .as_ref()
                .and_then(|v| v.raw),
            volume: quote.regular_market_volume.as_ref().and_then(|v| v.raw),
            market_cap: quote.market_cap.as_ref().and_then(|v| v.raw),
            pe_ratio: quote.trailing_pe.as_ref().and_then(|v| v.raw),
            forward_pe: quote.forward_pe.as_ref().and_then(|v| v.raw),
            eps: quote.trailing_eps.as_ref().and_then(|v| v.raw),
            dividend_yield: quote
                .trailing_annual_dividend_yield
                .as_ref()
                .and_then(|v| v.raw),
            beta: quote.beta.as_ref().and_then(|v| v.raw),
            week_52_low: quote.fifty_two_week_low.as_ref().and_then(|v| v.raw),
            week_52_high: quote.fifty_two_week_high.as_ref().and_then(|v| v.raw),
            fifty_day_avg: quote.fifty_day_average.as_ref().and_then(|v| v.raw),
            two_hundred_day_avg: quote.two_hundred_day_average.as_ref().and_then(|v| v.raw),
            profit_margin: quote.profit_margins.as_ref().and_then(|v| v.raw),
            operating_margin: quote.operating_margins.as_ref().and_then(|v| v.raw),
            return_on_equity: quote.return_on_equity.as_ref().and_then(|v| v.raw),
            revenue_growth: quote.revenue_growth.as_ref().and_then(|v| v.raw),
            earnings_growth: quote.earnings_growth.as_ref().and_then(|v| v.raw),
            analyst_rating: quote.recommendation_key.clone(),
            target_price: quote.target_mean_price.as_ref().and_then(|v| v.raw),
            sector: quote.sector.clone(),
            industry: quote.industry.clone(),
            employees: quote.full_time_employees,
            website: quote.website.clone(),
            description: quote.long_business_summary.clone(),
        };

        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&info_json)?);
            }
            OutputFormat::Csv => {
                let mut wtr = csv::Writer::from_writer(std::io::stdout());
                wtr.serialize(&info_json)?;
                wtr.flush()?;
            }
            _ => {}
        }
        return Ok(());
    }

    // Rich table output
    let term_width = terminal_width();

    // Extract values
    let name = quote
        .long_name
        .clone()
        .or(quote.short_name.clone())
        .unwrap_or_default();
    let exchange = quote
        .exchange_name
        .clone()
        .or(quote.exchange.clone())
        .unwrap_or_default();
    let currency = quote.currency.clone().unwrap_or_else(|| "USD".to_string());
    let currency_symbol = quote
        .currency_symbol
        .clone()
        .unwrap_or_else(|| "$".to_string());

    let price = quote.regular_market_price.as_ref().and_then(|v| v.raw);
    let change = quote.regular_market_change.as_ref().and_then(|v| v.raw);
    let change_pct = quote
        .regular_market_change_percent
        .as_ref()
        .and_then(|v| v.raw);

    // Print header
    println!();
    print_header(&quote.symbol, &name, &exchange, term_width);

    // Price section
    print_section_header("Price", term_width);
    print_price_line(price, change, change_pct, &currency_symbol);

    // Day's trading range
    let day_low = quote.regular_market_day_low.as_ref().and_then(|v| v.raw);
    let day_high = quote.regular_market_day_high.as_ref().and_then(|v| v.raw);
    if let (Some(low), Some(high), Some(current)) = (day_low, day_high, price) {
        print_range_bar(
            "Day Range",
            low,
            high,
            current,
            &currency_symbol,
            term_width,
        );
    }

    // 52-week range
    let week_52_low = quote.fifty_two_week_low.as_ref().and_then(|v| v.raw);
    let week_52_high = quote.fifty_two_week_high.as_ref().and_then(|v| v.raw);
    if let (Some(low), Some(high), Some(current)) = (week_52_low, week_52_high, price) {
        print_range_bar(
            "52W Range",
            low,
            high,
            current,
            &currency_symbol,
            term_width,
        );
    }

    // Key stats grid
    let prev_close = quote
        .regular_market_previous_close
        .as_ref()
        .and_then(|v| v.raw);
    let open = quote.regular_market_open.as_ref().and_then(|v| v.raw);
    let volume = quote.regular_market_volume.as_ref().and_then(|v| v.raw);
    let avg_volume = quote.average_volume.as_ref().and_then(|v| v.raw);

    println!();
    print_key_value_pair(
        "Open",
        open.map(|v| format!("{}{:.2}", currency_symbol, v)),
        "Prev Close",
        prev_close.map(|v| format!("{}{:.2}", currency_symbol, v)),
        term_width,
    );
    print_key_value_pair(
        "Volume",
        volume.map(format_volume),
        "Avg Volume",
        avg_volume.map(format_volume),
        term_width,
    );

    // Market data section
    print_section_header("Valuation", term_width);

    let market_cap = quote.market_cap.as_ref().and_then(|v| v.raw);
    let enterprise_value = quote.enterprise_value.as_ref().and_then(|v| v.raw);
    print_key_value_pair(
        "Market Cap",
        market_cap.map(format_market_cap),
        "Enterprise Value",
        enterprise_value.map(format_market_cap),
        term_width,
    );

    let pe = quote.trailing_pe.as_ref().and_then(|v| v.raw);
    let forward_pe = quote.forward_pe.as_ref().and_then(|v| v.raw);
    print_key_value_pair(
        "P/E (TTM)",
        pe.map(|v| format!("{:.2}", v)),
        "Forward P/E",
        forward_pe.map(|v| format!("{:.2}", v)),
        term_width,
    );

    let eps = quote.trailing_eps.as_ref().and_then(|v| v.raw);
    let forward_eps = quote.forward_eps.as_ref().and_then(|v| v.raw);
    print_key_value_pair(
        "EPS (TTM)",
        eps.map(|v| format!("{}{:.2}", currency_symbol, v)),
        "Forward EPS",
        forward_eps.map(|v| format!("{}{:.2}", currency_symbol, v)),
        term_width,
    );

    let price_to_book = quote.price_to_book.as_ref().and_then(|v| v.raw);
    let price_to_sales = quote
        .price_to_sales_trailing12_months
        .as_ref()
        .and_then(|v| v.raw);
    print_key_value_pair(
        "Price/Book",
        price_to_book.map(|v| format!("{:.2}", v)),
        "Price/Sales",
        price_to_sales.map(|v| format!("{:.2}", v)),
        term_width,
    );

    // Dividends & yield
    let div_yield = quote
        .trailing_annual_dividend_yield
        .as_ref()
        .and_then(|v| v.raw);
    let div_rate = quote.dividend_rate.as_ref().and_then(|v| v.raw);
    if div_yield.is_some() || div_rate.is_some() {
        print_key_value_pair(
            "Dividend Yield",
            div_yield.map(|v| format!("{:.2}%", v * 100.0)),
            "Annual Dividend",
            div_rate.map(|v| format!("{}{:.2}", currency_symbol, v)),
            term_width,
        );
    }

    // Fundamentals section (skip if compact mode)
    if !args.compact {
        let has_fundamentals = quote.profit_margins.is_some()
            || quote.return_on_equity.is_some()
            || quote.revenue_growth.is_some()
            || quote.earnings_growth.is_some();

        if has_fundamentals {
            print_section_header("Fundamentals", term_width);

            let profit_margin = quote.profit_margins.as_ref().and_then(|v| v.raw);
            let operating_margin = quote.operating_margins.as_ref().and_then(|v| v.raw);
            print_key_value_pair(
                "Profit Margin",
                profit_margin.map(format_percent),
                "Operating Margin",
                operating_margin.map(format_percent),
                term_width,
            );

            let roe = quote.return_on_equity.as_ref().and_then(|v| v.raw);
            let roa = quote.return_on_assets.as_ref().and_then(|v| v.raw);
            print_key_value_pair(
                "Return on Equity",
                roe.map(format_percent),
                "Return on Assets",
                roa.map(format_percent),
                term_width,
            );

            let revenue_growth = quote.revenue_growth.as_ref().and_then(|v| v.raw);
            let earnings_growth = quote.earnings_growth.as_ref().and_then(|v| v.raw);
            print_key_value_pair(
                "Revenue Growth",
                revenue_growth.map(format_growth),
                "Earnings Growth",
                earnings_growth.map(format_growth),
                term_width,
            );

            let debt_to_equity = quote.debt_to_equity.as_ref().and_then(|v| v.raw);
            let current_ratio = quote.current_ratio.as_ref().and_then(|v| v.raw);
            print_key_value_pair(
                "Debt/Equity",
                debt_to_equity.map(|v| format!("{:.2}", v)),
                "Current Ratio",
                current_ratio.map(|v| format!("{:.2}", v)),
                term_width,
            );
        }

        // Moving averages
        let ma_50 = quote.fifty_day_average.as_ref().and_then(|v| v.raw);
        let ma_200 = quote.two_hundred_day_average.as_ref().and_then(|v| v.raw);
        if ma_50.is_some() || ma_200.is_some() {
            print_section_header("Technical", term_width);
            print_key_value_pair(
                "50-Day MA",
                ma_50.map(|v| format!("{}{:.2}", currency_symbol, v)),
                "200-Day MA",
                ma_200.map(|v| format!("{}{:.2}", currency_symbol, v)),
                term_width,
            );

            let beta = quote.beta.as_ref().and_then(|v| v.raw);
            if let Some(b) = beta {
                print_single_value("Beta", &format!("{:.2}", b), term_width);
            }
        }

        // Analyst ratings
        let has_analyst = quote.recommendation_key.is_some() || quote.target_mean_price.is_some();
        if has_analyst {
            print_section_header("Analyst Ratings", term_width);

            if let Some(rating) = &quote.recommendation_key {
                let colored_rating = match rating.to_lowercase().as_str() {
                    "strong_buy" | "strongbuy" => "Strong Buy".green().bold(),
                    "buy" => "Buy".green(),
                    "hold" => "Hold".yellow(),
                    "sell" => "Sell".red(),
                    "strong_sell" | "strongsell" => "Strong Sell".red().bold(),
                    _ => rating.normal(),
                };
                let num_analysts = quote
                    .number_of_analyst_opinions
                    .as_ref()
                    .and_then(|v| v.raw);
                let analysts_str = num_analysts
                    .map(|n| format!(" ({} analysts)", n))
                    .unwrap_or_default();
                println!(
                    "  {:<20} {}{}",
                    "Recommendation".dimmed(),
                    colored_rating,
                    analysts_str.dimmed()
                );
            }

            let target_low = quote.target_low_price.as_ref().and_then(|v| v.raw);
            let target_mean = quote.target_mean_price.as_ref().and_then(|v| v.raw);
            let target_high = quote.target_high_price.as_ref().and_then(|v| v.raw);

            if target_mean.is_some() {
                print_key_value_pair(
                    "Target Low",
                    target_low.map(|v| format!("{}{:.2}", currency_symbol, v)),
                    "Target Mean",
                    target_mean.map(|v| format!("{}{:.2}", currency_symbol, v)),
                    term_width,
                );
                if let (Some(mean), Some(current)) = (target_mean, price) {
                    let upside = ((mean - current) / current) * 100.0;
                    let upside_str = if upside >= 0.0 {
                        format!("+{:.1}% upside", upside).green()
                    } else {
                        format!("{:.1}% downside", upside).red()
                    };
                    print_key_value_pair(
                        "Target High",
                        target_high.map(|v| format!("{}{:.2}", currency_symbol, v)),
                        "vs Current",
                        Some(upside_str.to_string()),
                        term_width,
                    );
                }
            }
        }

        // Company info
        let has_company_info = quote.sector.is_some()
            || quote.industry.is_some()
            || quote.full_time_employees.is_some();
        if has_company_info {
            print_section_header("Company", term_width);
            print_key_value_pair(
                "Sector",
                quote.sector.clone(),
                "Industry",
                quote.industry.clone(),
                term_width,
            );

            if let Some(employees) = quote.full_time_employees {
                print_key_value_pair(
                    "Employees",
                    Some(format_number(employees)),
                    "Currency",
                    Some(currency),
                    term_width,
                );
            }

            if let Some(website) = &quote.website {
                print_single_value("Website", website, term_width);
            }
        }

        // Description
        if let Some(desc) = &quote.long_business_summary {
            print_section_header("About", term_width);
            print_wrapped_text(desc, term_width);
        }
    }

    println!();
    Ok(())
}

// ===== Helper functions for rich display =====

fn terminal_width() -> usize {
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80)
}

fn print_header(symbol: &str, name: &str, exchange: &str, width: usize) {
    let header = format!("{} - {}", symbol.bold().cyan(), name);
    println!("  {}", header);
    println!("  {}", exchange.dimmed());
    println!("  {}", "─".repeat(width.min(60)));
}

fn print_section_header(title: &str, _width: usize) {
    println!();
    println!("  {} {}", "■".cyan(), title.bold());
}

fn print_price_line(
    price: Option<f64>,
    change: Option<f64>,
    change_pct: Option<f64>,
    currency_symbol: &str,
) {
    let price_str = price
        .map(|p| format!("{}{:.2}", currency_symbol, p))
        .unwrap_or_else(|| "N/A".to_string());

    let change_str = match (change, change_pct) {
        (Some(c), Some(p)) => {
            let pct = p * 100.0; // Convert decimal to percentage
            let sign = if c >= 0.0 { "+" } else { "" };
            let formatted = format!("{}{:.2} ({}{:.2}%)", sign, c, sign, pct);
            if c >= 0.0 {
                formatted.green()
            } else {
                formatted.red()
            }
        }
        _ => "".normal(),
    };

    println!("  {} {}", price_str.bold().white(), change_str);
}

fn print_range_bar(
    label: &str,
    low: f64,
    high: f64,
    current: f64,
    currency_symbol: &str,
    width: usize,
) {
    let bar_width = (width.saturating_sub(40)).min(30);
    let range = high - low;

    if range <= 0.0 {
        return;
    }

    let position =
        ((current - low) / range * bar_width as f64).clamp(0.0, bar_width as f64 - 1.0) as usize;

    // Build the bar as a string directly to avoid byte indexing issues
    let mut bar_parts: Vec<&str> = Vec::with_capacity(bar_width);
    for i in 0..bar_width {
        if i == position {
            bar_parts.push("●");
        } else {
            bar_parts.push("─");
        }
    }

    let left: String = bar_parts[..position].join("");
    let right: String = if position + 1 < bar_width {
        bar_parts[position + 1..].join("")
    } else {
        String::new()
    };

    println!(
        "  {:<11} {}{:.2} {}{}{}  {}{:.2}",
        label.dimmed(),
        currency_symbol.dimmed(),
        low,
        left.dimmed(),
        "●".cyan().bold(),
        right.dimmed(),
        currency_symbol.dimmed(),
        high
    );
}

fn print_key_value_pair(
    label1: &str,
    value1: Option<String>,
    label2: &str,
    value2: Option<String>,
    _width: usize,
) {
    let v1 = value1.unwrap_or_else(|| "—".to_string());
    let v2 = value2.unwrap_or_else(|| "—".to_string());
    println!(
        "  {:<20} {:<15}  {:<20} {}",
        label1.dimmed(),
        v1,
        label2.dimmed(),
        v2
    );
}

fn print_single_value(label: &str, value: &str, _width: usize) {
    println!("  {:<20} {}", label.dimmed(), value);
}

fn print_wrapped_text(text: &str, width: usize) {
    let max_width = width.saturating_sub(4).min(76);
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut line = String::new();

    for word in words {
        if line.len() + word.len() + 1 > max_width {
            println!("  {}", line.dimmed());
            line = word.to_string();
        } else {
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
    }

    if !line.is_empty() {
        println!("  {}", line.dimmed());
    }
}

/// Format volume with abbreviations (K, M, B)
fn format_volume(volume: i64) -> String {
    match volume {
        v if v >= 1_000_000_000 => format!("{:.2}B", v as f64 / 1_000_000_000.0),
        v if v >= 1_000_000 => format!("{:.2}M", v as f64 / 1_000_000.0),
        v if v >= 1_000 => format!("{:.2}K", v as f64 / 1_000.0),
        _ => volume.to_string(),
    }
}

/// Format market cap with abbreviations (M, B, T)
fn format_market_cap(market_cap: i64) -> String {
    match market_cap {
        v if v >= 1_000_000_000_000 => format!("{:.2}T", v as f64 / 1_000_000_000_000.0),
        v if v >= 1_000_000_000 => format!("{:.2}B", v as f64 / 1_000_000_000.0),
        v if v >= 1_000_000 => format!("{:.2}M", v as f64 / 1_000_000.0),
        _ => market_cap.to_string(),
    }
}

/// Format percentage (input is decimal, e.g., 0.25 -> 25.00%)
fn format_percent(value: f64) -> String {
    format!("{:.2}%", value * 100.0)
}

/// Format growth with sign and color indicator
fn format_growth(value: f64) -> String {
    let pct = value * 100.0;
    if pct >= 0.0 {
        format!("+{:.2}%", pct)
    } else {
        format!("{:.2}%", pct)
    }
}

/// Format large numbers with commas
fn format_number(n: i64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

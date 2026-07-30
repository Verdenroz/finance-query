use super::chart::render_chart;
use super::lookup::render_lookup;
use super::news::render_news;
use super::portfolio::render_portfolio;
use super::screeners::render_screeners;
use super::util::{format_market_cap, format_volume, is_overnight_session};
use crate::dashboard::state::{App, Tab};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub(super) fn render_details(f: &mut Frame, app: &App, area: Rect) {
    match app.active_tab {
        Tab::Watchlist => render_symbol_details(f, app, area),
        Tab::Charts => render_chart(f, app, area),
        Tab::News => render_news(f, app, area),
        Tab::Lookup => render_lookup(f, app, area),
        Tab::Screeners => render_screeners(f, app, area),
        Tab::Sectors => {} // Sectors tab renders full-width, not here
        Tab::Portfolio => render_portfolio(f, app, area),
        Tab::Alerts => {} // Alerts tab renders full-width, not here
    }
}

fn render_symbol_details(f: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    let Some(symbol) = app.selected_symbol() else {
        let paragraph = Paragraph::new(vec![
            Line::from(Span::styled(
                "No symbol selected",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from("Use j/k to navigate watchlist"),
            Line::from("Press 'a' to add a symbol"),
        ])
        .block(Block::default().borders(Borders::ALL).title("Details"));
        f.render_widget(paragraph, area);
        return;
    };

    let quote = app.quotes.get(symbol);

    let (price, change, change_pct, day_low, day_high, volume, streaming_market_hours) =
        if let Some(update) = app.price_updates.get(symbol) {
            (
                update.price as f64,
                update.change as f64,
                update.change_percent as f64,
                Some(update.day_low as f64),
                Some(update.day_high as f64),
                update.day_volume,
                Some(update.market_hours),
            )
        } else if let Some(q) = quote {
            let p = q
                .regular_market_price
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0);
            let c = q
                .regular_market_change
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0);
            let cp = q
                .regular_market_change_percent
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0);
            let dl = q.regular_market_day_low.as_ref().and_then(|v| v.raw);
            let dh = q.regular_market_day_high.as_ref().and_then(|v| v.raw);
            let vol = q
                .regular_market_volume
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0);
            (p, c, cp, dl, dh, vol, None)
        } else {
            let paragraph = Paragraph::new(vec![
                Line::from(Span::styled(
                    symbol,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Loading quote data...",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(Block::default().borders(Borders::ALL).title("Details"));
            f.render_widget(paragraph, area);
            return;
        };

    let change_sign = if change >= 0.0 { "+" } else { "" };
    let change_color = if change >= 0.0 {
        Color::Green
    } else {
        Color::Red
    };

    let mut left_lines = vec![];
    // Use streaming market_hours when available, otherwise fall back to quote's market_state
    let status = if let Some(market_hours) = streaming_market_hours {
        use finance_query::streaming::MarketHoursType;
        match market_hours {
            MarketHoursType::RegularMarket => ("● LIVE", Color::Green),
            MarketHoursType::PreMarket => {
                // Distinguish overnight (8 PM - 4 AM ET) from pre-market (4 AM - 9:30 AM ET)
                if is_overnight_session() {
                    ("◐ OVERNIGHT", Color::Cyan)
                } else {
                    ("◐ PRE", Color::Yellow)
                }
            }
            MarketHoursType::PostMarket => ("◑ POST", Color::Yellow),
            MarketHoursType::ExtendedHoursMarket => ("◑ EXT", Color::Yellow),
        }
    } else if let Some(q) = quote {
        match q.market_state.as_deref().unwrap_or("CLOSED") {
            "REGULAR" => ("● OPEN", Color::Green),
            "PRE" => ("◐ PRE", Color::Yellow),
            "POST" => ("◑ POST", Color::Yellow),
            _ => ("○ CLOSED", Color::DarkGray),
        }
    } else {
        ("○ CLOSED", Color::DarkGray)
    };

    left_lines.push(Line::from(vec![
        Span::styled(
            symbol,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {}", status.0), Style::default().fg(status.1)),
    ]));

    if let Some(q) = quote
        && let Some(name) = q.short_name.clone().or_else(|| q.long_name.clone())
    {
        left_lines.push(Line::from(Span::styled(
            name,
            Style::default().fg(Color::DarkGray),
        )));
    }

    left_lines.push(Line::from(""));
    left_lines.push(Line::from(vec![
        Span::styled(
            format!("${:.2}", price),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled(
            format!(
                "{}{:.2} ({}{:.2}%)",
                change_sign, change, change_sign, change_pct
            ),
            Style::default()
                .fg(change_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Pre/Post market data right after main price
    if let Some(q) = quote {
        let market_state = q.market_state.as_deref().unwrap_or("");
        if market_state == "PRE" {
            if let Some(pre_price) = q.pre_market_price.as_ref().and_then(|v| v.raw) {
                let pre_pct = q
                    .pre_market_change_percent
                    .as_ref()
                    .and_then(|v| v.raw)
                    .unwrap_or(0.0);
                let pre_color = if pre_pct >= 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };
                left_lines.push(Line::from(vec![
                    Span::styled("Pre-Mkt ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("${:.2} ", pre_price)),
                    Span::styled(
                        format!("({:+.2}%)", pre_pct),
                        Style::default().fg(pre_color),
                    ),
                ]));
            }
        } else if market_state == "POST"
            && let Some(post_price) = q.post_market_price.as_ref().and_then(|v| v.raw)
        {
            let post_pct = q
                .post_market_change_percent
                .as_ref()
                .and_then(|v| v.raw)
                .unwrap_or(0.0);
            let post_color = if post_pct >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            left_lines.push(Line::from(vec![
                Span::styled("After   ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("${:.2} ", post_price)),
                Span::styled(
                    format!("({:+.2}%)", post_pct),
                    Style::default().fg(post_color),
                ),
            ]));
        }
    }

    left_lines.push(Line::from(""));

    if let Some(q) = quote {
        if let Some(o) = q.regular_market_open.as_ref().and_then(|v| v.raw) {
            left_lines.push(Line::from(vec![
                Span::styled("Open        ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("${:.2}", o)),
            ]));
        }
        if let Some(pc) = q.regular_market_previous_close.as_ref().and_then(|v| v.raw) {
            left_lines.push(Line::from(vec![
                Span::styled("Prev Close  ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("${:.2}", pc)),
            ]));
        }
    }
    if let (Some(low), Some(high)) = (day_low, day_high) {
        left_lines.push(Line::from(vec![
            Span::styled("Day Range   ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("${:.2} - ${:.2}", low, high)),
        ]));
    }
    left_lines.push(Line::from(vec![
        Span::styled("Volume      ", Style::default().fg(Color::DarkGray)),
        Span::raw(format_volume(volume)),
    ]));

    if let Some(q) = quote {
        if let (Some(low), Some(high)) = (
            q.fifty_two_week_low.as_ref().and_then(|v| v.raw),
            q.fifty_two_week_high.as_ref().and_then(|v| v.raw),
        ) {
            left_lines.push(Line::from(vec![
                Span::styled("52W Range   ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("${:.2} - ${:.2}", low, high)),
            ]));
        }
        if let Some(avg_vol) = q.average_volume.as_ref().and_then(|v| v.raw) {
            let vol_ratio = if avg_vol > 0 {
                volume as f64 / avg_vol as f64
            } else {
                0.0
            };
            let vol_indicator = if vol_ratio > 1.5 {
                " ▲"
            } else if vol_ratio < 0.5 {
                " ▼"
            } else {
                ""
            };
            left_lines.push(Line::from(vec![
                Span::styled("Avg Volume  ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{}{}", format_volume(avg_vol), vol_indicator)),
            ]));
        }

        // 52 Week Change
        if let Some(w52_change) = q.week_52_change.as_ref().and_then(|v| v.raw) {
            let w52_color = if w52_change >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };
            left_lines.push(Line::from(vec![
                Span::styled("52W Change  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:+.2}%", w52_change * 100.0),
                    Style::default().fg(w52_color),
                ),
            ]));
        }

        // Bid/Ask spread
        if let (Some(bid), Some(ask)) = (
            q.bid.as_ref().and_then(|v| v.raw),
            q.ask.as_ref().and_then(|v| v.raw),
        ) && bid > 0.0
            && ask > 0.0
        {
            left_lines.push(Line::from(vec![
                Span::styled("Bid/Ask     ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("${:.2} / ${:.2}", bid, ask)),
            ]));
        }

        // Shares outstanding and float
        if let Some(shares) = q.shares_outstanding.as_ref().and_then(|v| v.raw) {
            left_lines.push(Line::from(vec![
                Span::styled("Shares Out  ", Style::default().fg(Color::DarkGray)),
                Span::raw(format_volume(shares)),
            ]));
        }
        if let Some(float) = q.float_shares.as_ref().and_then(|v| v.raw) {
            left_lines.push(Line::from(vec![
                Span::styled("Float       ", Style::default().fg(Color::DarkGray)),
                Span::raw(format_volume(float)),
            ]));
        }

        // Short ratio (days to cover)
        if let Some(short_ratio) = q.short_ratio.as_ref().and_then(|v| v.raw) {
            let sr_color = if short_ratio > 5.0 {
                Color::Red // High short interest
            } else if short_ratio > 2.0 {
                Color::Yellow
            } else {
                Color::White
            };
            left_lines.push(Line::from(vec![
                Span::styled("Short Ratio ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.1} days", short_ratio),
                    Style::default().fg(sr_color),
                ),
            ]));
        }

        // Price to Book
        if let Some(pb) = q.price_to_book.as_ref().and_then(|v| v.raw) {
            left_lines.push(Line::from(vec![
                Span::styled("P/B         ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{:.2}", pb)),
            ]));
        }

        // Ex-Dividend Date
        if let Some(ex_div) = q.ex_dividend_date.as_ref().and_then(|v| v.raw) {
            use chrono::{TimeZone, Utc};
            if let Some(date) = Utc.timestamp_opt(ex_div, 0).single() {
                let today = Utc::now().date_naive();
                let ex_date = date.date_naive();
                let is_upcoming = ex_date >= today;
                let date_color = if is_upcoming {
                    Color::Yellow
                } else {
                    Color::DarkGray
                };
                left_lines.push(Line::from(vec![
                    Span::styled("Ex-Div Date ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        date.format("%Y-%m-%d").to_string(),
                        Style::default().fg(date_color),
                    ),
                ]));
            }
        }
    }

    let trading_height = columns[0].height.saturating_sub(2) as usize; // Account for borders
    let trading_content_len = left_lines.len();
    let trading_max_scroll = trading_content_len.saturating_sub(trading_height);
    let trading_scroll = if trading_max_scroll > 0 {
        (app.detail_scroll[0] as usize) % (trading_max_scroll + 1)
    } else {
        0
    };
    let trading_scroll_indicator = if trading_max_scroll > 0 {
        format!(" [{}↕]", trading_content_len)
    } else {
        String::new()
    };
    let left_paragraph = Paragraph::new(left_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Trading{}", trading_scroll_indicator)),
        )
        .scroll((trading_scroll as u16, 0));
    f.render_widget(left_paragraph, columns[0]);

    let mut right_lines = vec![];
    if let Some(q) = quote {
        right_lines.push(Line::from(Span::styled(
            "─ Valuation ─",
            Style::default().fg(Color::DarkGray),
        )));
        if let Some(mcap) = q.market_cap.as_ref().and_then(|v| v.raw) {
            right_lines.push(Line::from(vec![
                Span::styled("Mkt Cap   ", Style::default().fg(Color::DarkGray)),
                Span::raw(format_market_cap(mcap)),
            ]));
        }
        if let Some(pe) = q.trailing_pe.as_ref().and_then(|v| v.raw) {
            right_lines.push(Line::from(vec![
                Span::styled("P/E TTM   ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{:.2}", pe)),
            ]));
        }
        if let Some(fwd_pe) = q.forward_pe.as_ref().and_then(|v| v.raw) {
            right_lines.push(Line::from(vec![
                Span::styled("P/E Fwd   ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{:.2}", fwd_pe)),
            ]));
        }
        if let Some(eps) = q.trailing_eps.as_ref().and_then(|v| v.raw) {
            right_lines.push(Line::from(vec![
                Span::styled("EPS TTM   ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("${:.2}", eps)),
            ]));
        }
        if let Some(div_yield) = q.dividend_yield.as_ref().and_then(|v| v.raw) {
            right_lines.push(Line::from(vec![
                Span::styled("Div Yld   ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{:.2}%", div_yield * 100.0)),
            ]));
        }
        if let Some(beta) = q.beta.as_ref().and_then(|v| v.raw) {
            right_lines.push(Line::from(vec![
                Span::styled("Beta      ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{:.2}", beta)),
            ]));
        }

        let has_ma = q.fifty_day_average.as_ref().and_then(|v| v.raw).is_some()
            || q.two_hundred_day_average
                .as_ref()
                .and_then(|v| v.raw)
                .is_some();
        if has_ma {
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(Span::styled(
                "─ Technicals ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(ma50) = q.fifty_day_average.as_ref().and_then(|v| v.raw) {
                let vs_ma = if price > ma50 { "▲" } else { "▼" };
                right_lines.push(Line::from(vec![
                    Span::styled("50D MA    ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("${:.2} {}", ma50, vs_ma)),
                ]));
            }
            if let Some(ma200) = q.two_hundred_day_average.as_ref().and_then(|v| v.raw) {
                let vs_ma = if price > ma200 { "▲" } else { "▼" };
                right_lines.push(Line::from(vec![
                    Span::styled("200D MA   ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("${:.2} {}", ma200, vs_ma)),
                ]));
            }
        }

        let has_analyst = q.target_mean_price.as_ref().and_then(|v| v.raw).is_some()
            || q.recommendation_key.is_some();
        if has_analyst {
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(Span::styled(
                "─ Analysts ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(target) = q.target_mean_price.as_ref().and_then(|v| v.raw) {
                let upside = ((target - price) / price) * 100.0;
                let upside_color = if upside > 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };
                right_lines.push(Line::from(vec![
                    Span::styled("Target    ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("${:.2} ", target)),
                    Span::styled(
                        format!("({:+.1}%)", upside),
                        Style::default().fg(upside_color),
                    ),
                ]));
            }
            if let Some(rec) = &q.recommendation_key {
                let rec_color = match rec.to_lowercase().as_str() {
                    "buy" | "strongbuy" => Color::Green,
                    "hold" => Color::Yellow,
                    "sell" | "strongsell" => Color::Red,
                    _ => Color::White,
                };
                right_lines.push(Line::from(vec![
                    Span::styled("Rating    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(rec.to_uppercase(), Style::default().fg(rec_color)),
                ]));
            }
        }

        if q.sector.is_some() || q.industry.is_some() {
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(Span::styled(
                "─ Company ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(sector) = &q.sector {
                right_lines.push(Line::from(vec![
                    Span::styled("Sector    ", Style::default().fg(Color::DarkGray)),
                    Span::raw(sector.clone()),
                ]));
            }
            if let Some(industry) = &q.industry {
                right_lines.push(Line::from(vec![
                    Span::styled("Industry  ", Style::default().fg(Color::DarkGray)),
                    Span::raw(industry.clone()),
                ]));
            }
        }
    } else {
        right_lines.push(Line::from(Span::styled(
            "No fundamental data",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let fund_height = columns[1].height.saturating_sub(2) as usize;
    let fund_content_len = right_lines.len();
    let fund_max_scroll = fund_content_len.saturating_sub(fund_height);
    let fund_scroll = if fund_max_scroll > 0 {
        (app.detail_scroll[1] as usize) % (fund_max_scroll + 1)
    } else {
        0
    };
    let fund_scroll_indicator = if fund_max_scroll > 0 {
        format!(" [{}↕]", fund_content_len)
    } else {
        String::new()
    };
    let right_paragraph = Paragraph::new(right_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Fundamentals{}", fund_scroll_indicator)),
        )
        .scroll((fund_scroll as u16, 0));
    f.render_widget(right_paragraph, columns[1]);

    let mut third_lines = vec![];
    if let Some(q) = quote {
        let has_financials = q.total_revenue.as_ref().and_then(|v| v.raw).is_some()
            || q.profit_margins.as_ref().and_then(|v| v.raw).is_some();
        if has_financials {
            third_lines.push(Line::from(Span::styled(
                "─ Financials ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(rev) = q.total_revenue.as_ref().and_then(|v| v.raw) {
                third_lines.push(Line::from(vec![
                    Span::styled("Revenue   ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format_market_cap(rev)),
                ]));
            }
            if let Some(margin) = q.profit_margins.as_ref().and_then(|v| v.raw) {
                let margin_color = if margin > 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };
                third_lines.push(Line::from(vec![
                    Span::styled("Profit %  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.1}%", margin * 100.0),
                        Style::default().fg(margin_color),
                    ),
                ]));
            }
        }

        let has_efficiency = q.return_on_equity.as_ref().and_then(|v| v.raw).is_some()
            || q.return_on_assets.as_ref().and_then(|v| v.raw).is_some()
            || q.debt_to_equity.as_ref().and_then(|v| v.raw).is_some();
        if has_efficiency {
            if !third_lines.is_empty() {
                third_lines.push(Line::from(""));
            }
            third_lines.push(Line::from(Span::styled(
                "─ Efficiency ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(roe) = q.return_on_equity.as_ref().and_then(|v| v.raw) {
                let roe_color = if roe > 0.15 {
                    Color::Green
                } else if roe > 0.0 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                third_lines.push(Line::from(vec![
                    Span::styled("ROE       ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.1}%", roe * 100.0),
                        Style::default().fg(roe_color),
                    ),
                ]));
            }
            if let Some(roa) = q.return_on_assets.as_ref().and_then(|v| v.raw) {
                let roa_color = if roa > 0.05 {
                    Color::Green
                } else if roa > 0.0 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                third_lines.push(Line::from(vec![
                    Span::styled("ROA       ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.1}%", roa * 100.0),
                        Style::default().fg(roa_color),
                    ),
                ]));
            }
            if let Some(de) = q.debt_to_equity.as_ref().and_then(|v| v.raw) {
                let de_color = if de < 50.0 {
                    Color::Green
                } else if de < 100.0 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                third_lines.push(Line::from(vec![
                    Span::styled("D/E       ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{:.1}", de), Style::default().fg(de_color)),
                ]));
            }
        }

        let has_ownership = q
            .held_percent_insiders
            .as_ref()
            .and_then(|v| v.raw)
            .is_some()
            || q.held_percent_institutions
                .as_ref()
                .and_then(|v| v.raw)
                .is_some()
            || q.short_percent_of_float
                .as_ref()
                .and_then(|v| v.raw)
                .is_some();
        if has_ownership {
            if !third_lines.is_empty() {
                third_lines.push(Line::from(""));
            }
            third_lines.push(Line::from(Span::styled(
                "─ Ownership ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(insider) = q.held_percent_insiders.as_ref().and_then(|v| v.raw) {
                third_lines.push(Line::from(vec![
                    Span::styled("Insider % ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("{:.1}%", insider * 100.0)),
                ]));
            }
            if let Some(inst) = q.held_percent_institutions.as_ref().and_then(|v| v.raw) {
                third_lines.push(Line::from(vec![
                    Span::styled("Instit %  ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("{:.1}%", inst * 100.0)),
                ]));
            }
            if let Some(short_pct) = q.short_percent_of_float.as_ref().and_then(|v| v.raw) {
                let short_color = if short_pct > 0.20 {
                    Color::Red
                } else if short_pct > 0.10 {
                    Color::Yellow
                } else {
                    Color::White
                };
                third_lines.push(Line::from(vec![
                    Span::styled("Short %   ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.1}%", short_pct * 100.0),
                        Style::default().fg(short_color),
                    ),
                ]));
            }
        }

        let has_growth = q.earnings_growth.as_ref().and_then(|v| v.raw).is_some()
            || q.revenue_growth.as_ref().and_then(|v| v.raw).is_some();
        if has_growth {
            if !third_lines.is_empty() {
                third_lines.push(Line::from(""));
            }
            third_lines.push(Line::from(Span::styled(
                "─ Growth ─",
                Style::default().fg(Color::DarkGray),
            )));
            if let Some(eg) = q.earnings_growth.as_ref().and_then(|v| v.raw) {
                let eg_color = if eg > 0.0 { Color::Green } else { Color::Red };
                third_lines.push(Line::from(vec![
                    Span::styled("Earnings  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:+.1}%", eg * 100.0),
                        Style::default().fg(eg_color),
                    ),
                ]));
            }
            if let Some(rg) = q.revenue_growth.as_ref().and_then(|v| v.raw) {
                let rg_color = if rg > 0.0 { Color::Green } else { Color::Red };
                third_lines.push(Line::from(vec![
                    Span::styled("Revenue   ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:+.1}%", rg * 100.0),
                        Style::default().fg(rg_color),
                    ),
                ]));
            }
        }
    }

    if third_lines.is_empty() {
        // Check if we're loading detailed quote for this symbol
        let is_loading = app.is_loading_detailed_quote
            && app.loading_detailed_symbol.as_ref() == Some(&symbol.to_string());

        if is_loading {
            third_lines.push(Line::from(Span::styled(
                "Loading detailed data...",
                Style::default().fg(Color::Yellow),
            )));
        } else {
            third_lines.push(Line::from(Span::styled(
                "Data not available",
                Style::default().fg(Color::DarkGray),
            )));
            third_lines.push(Line::from(Span::styled(
                "for this symbol",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    let growth_height = columns[2].height.saturating_sub(2) as usize;
    let growth_content_len = third_lines.len();
    let growth_max_scroll = growth_content_len.saturating_sub(growth_height);
    let growth_scroll = if growth_max_scroll > 0 {
        (app.detail_scroll[2] as usize) % (growth_max_scroll + 1)
    } else {
        0
    };
    let growth_scroll_indicator = if growth_max_scroll > 0 {
        format!(" [{}↕]", growth_content_len)
    } else {
        String::new()
    };
    let third_paragraph = Paragraph::new(third_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Growth & Ownership{}", growth_scroll_indicator)),
        )
        .scroll((growth_scroll as u16, 0));
    f.render_widget(third_paragraph, columns[2]);
}

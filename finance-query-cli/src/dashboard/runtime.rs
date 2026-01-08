use super::{input::handle_key_event, render::ui, state::App};
use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use finance_query::streaming::{PriceStream, PriceUpdate};
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use tokio::time::Duration;

pub async fn run_dashboard() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new().await?;

    let mut price_stream: Option<PriceStream> = None;
    if !app.current_watchlist.symbols.is_empty() {
        let symbols: Vec<&str> = app
            .current_watchlist
            .symbols
            .iter()
            .map(|s| s.as_str())
            .collect();
        match PriceStream::subscribe(&symbols).await {
            Ok(stream) => {
                price_stream = Some(stream);
                app.status_message = format!("Connected • {} symbols", symbols.len());
            }
            Err(e) => {
                app.status_message = format!("WebSocket error: {}", e);
            }
        }
    }

    let _ = app.fetch_all_quotes().await;
    let _ = app.refresh_details().await;
    let _ = app.refresh_portfolio_prices().await;

    let result = run_event_loop(&mut terminal, &mut app, price_stream).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    mut price_stream: Option<PriceStream>,
) -> Result<()> {
    // Optional handle for parallel sectors fetch task
    let mut sectors_task: Option<
        tokio::task::JoinHandle<Vec<(finance_query::SectorType, finance_query::Sector)>>,
    > = None;

    // Optional handle for detailed quote fetch task
    let mut detailed_quote_task: Option<
        tokio::task::JoinHandle<Option<(String, finance_query::Quote)>>,
    > = None;

    loop {
        terminal.draw(|f| ui(f, app))?;

        if app.should_quit {
            if let Some(task) = sectors_task.take() {
                task.abort();
            }
            if let Some(task) = detailed_quote_task.take() {
                task.abort();
            }
            break;
        }

        // Spawn detailed quote fetch if needed
        if app.is_loading_detailed_quote && detailed_quote_task.is_none() {
            if let Some(symbol) = app.loading_detailed_symbol.clone() {
                detailed_quote_task = Some(tokio::spawn(async move {
                    match finance_query::Ticker::new(&symbol).await {
                        Ok(ticker) => match ticker.quote(false).await {
                            Ok(quote) => Some((symbol, quote)),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    }
                }));
            }
        }

        // Spawn parallel sectors fetch if needed
        if app.is_loading_sectors && sectors_task.is_none() && app.sectors_data.is_empty() {
            app.status_message = "Loading all sectors...".to_string();
            sectors_task = Some(tokio::spawn(async {
                use finance_query::{SectorType, finance};
                use futures::future::join_all;

                let futures = SectorType::all().iter().map(|&sector_type| async move {
                    match finance::sector(sector_type).await {
                        Ok(sector) => Some((sector_type, sector)),
                        Err(_) => None,
                    }
                });

                join_all(futures).await.into_iter().flatten().collect()
            }));
        }

        tokio::select! {
            // Check if detailed quote task completed
            result = async {
                match &mut detailed_quote_task {
                    Some(task) => Some(task.await),
                    None => futures::future::pending().await,
                }
            }, if detailed_quote_task.is_some() => {
                detailed_quote_task = None;
                app.is_loading_detailed_quote = false;
                if let Some(Ok(Some((symbol, quote)))) = result {
                    app.quotes.insert(symbol, quote);
                }
                app.loading_detailed_symbol = None;
            },
            // Check if sectors task completed
            result = async {
                match &mut sectors_task {
                    Some(task) => Some(task.await),
                    None => futures::future::pending().await,
                }
            }, if sectors_task.is_some() => {
                sectors_task = None;
                if let Some(Ok(sectors)) = result {
                    for (sector_type, sector) in sectors {
                        app.sectors_data.insert(sector_type, sector);
                    }
                    app.is_loading_sectors = false;
                    app.status_message = format!("Loaded {} sectors", app.sectors_data.len());
                    app.last_update = Some(chrono::Local::now());
                }
            },
            price_update = async {
                match &mut price_stream {
                    Some(stream) => stream.next().await,
                    None => futures::future::pending::<Option<PriceUpdate>>().await,
                }
            } => {
                if let Some(update) = price_update {
                    app.price_updates.insert(update.id.clone(), update);
                    app.last_update = Some(chrono::Local::now());
                    app.check_alerts();
                }
            },
            _ = app.refresh_interval.tick() => {
                let _ = app.refresh_details().await;
                let _ = app.refresh_portfolio_prices().await;
            },
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                while event::poll(Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        handle_key_event(app, key).await?;
                        break;
                    }
                }
            },
        }
    }

    Ok(())
}

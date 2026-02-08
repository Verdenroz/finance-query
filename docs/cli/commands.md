# Command Reference

Use `fq --help` to see all commands, or `fq <command> --help` for detailed help on any command.

## Market Data Commands

### `quote`

Get current stock quotes with price, volume, and key metrics.

```bash
fq quote AAPL                  # Single symbol
fq quote AAPL MSFT GOOGL       # Multiple symbols
fq quote AAPL -o json          # JSON output
fq quote BTC-USD ETH-USD       # Crypto
```

**Options:**

- `-o, --format` - Output format (table, json, csv)
- `-v, --verbose` - Show more details

### `chart`

View historical OHLCV data with interactive TUI.

```bash
fq chart AAPL                  # Default (1 month, 1 day interval)
fq chart AAPL -i 1d -r 6mo    # 6 months, daily bars
fq chart AAPL -i 5m -r 1d     # Intraday (5 minute)
fq chart AAPL -o csv          # Export as CSV
```

**Options:**

- `-i, --interval` - 1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo
- `-r, --range` - 1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max
- `-o, --format` - Output format

### `stream`

Real-time price streaming via WebSocket.

```bash
fq stream AAPL TSLA NVDA       # Stream multiple symbols
```

Press `Ctrl+C` to stop.

### `market`

Market summary including indices, futures, and crypto.

```bash
fq market                      # Display market summary
fq market -o json              # JSON output
```

### `trending`

Trending symbols by region.

```bash
fq trending                    # Default US
fq trending --region ca        # Canada
```

### `indices`

World market indices.

```bash
fq indices                     # All major indices
fq indices -o json             # JSON output
```

### `sector`

Sector performance and companies.

```bash
fq sector technology           # Technology sector
fq sector healthcare           # Healthcare sector
fq sector --companies          # Show top companies in sector
fq sector energy --industries  # Show industries in sector
```

**Available sectors:** technology, financial-services, consumer-cyclical, communication-services, healthcare, industrials, consumer-defensive, energy, basic-materials, real-estate, utilities

**Options:**

- `--companies` - Show top companies in the sector
- `--industries` - Show industries within the sector
- `--all` - Show all details

### `screener`

Pre-built screeners for market analysis.

```bash
fq screener most-actives           # Most active symbols
fq screener day-gainers            # Top day gainers
fq screener day-losers             # Top day losers
fq screener growth-technology-stocks  # Growth tech stocks
fq screener most-shorted-stocks    # Most shorted stocks
fq screener -l 50                  # Limit to 50 results
```

**Screener types:**

- **Equity movers:** most-actives, day-gainers, day-losers, most-shorted-stocks
- **Growth/Value:** growth-technology-stocks, aggressive-small-caps, small-cap-gainers, undervalued-growth-stocks, undervalued-large-caps
- **Funds:** top-mutual-funds, solid-large-growth-funds, solid-midcap-growth-funds, conservative-foreign-funds, high-yield-bond, portfolio-anchors

**Options:**

- `-l, --limit` - Maximum number of results (default: 25)

## Company Information Commands

### `info`

Full company details: profile, stats, and financials.

```bash
fq info AAPL                   # Company info
fq info AAPL -o json           # JSON output
```

### `profile`

Company description, sector, industry, and employee count.

```bash
fq profile AAPL                # Company profile
```

### `financials`

Financial statements: income, balance sheet, and cash flow.

```bash
fq financials AAPL             # Latest financials
fq financials AAPL -o json     # JSON format
```

### `earnings`

Earnings history and estimates.

```bash
fq earnings AAPL               # Earnings data
```

### `news`

Recent company news.

```bash
fq news AAPL                   # Latest news
fq news AAPL -o json           # JSON format
```

### `recommendations`

Analyst buy/hold/sell ratings.

```bash
fq recommendations AAPL        # Analyst recommendations
```

### `holders`

Institutional and insider ownership.

```bash
fq holders AAPL                # Shareholding data
```

### `edgar`

Interactive SEC EDGAR filings browser (TUI). Browse company filings by symbol or search across all filings.

!!! note "EDGAR Email Required"
    SEC requires a contact email in the User-Agent header. Set via `--email` or `EDGAR_EMAIL` environment variable.

```bash
# Set email (required by SEC)
export EDGAR_EMAIL="user@example.com"

# Open TUI ready to search
fq edgar                                   # Start with empty search prompt

# Browse filings by company symbol
fq edgar AAPL                              # Browse all AAPL filings interactively

# Search across all filings from command line
fq edgar --search "artificial intelligence"  # Search all companies
fq edgar -s "climate risk" -f 10-K           # Search only 10-K filings
fq edgar -s "acquisition" --start-date 2024-01-01  # Date-filtered search
```

**Interactive Controls:**

- `↑/↓, j/k` - Navigate filings
- `←/→` - Previous/next page (search mode only)
- `PgUp/PgDn, Ctrl+d/u` - Page up/down within current page
- `Home/End, g/G` - Jump to top/bottom of current page
- `Enter, o` - Open filing in browser
- `f` - Cycle filters (10-K → 10-Q → 8-K → 4 → S-1 → DEF 14A → 10-K/A → 10-Q/A → S-3 → 20-F → All)
- `r` - Reset filter (show all filings)
- `/` - Search all filings (in-TUI search)
- `s` - Search by symbol (switch to symbol mode)
- `q, Esc, Ctrl+C` - Quit

**Symbol Mode Features:**

- Browse ~1,000 recent filings for a company
- Company metadata (name, CIK)
- Color-coded form types (10-K green, 10-Q cyan, 8-K yellow)
- Filing metadata: date, report date, size, accession number
- Filter by form type interactively

**Search Mode Features:**

- Full-text search across all SEC filings
- Pagination support - navigate through thousands of results (←/→ or n/p keys)
- Filter by form type (`-f 10-K`, `-f 10-Q`, etc.)
- Date range filtering (`--start-date`, `--end-date`)
- Interactive navigation of search results
- Page indicator shows current page and total matches
- Open any filing directly in browser

**Options:**

- `-s, --search` - Full-text search query (omit for symbol mode)
- `-f, --form-type` - Filter by form type (10-K, 10-Q, 8-K, 4, S-1, DEF 14A, etc.)
- `--start-date` - Start date for search (YYYY-MM-DD, search mode only)
- `--end-date` - End date for search (YYYY-MM-DD, search mode only)
- `-e, --email` - Contact email for SEC User-Agent (or set `EDGAR_EMAIL` env var)

### `facts`

Get XBRL company facts from SEC EDGAR.

```bash
# Set email (required by SEC)
export EDGAR_EMAIL="user@example.com"

fq facts AAPL                  # All key financial facts
fq facts AAPL --concept GrossProfit  # Specific concept
fq facts AAPL --unit USD       # Filter by unit
fq facts AAPL -o json          # JSON output
```

**Options:**

- `-c, --concept` - Specific XBRL concept (e.g., "Revenue", "Assets", "NetIncomeLoss")
- `-u, --unit` - Filter by unit (e.g., "USD", "shares")
- `-l, --limit` - Max data points per concept (default: 10)
- `-o, --format` - Output format (table, json, csv)

**Common concepts:** Revenue, GrossProfit, OperatingIncomeLoss, NetIncomeLoss, EarningsPerShareBasic, Assets, Liabilities, StockholdersEquity, CashAndCashEquivalentsAtCarryingValue

### `transcript`

Earnings call transcripts.

```bash
fq transcript AAPL             # Latest transcript
fq transcript AAPL --quarter 2024-Q1  # Specific quarter
```

### `grades`

Analyst upgrade/downgrade history.

```bash
fq grades AAPL                 # Rating changes
```

## Technical Analysis Commands

### `indicator`

Calculate 40+ technical indicators.

```bash
fq indicator AAPL                             # Interactive TUI (choose indicators)
fq indicator AAPL --indicator rsi:14          # RSI with period 14
fq indicator AAPL --indicator sma:20,50,200   # SMAs
fq indicator AAPL --indicator macd:12,26,9    # MACD
fq indicator AAPL --indicator bollinger:20,2  # Bollinger Bands

# Non-interactive with range/interval
fq indicator AAPL --indicator rsi:14 -i 1d -r 6mo
fq indicator AAPL --indicator sma:20,50,200 --no-tui -o csv
```

**Available indicators:**
sma, ema, rsi, macd, bollinger, atr, stochastic, adx, obv, vwap, cci, williamsr, stochrsi, psar, supertrend, mfi, ichimoku, donchian

**Options:**

- `--indicator <INDICATOR>` - Format: `name:param1,param2,...` (skips TUI)
- `-i, --interval` - Candle interval (1m, 5m, 15m, 1h, 1d, 1wk, 1mo)
- `-r, --range` - Historical range (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
- `--no-tui` - Skip interactive TUI (requires --indicator)
- `--latest` - Show only the latest value
- `-o, --format` - Output format (table, json, csv)

### `backtest`

Test trading strategies with performance metrics.

```bash
fq backtest AAPL               # Interactive backtest TUI
fq backtest AAPL --preset swing      # Use swing trading preset
fq backtest AAPL --preset trend      # Use trend following preset
fq backtest AAPL --json              # Output JSON instead of TUI
fq backtest AAPL --no-tui --preset aggressive  # Run preset without TUI
```

**Available presets:** swing, day, trend, mean-reversion, conservative, aggressive

**Options:**

- `-p, --preset` - Use a preset strategy
- `--json` - Output JSON instead of TUI
- `--no-tui` - Skip interactive TUI and run directly with preset

## Options & Dividends Commands

### `options`

Interactive options chain explorer (TUI).

```bash
fq options AAPL                # Latest expiration
fq options AAPL --expiration 2024-12-20  # Specific date
```

### `dividends`

Dividend payment history.

```bash
fq dividends AAPL              # Dividend history
```

### `splits`

Stock split history.

```bash
fq splits AAPL                 # Stock splits
```

## Utility Commands

### `lookup`

Search for symbols by name or keyword.

```bash
fq lookup Apple                # Search by name
fq lookup --type etf           # Search by type
```

### `hours`

Market hours and trading status.

```bash
fq hours                       # Check if markets open
fq hours NASDAQ                # Specific exchange
```

### `currencies`

Currency list and exchange rates.

```bash
fq currencies                  # All currencies
fq currencies EUR              # Specific currency
```

### `exchanges`

Supported stock exchanges.

```bash
fq exchanges                   # List all exchanges
```

### `alerts`

Price alerts with desktop notifications.

```bash
# Open alerts TUI
fq alerts

# Add alerts
fq alerts add AAPL price-above:200
fq alerts add TSLA price-below:150
fq alerts add AAPL change-above:5      # 5%+ change
fq alerts add TSLA volume-spike:2.0    # 2x volume

# List alerts
fq alerts list

# Check alerts (one-time)
fq alerts check

# Watch continuously
fq alerts watch

# Install as system service
fq alerts service install
fq alerts service status

# Manage alerts
fq alerts toggle <id>          # Enable/disable
fq alerts remove <id>          # Delete
fq alerts clear                # Delete all
```

### `dashboard`

Interactive market dashboard (TUI).

```bash
fq dashboard  # Launch dashboard
```

Keyboard shortcuts in dashboard:

- `j/k` or arrow keys - Navigate
- `Enter` - Select
- `q` - Quit
- `r` - Refresh
- `+/-` - Add/remove from watchlist

## Global Options

```bash
fq [OPTIONS] [COMMAND]

Options:
  -v, --verbose         Enable verbose logging
      --no-color        Disable colored output
  -h, --help            Print help
  -V, --version         Print version
```

## Output Formats

All commands support multiple output formats:

```bash
fq quote AAPL -o table         # Pretty table (default)
fq quote AAPL -o json          # JSON
fq quote AAPL -o csv           # CSV (comma-separated)
```

Export to file:

```bash
fq chart AAPL -r 1y -o csv > aapl_2024.csv
fq quote AAPL -o json > aapl.json
```

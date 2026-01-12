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

### `filings`

SEC filings (10-K, 10-Q, 8-K, etc).

```bash
fq filings AAPL                # Latest filings
```

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

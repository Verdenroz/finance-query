# Usage Examples

Real-world examples for common use cases.

## Stock Research

### Get a company overview

```bash
fq info AAPL
fq profile AAPL
fq financials AAPL
```

### Check analyst recommendations

```bash
fq recommendations AAPL
fq grades AAPL
```

### View earnings

```bash
fq earnings AAPL
fq transcript AAPL --quarter 2024-Q1
```

### Check insider holdings

```bash
fq holders AAPL
```

## Price Monitoring

### Track multiple stocks

```bash
fq quote AAPL MSFT GOOGL AMZN META
```

### Stream live prices

```bash
fq stream AAPL TSLA NVDA PLTR
```

### Compare stock performance

```bash
# Get 1-year charts for comparison
fq chart AAPL -r 1y -o csv > aapl_1y.csv
fq chart MSFT -r 1y -o csv > msft_1y.csv

# Then compare in your preferred tool
```

### Set price alerts

```bash
# Open alerts TUI
fq alerts

# Alert when Apple hits $200
fq alerts add AAPL price-above:200

# Alert when Tesla drops below $150
fq alerts add TSLA price-below:150

# Alert on 5%+ change
fq alerts add AAPL change-above:5

# Check all alerts
fq alerts list

# Monitor continuously in background
fq alerts watch
```

## Technical Analysis

### Calculate indicators for a stock

```bash
# Interactive TUI (choose indicators)
fq indicator AAPL

# Single indicator (non-interactive)
fq indicator AAPL --indicator rsi:14 --no-tui

# Multiple indicators
fq indicator AAPL --indicator sma:20,50,200 -i 1d -r 6mo --no-tui
fq indicator AAPL --indicator macd:12,26,9 --no-tui

# Bollinger Bands
fq indicator AAPL --indicator bollinger:20,2 -i 1d -r 3mo --no-tui
```

### Export indicator data

```bash
fq indicator AAPL --indicator rsi:14 -i 1d -r 1y -o csv --no-tui > aapl_indicators.csv
fq indicator AAPL --indicator sma:20,50,200 -o csv --no-tui > sma_analysis.csv
```

### Get latest indicator values

```bash
fq indicator AAPL --indicator rsi:14 --latest --no-tui
```

## Backtesting

### Test a simple strategy

```bash
# Interactive backtest TUI
fq backtest AAPL

# Use swing trading preset
fq backtest AAPL --preset swing

# Use trend following preset
fq backtest AAPL --preset trend

# Mean reversion strategy
fq backtest AAPL --preset mean-reversion
```

### Export results

```bash
# Get backtest results as JSON
fq backtest AAPL --preset swing --json > backtest_results.json

# Run without TUI
fq backtest AAPL --no-tui --preset aggressive
```

## Market Analysis

### Market overview

```bash
# Check if markets are open
fq hours

# View market summary
fq market

# Check technology sector performance
fq sector technology

# Healthcare sector with companies
fq sector healthcare --companies

# Top gainers/losers
fq screener day-gainers
fq screener day-losers

# Most active symbols
fq screener most-actives -l 50
```

### Trending analysis

```bash
# Trending stocks (US)
fq trending

# By region
fq trending --region ca  # Canada
fq trending --region gb  # UK
```

### Economic indices

```bash
# Major indices
fq indices

# S&P 500
fq quote ^GSPC

# Dow Jones
fq quote ^DJI

# Nasdaq
fq quote ^IXIC
```

## Options Trading

### Explore options

```bash
# View options chain (interactive TUI)
fq options AAPL

# Specific expiration
fq options AAPL --expiration 2024-12-20
```

### Dividend tracking

```bash
# Dividend history
fq dividends AAPL

# Stock splits
fq splits AAPL
```

## Data Export

### Export to CSV

```bash
# Historical price data
fq chart AAPL -r 1y -o csv > aapl_historical.csv

# Quote data
fq quote AAPL MSFT GOOGL -o csv > quotes.csv

# Indicator data
fq indicator AAPL --indicator rsi:14 -o csv --no-tui > indicators.csv
fq indicator AAPL --indicator sma:20,50,200 -o csv --no-tui > sma.csv
```

### Export to JSON

```bash
# Backtest results
fq backtest AAPL --preset swing --json > backtest_results.json

# Company info
fq info AAPL -o json > aapl_info.json
```

### Pipe to other tools

```bash
# Use jq to parse JSON
fq quote AAPL -o json | jq '.price'

# Use grep to filter CSV
fq chart AAPL -r 1y -o csv | grep "2024"

# Count lines in CSV
fq chart AAPL -r 1y -o csv | wc -l
```

## Dashboard Usage

### Launch and navigate

```bash
# Start dashboard
fq dashboard
```

**In the dashboard:**

- **View market summary** - See indices, futures, crypto
- **Manage watchlist** - `+` to add, `-` to remove symbols
- **View details** - Press Enter on a symbol for details
- **Refresh** - Press `r` to refresh data
- **Search** - Press `/` to search
- **Quit** - Press `q` to exit

## Automation

### Check alerts periodically

```bash
# Run alert check every 5 minutes
watch -n 300 'fq alerts check'

# Or use system service
fq alerts service install
```

### Generate daily reports

```bash
#!/bin/bash
# daily-report.sh

DATE=$(date +%Y-%m-%d)

# Quotes
fq quote AAPL MSFT GOOGL -o csv > reports/quotes_${DATE}.csv

# Market summary
fq market -o json > reports/market_${DATE}.json

# Top movers
fq screener day-gainers -o csv > reports/gainers_${DATE}.csv
fq screener day-losers -o csv > reports/losers_${DATE}.csv

echo "Daily report generated: reports/"
```

```bash
# Schedule with cron (daily at 9am)
0 9 * * * /path/to/daily-report.sh
```

## Troubleshooting

### Check version

```bash
fq --version
```

### Enable debug logging

```bash
RUST_LOG=debug fq quote AAPL
# or
fq quote AAPL --verbose
```

### Verify installation

```bash
which fq
fq --version
fq quote --help
```

### Common errors

```bash
# "Command not found"
# Make sure ~/.cargo/bin is in your PATH

# "Connection timeout"
# Check your internet connection
# Yahoo Finance API might be temporarily unavailable

# "Symbol not found"
# Use 'fq lookup' to find the correct symbol
```

# finance-query-mcp

MCP server for [finance-query](https://crates.io/crates/finance-query) — real-time stock quotes, charts, technical indicators, financials, options, SEC filings, and more as MCP tools.

Works with any MCP-compatible agent: Claude Code, Claude Desktop, Cursor, Windsurf, Zed, Continue, VS Code Copilot, and others.

## Hosted (Recommended)

A hosted instance runs at `https://finance-query.com/mcp`. No installation required — just add it to your client config.

> **Note:** The hosted instance runs with `EDGAR_EMAIL` and `FRED_API_KEY` pre-configured, so all tools are available.

### Claude Code

Add to `.mcp.json` in your project root:

```json
{
  "mcpServers": {
    "finance-query": {
      "type": "http",
      "url": "https://finance-query.com/mcp"
    }
  }
}
```

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "finance-query": {
      "type": "http",
      "url": "https://finance-query.com/mcp"
    }
  }
}
```

### Cursor

Add to `.cursor/mcp.json` in your project or `~/.cursor/mcp.json` globally:

```json
{
  "mcpServers": {
    "finance-query": {
      "type": "http",
      "url": "https://finance-query.com/mcp"
    }
  }
}
```

### Windsurf

Add to `~/.codeium/windsurf/mcp_config.json`:

```json
{
  "mcpServers": {
    "finance-query": {
      "type": "http",
      "url": "https://finance-query.com/mcp"
    }
  }
}
```

### VS Code (GitHub Copilot)

Add to `.vscode/mcp.json` in your project:

```json
{
  "servers": {
    "finance-query": {
      "type": "http",
      "url": "https://finance-query.com/mcp"
    }
  }
}
```

### Continue

Add to `.continue/config.yaml`:

```yaml
mcpServers:
  - name: finance-query
    type: http
    url: https://finance-query.com/mcp
```

### Zed

Add to `~/.config/zed/settings.json`:

```json
{
  "context_servers": {
    "finance-query": {
      "command": {
        "path": "cargo",
        "args": ["run", "-p", "finance-query-mcp", "--quiet"],
        "env": {
          "EDGAR_EMAIL": "your@email.com",
          "FRED_API_KEY": "your_fred_api_key"
        }
      }
    }
  }
}
```

> Zed runs MCP servers as local processes (stdio). Clone the repo and run from source, or build the binary with `cargo build -p finance-query-mcp --release` and point `"path"` at `target/release/fq-mcp`.

---

## Self-Hosting

If you'd prefer to run your own instance, the server supports both HTTP and stdio transports.

### Docker (recommended)

```bash
# HTTP transport (remote/VPS)
docker run -p 3000:3000 \
  -e MCP_TRANSPORT=http \
  -e EDGAR_EMAIL=your@email.com \
  -e FRED_API_KEY=your_fred_api_key \
  ghcr.io/verdenroz/finance-query/mcp:latest
```

Point your client at `http://localhost:3000/mcp`.

### From Source

```bash
# stdio (default — for local clients that launch the process)
cargo run -p finance-query-mcp --quiet

# HTTP
MCP_TRANSPORT=http cargo run -p finance-query-mcp --quiet
```

For stdio, point your client at the binary:

```json
{
  "mcpServers": {
    "finance-query": {
      "command": "cargo",
      "args": ["run", "-p", "finance-query-mcp", "--quiet"],
      "env": {
        "EDGAR_EMAIL": "your@email.com",
        "FRED_API_KEY": "your_fred_api_key"
      }
    }
  }
}
```

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `EDGAR_EMAIL` | Optional | Enables `get_edgar_*` tools. SEC requires a contact email in the User-Agent. |
| `FRED_API_KEY` | Optional | Enables `get_fred_series`. Free key at [fred.stlouisfed.org](https://fred.stlouisfed.org/docs/api/api_key.html). `get_treasury_yields` is keyless and always available. |

---

## Available Tools

### Quotes
| Tool | Description |
|------|-------------|
| `get_quote` | Current quote + full company data for a symbol |
| `get_quotes` | Batch quotes for multiple symbols |
| `get_recommendations` | Similar stocks and analyst ratings |
| `get_splits` | Historical stock split history |

### Charts & Sparklines
| Tool | Description |
|------|-------------|
| `get_chart` | Historical OHLCV candlestick data |
| `get_charts` | Batch OHLCV data for multiple symbols |
| `get_spark` | Lightweight close-price sparklines (faster than `get_charts`) |

### Fundamentals
| Tool | Description |
|------|-------------|
| `get_financials` | Income statement, balance sheet, or cash flow |
| `get_batch_financials` | Financials for multiple symbols |
| `get_holders` | Major, institutional, fund, or insider holders |
| `get_analysis` | Analyst recommendations, upgrades/downgrades, earnings estimates |

### Technical Indicators (42 indicators)
| Tool | Description |
|------|-------------|
| `get_indicators` | All 42 indicators for a symbol (SMA, EMA, RSI, MACD, Bollinger Bands, Ichimoku, etc.) |
| `get_batch_indicators` | Indicators for multiple symbols |

### Options
| Tool | Description |
|------|-------------|
| `get_options` | Options chain for a symbol (nearest or specific expiration) |

### Dividends
| Tool | Description |
|------|-------------|
| `get_dividends` | Dividend history + analytics (CAGR, avg payment) |
| `get_batch_dividends` | Dividends for multiple symbols |

### Market Data
| Tool | Description |
|------|-------------|
| `get_market_summary` | Major indices and currencies by region |
| `get_market_hours` | Market open/closed status by region |
| `get_trending` | Trending tickers by region |
| `get_indices` | World market indices (S&P 500, DAX, Nikkei, etc.) |
| `get_fear_and_greed` | CNN Fear & Greed Index (0–100) |
| `get_sector` | Sector data for any of the 11 GICS sectors |
| `get_industry` | Industry data by slug |

### Search & Discovery
| Tool | Description |
|------|-------------|
| `search` | Search stocks, ETFs, and companies by name or ticker |
| `lookup` | Type-filtered ticker discovery (equity, ETF, fund, index, etc.) |
| `screener` | Predefined screeners (most-actives, day-gainers, undervalued-growth, etc.) |
| `get_news` | News for a symbol or general market news |
| `get_feeds` | RSS/Atom feeds from Bloomberg, WSJ, FT, SEC, MarketWatch, and more |
| `get_transcripts` | Earnings call transcripts |

### Risk Analytics
| Tool | Description |
|------|-------------|
| `get_risk` | VaR (95/99%), Sharpe/Sortino/Calmar ratios, beta, max drawdown |

### Crypto (CoinGecko, no API key)
| Tool | Description |
|------|-------------|
| `get_crypto_coins` | Top N coins by market cap |

### FRED / US Treasury
| Tool | Description |
|------|-------------|
| `get_fred_series` | Any FRED time series (FEDFUNDS, CPI, GDP, UNRATE, etc.) |
| `get_treasury_yields` | US Treasury yield curve (1m–30y) for a given year |

### SEC EDGAR
| Tool | Description |
|------|-------------|
| `get_edgar_facts` | XBRL structured financial data for any public company |
| `get_edgar_submissions` | SEC filing history and company metadata |
| `get_edgar_search` | Full-text search across SEC filings |

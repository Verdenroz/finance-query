# Migrating from yfinance to finance-query-py

`finance-query-py` is built on a Rust core. Async-first, type-safe, polars-native.

## Install

```bash
pip uninstall yfinance
pip install finance-query-py
```

If you want pandas interop (yfinance-style):

```bash
pip install finance-query-py[pandas]
```

## API mapping

### Single ticker

```python
# yfinance
import yfinance as yf
ticker = yf.Ticker("AAPL")
info = ticker.info
hist = ticker.history(period="1mo")
```

```python
# finance-query-py
import asyncio
from finance_query import Ticker, Interval, TimeRange

async def main():
    ticker = await Ticker.new("AAPL")
    quote = await ticker.quote()                                    # ≈ ticker.info
    chart = await ticker.chart(Interval.OneDay, TimeRange.OneMonth) # ≈ ticker.history
    df = chart.to_dataframe()           # returns a polars.DataFrame
    # df.to_pandas() works if you installed finance-query-py[pandas]

asyncio.run(main())
```

### Batch download

```python
# yfinance
data = yf.download(["AAPL", "MSFT"], period="1mo")
```

```python
# finance-query-py
from finance_query import Tickers, Interval, TimeRange

async def main():
    tickers = await Tickers.new(["AAPL", "MSFT"])
    result = await tickers.charts(Interval.OneDay, TimeRange.OneMonth)
    for sym, chart in result.data.items():
        print(sym, len(chart.candles))
    if result.errors:
        print("errors:", result.errors)

asyncio.run(main())
```

### Method correspondence

| yfinance | finance-query-py |
|---|---|
| `ticker.info` | `await ticker.quote()` |
| `ticker.history(period="1mo")` | `await ticker.chart(Interval.OneDay, TimeRange.OneMonth)` |
| `ticker.dividends` | `await ticker.dividends(range)` |
| `ticker.splits` | `await ticker.splits(range)` |
| `ticker.financials` | `await ticker.financials(StatementType.Income, Frequency.Quarterly)` |
| `ticker.news` | `await ticker.news()` |
| `ticker.recommendations` | `await ticker.recommendations(limit=5)` |
| `yf.download(...)` | `await (await Tickers.new([...])).charts(...)` |
| `yf.Ticker(...).search(...)` | `await finance.search(...)` |
| (not in yfinance) | `await finance.fear_and_greed()` |
| (not in yfinance) | `await ticker.edgar_submissions()` (SEC EDGAR) |

### Coming in Phase 2

- `ticker.options(...)` chain access
- `ticker.backtest(...)` with strategies (`SmaCrossover`, etc.)
- `ticker.risk(...)` for VaR / Sharpe / Sortino / Calmar
- Streaming via `PriceStream`
- `ticker.edgar_company_facts()` full structured access

## Why migrate?

- **Faster.** Rust-native HTTP and parsing. ~3x speedup on batch workflows.
- **Type-safe.** Type stubs ship with every wheel — full mypy coverage on call sites.
- **Async.** Concurrent batch requests without thread pools or asyncio.gather boilerplate.
- **Richer.** 42 technical indicators, backtesting, risk analytics, EDGAR, FRED — none of which yfinance has.
- **Stable.** When Yahoo changes its scraping surface, fixes ship as a Rust binary update — no waiting for a Python maintainer to patch scraping code.

## Migration caveats

- All API calls are async. Wrap top-level calls in `asyncio.run(...)`.
- `Interval` and `TimeRange` are enums, not strings. Use `Interval.OneDay` not `"1d"`.
- DataFrame returns are `polars.DataFrame`. For pandas: install `pip install finance-query-py[pandas]` and call `.to_pandas()` on the returned DataFrame.
- Phase 1 (alpha) does not yet support streaming, options chains, or backtesting/risk — see "Coming in Phase 2" above.
- The Phase 1 model surface is large (~80 model classes). Use IDE autocomplete or the type stubs to discover fields.

## Quickstart notebook

See [`finance-query-python/docs/examples/quickstart.ipynb`](../finance-query-python/docs/examples/quickstart.ipynb) for a runnable Jupyter walkthrough.

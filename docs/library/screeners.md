# Screeners

The screener API lets you filter stocks and mutual funds by hundreds of financial criteria. FinanceQuery supports both Yahoo Finance's **predefined screeners** and a fully **typed custom screener query builder** that gives you IDE autocomplete and compile-time field safety.

## Predefined Screeners

Yahoo Finance maintains a set of curated screeners. Pass a `Screener` variant and a result count to `finance::screener`:

```rust
use finance_query::{finance, Screener};

// Top 25 day gainers
let gainers = finance::screener(Screener::DayGainers, 25).await?;

// Most actively traded stocks
let actives = finance::screener(Screener::MostActives, 50).await?;

// Process results
for quote in &gainers.quotes {
    let change_pct = quote.regular_market_change_percent.raw.unwrap_or(0.0);
    println!("{}: {:+.2}%", quote.symbol, change_pct);
}
```

**Available `Screener` variants:**

| Variant | Description |
|---------|-------------|
| `DayGainers` | Top gaining stocks by % change |
| `DayLosers` | Top losing stocks by % change |
| `MostActives` | Highest trading volume stocks |
| `AggressiveSmallCaps` | High-risk small-cap stocks |
| `ConservativeForeignFunds` | Conservative international funds |
| `GrowthTechnologyStocks` | High-growth technology companies |
| `HighYieldBond` | High-yield bond funds |
| `MostShortedStocks` | Most heavily shorted stocks |
| `PortfolioAnchors` | Stable, large-cap anchor stocks |
| `SmallCapGainers` | Top small-cap gainers |
| `SolidLargeCap` | Established large-cap companies |
| `SolidMidcap` | Stable mid-cap companies |
| `TopMutualFunds` | Highest-rated mutual funds |
| `UndervaluedGrowthStocks` | Undervalued growth opportunities |
| `UndervaluedLargeCaps` | Undervalued large-cap companies |

## Custom Screeners — Typed Query Builder

The custom screener API uses typed field enums so your IDE can autocomplete field names and the compiler catches typos at build time.

### Core Types

| Type | Description |
|------|-------------|
| `EquityScreenerQuery` | Query for stocks (uses `EquityField`) |
| `FundScreenerQuery` | Query for mutual funds (uses `FundField`) |
| `EquityField` | ~80 typed field names for equity filters |
| `FundField` | Typed field names for fund filters |
| `ScreenerFieldExt` | Trait providing `.gt()`, `.lt()`, `.between()`, `.eq_str()` etc. |

### Basic Example

```rust
use finance_query::{finance, EquityScreenerQuery, EquityField, ScreenerFieldExt};

// Find US large-cap value stocks with healthy volume
let query = EquityScreenerQuery::new()
    .size(50)
    .sort_by(EquityField::IntradayMarketCap, false)   // descending
    .add_condition(EquityField::Region.eq_str("us"))
    .add_condition(EquityField::AvgDailyVol3M.gt(500_000.0))
    .add_condition(EquityField::IntradayMarketCap.gt(10_000_000_000.0))
    .add_condition(EquityField::PeRatio.between(10.0, 25.0));

let results = finance::custom_screener(query).await?;
println!("Found {} stocks", results.quotes.len());
```

### Condition Methods (`ScreenerFieldExt`)

Import `ScreenerFieldExt` to access the fluent condition builders on any field variant:

```rust
use finance_query::ScreenerFieldExt;

// Numeric comparisons
EquityField::PeRatio.gt(5.0)           // P/E > 5
EquityField::PeRatio.lt(30.0)          // P/E < 30
EquityField::PeRatio.gte(10.0)         // P/E >= 10
EquityField::PeRatio.lte(25.0)         // P/E <= 25
EquityField::PeRatio.eq_num(15.0)      // P/E == 15 (exact)
EquityField::PeRatio.between(10.0, 25.0)  // 10 <= P/E <= 25

// String equality (for categorical fields)
EquityField::Region.eq_str("us")
EquityField::Sector.eq_str("Technology")
EquityField::Exchange.eq_str("NMS")
```

### Controlling Results

```rust
let query = EquityScreenerQuery::new()
    .size(100)          // number of results (max 250, default 25)
    .offset(50)         // pagination offset
    .sort_by(EquityField::PeRatio, true)  // sort field + ascending=true
    .include_fields(vec![               // columns to return
        EquityField::Ticker,
        EquityField::CompanyShortName,
        EquityField::IntradayPrice,
        EquityField::PeRatio,
        EquityField::IntradayMarketCap,
    ]);
```

### OR Logic — `add_or_conditions`

All `add_condition` calls are AND'd together by default. Use `add_or_conditions` when you want any of several values to match:

```rust
// US OR Canadian equities, large-cap
let query = EquityScreenerQuery::new()
    .add_or_conditions(vec![
        EquityField::Region.eq_str("us"),
        EquityField::Region.eq_str("ca"),
    ])
    .add_condition(EquityField::IntradayMarketCap.gt(5_000_000_000.0));
```

### Preset Constructors

`EquityScreenerQuery` includes three built-in presets:

```rust
use finance_query::{EquityScreenerQuery, finance};

// Most shorted US stocks with average volume > 200K
let results = finance::custom_screener(EquityScreenerQuery::most_shorted()).await?;

// High-dividend US stocks (forward yield > 3%, volume > 100K)
let results = finance::custom_screener(EquityScreenerQuery::high_dividend()).await?;

// US large-cap growth stocks (market cap > $10B, positive EPS growth)
let results = finance::custom_screener(EquityScreenerQuery::large_cap_growth()).await?;
```

### Mutual Fund Screener

Use `FundScreenerQuery` with `FundField` for mutual funds:

```rust
use finance_query::{finance, FundScreenerQuery, FundField, ScreenerFieldExt};

let query = FundScreenerQuery::new()
    .size(25)
    .sort_by(FundField::PerformanceRating, false)
    .add_condition(FundField::RiskRating.lte(3.0))
    .include_fields(vec![
        FundField::Ticker,
        FundField::CompanyShortName,
        FundField::IntradayPrice,
        FundField::CategoryName,
        FundField::PerformanceRating,
        FundField::RiskRating,
    ]);

let results = finance::custom_screener(query).await?;
```

**Available `FundField` variants:**

| Variant | Yahoo Field | Description |
|---------|------------|-------------|
| `Ticker` | `"ticker"` | Ticker symbol (display only) |
| `CompanyShortName` | `"companyshortname"` | Fund name (display only) |
| `EodPrice` | `"eodprice"` | End-of-day price |
| `IntradayPrice` | `"intradayprice"` | Intraday price |
| `IntradayPriceChange` | `"intradaypricechange"` | Intraday price change |
| `CategoryName` | `"categoryname"` | Morningstar category |
| `PerformanceRating` | `"performanceratingoverall"` | Overall performance rating |
| `InitialInvestment` | `"initialinvestment"` | Minimum initial investment |
| `AnnualReturnRank` | `"annualreturnnavy1categoryrank"` | 1-year return rank within category |
| `RiskRating` | `"riskratingoverall"` | Overall risk rating |
| `Exchange` | `"exchange"` | Exchange |

## `EquityField` Reference

### Price & Market Cap

| Variant | Description |
|---------|-------------|
| `Ticker` | Ticker symbol (display only) |
| `CompanyShortName` | Company name (display only) |
| `EodPrice` | End-of-day price |
| `IntradayPrice` | Current intraday price |
| `IntradayPriceChange` | Intraday price change |
| `PercentChange` | Percentage price change |
| `Lastclose52WkHigh` | 52-week high at last close |
| `Lastclose52WkLow` | 52-week low at last close |
| `FiftyTwoWkPctChange` | 52-week percentage change |
| `IntradayMarketCap` | Current market capitalization |
| `LastcloseMarketCap` | Market cap at last close |

### Categorical (use `eq_str`)

| Variant | Example values |
|---------|---------------|
| `Region` | `"us"`, `"gb"`, `"jp"`, `"ca"` |
| `Sector` | `"Technology"`, `"Healthcare"`, `"Financials"` |
| `Industry` | `"Semiconductors"`, `"Software"` |
| `Exchange` | `"NMS"`, `"NYQ"`, `"ASE"` |
| `PeerGroup` | Peer group identifier |

### Trading & Volume

| Variant | Description |
|---------|-------------|
| `Beta` | Stock beta vs market |
| `AvgDailyVol3M` | 3-month average daily volume |
| `DayVolume` | Intraday volume |
| `EodVolume` | End-of-day volume |
| `PctHeldInsider` | % held by insiders |
| `PctHeldInst` | % held by institutions |

### Short Interest

| Variant | Description |
|---------|-------------|
| `ShortPctFloat` | Short % of float |
| `ShortPctSharesOut` | Short % of shares outstanding |
| `ShortInterest` | Short interest value |
| `DaysToCover` | Days to cover short |
| `ShortInterestPctChange` | Short interest % change |

### Valuation

| Variant | Description |
|---------|-------------|
| `PeRatio` | Trailing twelve-month P/E |
| `PegRatio5Y` | 5-year PEG ratio |
| `PriceBookRatio` | Price-to-book ratio |
| `PriceTangibleBook` | Price to tangible book value |
| `PriceEarnings` | Price to earnings (last close) |
| `BookValueShare` | Book value per share |
| `MarketCapToRevenue` | Market cap / total revenue |
| `TevToRevenue` | TEV / total revenue |
| `TevEbit` | TEV / EBIT |
| `TevEbitda` | TEV / EBITDA |

### Profitability & Dividends

| Variant | Description |
|---------|-------------|
| `Roa` | Return on assets (TTM) |
| `Roe` | Return on equity (TTM) |
| `ReturnOnCapital` | Return on total capital (TTM) |
| `ForwardDivYield` | Forward dividend yield |
| `ForwardDivPerShare` | Forward dividend per share |
| `ConsecutiveDivYears` | Consecutive years of dividend growth |
| `NetIncomeMargin` | Net income margin (TTM) |
| `GrossProfitMargin` | Gross profit margin (TTM) |
| `EbitdaMargin` | EBITDA margin (TTM) |

### Income Statement

| Variant | Description |
|---------|-------------|
| `TotalRevenues` | Total revenues (TTM) |
| `GrossProfit` | Gross profit (TTM) |
| `Ebitda` | EBITDA (TTM) |
| `Ebitda1YrGrowth` | EBITDA 1-year growth |
| `NetIncome` | Net income (TTM) |
| `NetIncome1YrGrowth` | Net income 1-year growth |
| `Revenue1YrGrowth` | Revenue 1-year growth |
| `QuarterlyRevGrowth` | Quarterly revenue growth |
| `EpsGrowth` | EPS growth (TTM) |
| `DilutedEps1YrGrowth` | Diluted EPS 1-year growth |
| `NetEpsBasic` | Basic EPS (TTM) |
| `NetEpsDiluted` | Diluted EPS (TTM) |
| `OperatingIncome` | Operating income (TTM) |
| `Ebit` | EBIT (TTM) |

### Balance Sheet & Leverage

| Variant | Description |
|---------|-------------|
| `TotalAssets` | Total assets (TTM) |
| `TotalDebt` | Total debt (TTM) |
| `TotalEquity` | Total equity (TTM) |
| `TotalCommonEquity` | Total common equity (TTM) |
| `TotalCurrentAssets` | Total current assets |
| `TotalCurrentLiab` | Total current liabilities |
| `CashAndStInvestments` | Cash and short-term investments |
| `CommonSharesOut` | Common shares outstanding |
| `TotalSharesOut` | Total shares outstanding |
| `TotalDebtEquity` | Total debt / equity |
| `LtDebtEquity` | Long-term debt / equity |
| `TotalDebtEbitda` | Total debt / EBITDA |
| `NetDebtEbitda` | Net debt / EBITDA |
| `EbitInterestExp` | EBIT / interest expense |
| `EbitdaInterestExp` | EBITDA / interest expense |

### Liquidity

| Variant | Description |
|---------|-------------|
| `QuickRatio` | Quick ratio (TTM) |
| `CurrentRatio` | Current ratio (TTM) |
| `AltmanZScore` | Altman Z-score |
| `OcfToCurrentLiab` | Operating cash flow / current liabilities |

### Cash Flow

| Variant | Description |
|---------|-------------|
| `CashFromOps` | Cash from operations (TTM) |
| `CashFromOps1YrGrowth` | Cash from ops 1-year growth |
| `LeveredFcf` | Levered free cash flow (TTM) |
| `LeveredFcf1YrGrowth` | Levered FCF 1-year growth |
| `UnleveredFcf` | Unlevered free cash flow |
| `Capex` | Capital expenditure (TTM) |

### ESG

| Variant | Description |
|---------|-------------|
| `EsgScore` | Overall ESG score |
| `EnvironmentalScore` | Environmental score |
| `GovernanceScore` | Governance score |
| `SocialScore` | Social score |
| `HighestControversy` | Highest controversy level |

## Next Steps

- [Finance Module](finance.md) - All market-wide functions including `screener()` and `custom_screener()`
- [Models Reference](models.md) - `ScreenerResults` and `ScreenerQuote` response types

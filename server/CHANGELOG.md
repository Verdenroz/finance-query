# Server Changelog

All notable changes to the `finance-query-server` Axum HTTP/WebSocket/GraphQL
server will be documented in this file. The server isn't published
independently (it's a deployed binary, not a crate) — its version number is
bumped in lockstep with the root [`CHANGELOG.md`](../CHANGELOG.md) via
`make bump`, so version numbers and dates here match that file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added

- **Financial event calendar** — `GET /v2/calendar?symbols=&range=` aggregates
  upcoming earnings, ex-dividend/dividend-payment dates, and standard monthly
  options expirations across one or more symbols into a single time-sorted
  list; with the `fred` feature, market-wide economic releases (CPI, NFP, GDP,
  …) are appended. Cached, GraphQL-bridged, and documented in `openapi.yaml`.
- **WebSocket streaming of RSS/Atom feed entries** (`server/src/handlers/feeds_stream.rs`),
  built on the library's new `NewsStream` primitive — mirrors the existing
  price-stream WS pattern but for pull-only feed polling, deduplicated by URL.
- **Relay-style cursor pagination** (`server/src/graphql/pagination.rs`) for
  every list-returning GraphQL field. Opt-in and backward compatible on REST
  (`limit`/`cursor` query params; omitted = full unpaginated response, the
  prior behavior); MCP always paginates to protect LLM context windows.

### Changed

- **Every REST and MCP data endpoint now routes through one typed GraphQL
  schema** (`server/src/graphql/`) instead of each transport hand-rolling its
  own response shaping. REST requests are bridged via the new
  `server/src/handlers/gql_bridge.rs`; MCP tools bridge to the same
  in-process schema (`finance-query-mcp/src/tools/gql.rs`) so both transports
  share identical field-selection and pagination behavior.
- **`fields` query param + `VALID_FIELDS` allow-lists** (`server/src/graphql/fields.rs`)
  bring GraphQL-style field selection to REST and MCP — comma-separated,
  exact-match-only field names validated against typed allow-lists per
  domain type (injection-safe), so callers can trim response payloads
  without a GraphQL client.
- **`server/src/main.rs` split** from a single 3500+ line file into `lib.rs`
  plus focused `handlers/<domain>.rs` modules (one per REST domain: quote,
  chart, financials, options, holders, analysis, edgar, calendar, screener,
  search, market, crypto, fred, feeds, indicators, risk, sector, transcripts,
  news, metadata, system, stream, feeds_stream) — no functional change to
  any individual route, but each handler is now independently readable and
  testable.
- **RSS/Atom feeds bridged through GraphQL**, retiring the old `apply_transforms`
  post-processing path in favor of the shared field-selection/pagination
  mechanism used by every other domain.
- **Four REST query params renamed snake_case → camelCase** to match
  GraphQL's existing convention (Rust field names unchanged): `start_date` →
  `startDate`, `end_date` → `endDate`, `vs_currency` → `vsCurrency`,
  `form_type` → `formType`. WebSocket commands were standardized the same
  way via `#[serde(rename_all = "camelCase")]`. **Breaking**: callers using
  the old snake_case params (EDGAR submissions date range, crypto quote
  currency, feeds form-type filter) must update to camelCase.
- Docker builds for both `server` and `finance-query-mcp` now use BuildKit
  cache mounts (`--mount=type=cache`) for the Cargo registry and target dir,
  so a trivial `Cargo.lock` churn (e.g. an internal workspace-member bump)
  no longer busts the compiled-dependency cache — notably preserving the
  from-source CTranslate2 build used by `translation-offline`, by far the
  most expensive part of the image build.

### Fixed

- **Debug-build stack overflow** in the holders/analysis GraphQL resolvers,
  caused by a multi-branch async closure; split into monomorphic per-type
  service functions (`server/src/services/holders.rs`, `services/analysis.rs`).
- Data-completeness gaps surfaced while auditing the new GraphQL schema
  against production: missing fields on the chart/news/market-summary/EDGAR
  submissions GraphQL types, an options-parsing correctness bug, and
  discovery/lookup enrichment gaps — all now match the REST responses they
  were bridged from.

## [2.7.1] - 2026-06-20

No server-specific code changes in this release — `server/Cargo.toml`,
`openapi.yaml`, and `asyncapi.yaml` were bumped in lockstep with the
library's version. The release's substantive changes (opt-in domain-handle
caching, `Tickers::spark()` provider routing, removal of the deprecated
`Fetch::All`) are internal to `src/` and don't change any server request or
response behavior.

### Security

No publicly known run-time vulnerabilities with a CVE or RUSTSEC assignment
were fixed in the server or its direct dependencies in this release.

## [2.7.0] - 2026-06-18

Bridges the library's two new offline-capable enrichment layers —
translation and sentiment scoring — into the server's request/response
surface. Both remain feature-gated and off by default at the library level,
but this build enables `sentiment` for the server and adds the `lang` query
param end-to-end.

### Added

- **`lang` query param** on quote, news (general + per-symbol), search, and
  market-summary endpoints (plus more via the OpenAPI `Lang` parameter
  ref), with `Accept-Language` header fallback when `lang` is omitted
  (`server/src/lang.rs`). The explicit query param always wins over the
  header; English, absent, and unparseable tags all resolve to no
  translation, and the resolved code feeds directly into cache keys so
  translated and untranslated responses for the same symbol are cached
  separately (`services::lang_key`, `services::translate` helpers in
  `server/src/services/mod.rs`).
- **GraphQL `lang` argument** added to `search`, `marketSummary`,
  `generalNews`, `Ticker.quote`, and `Ticker.news` resolvers
  (`server/src/graphql/query.rs`), resolving through the same
  `crate::lang::resolve_lang` path as REST (no `Accept-Language` fallback
  for GraphQL — the arg is explicit-only).
- **`sentiment` feature enabled by default in the server build**
  (`server/Cargo.toml`) — `News`/`SearchNews` articles and transcript
  paragraphs automatically gain a `sentiment` field (`Bullish`/`Neutral`/`Bearish`
  + score/confidence) on `/v2/news*`, `/v2/search`, and `/v2/transcripts*`,
  computed via the offline VADER lexicon (no API key, no model download).
- Docker image: HF model cache directory pre-created with correct
  `appuser` ownership and `HF_HOME` set, so a mounted volume persists
  downloaded translation models across deploys; `cmake`/`g++`/`make` added
  to the build stage and `libstdc++6`/`libgomp1` to the runtime stage for
  the CTranslate2-based offline translation backend.

### Changed

- **Server Docker images now build with `translation-offline` enabled**
  (`cargo build --features translation-offline`), replacing the prior
  "dictionary tier only" posture — production responses can now use the
  full local machine-translation backend, not just the built-in finance
  vocabulary dictionary.
- **NLLB-200 → opus-mt bilingual models** as the offline translation
  backend: instead of one ~600 MB multilingual model, each of ~48
  supported languages downloads its own small (~80–210 MB) opus-mt model
  lazily on first use, cached under `$HF_HOME/argos`. Cuts cold
  translation latency roughly in half and narrows the cross-language
  latency spread (measured ~40–62% faster per language on a transcript
  fixture). The `FINANCE_QUERY_TRANSLATION_MODEL` env var (NLLB-specific)
  is removed; offline language coverage drops from NLLB's ~200 languages to
  opus-mt's ~48, with unsupported targets degrading gracefully (dictionary
  tier still applies, free-form text stays English).

### Security

No publicly known run-time vulnerabilities with a CVE or RUSTSEC assignment
were fixed in the server or its direct dependencies in this release. The
offline translation backend downloads opus-mt models over HTTPS on first use
and caches them under `$HF_HOME`; no network call is made unless the server
is built with `translation-offline` and a non-English `lang` is requested.
Sentiment scoring is fully offline (bundled VADER lexicon) with no network
access.

## [2.6.1] - 2026-05-27

### Changed

- `services::quote::get_quote` updated for the library's `Quote<F: Format>` refactor (finance-query PR #166): the handler now explicitly requests `ticker.quote::<finance_query::format::Both>()` so `GET /v2/quote/{symbol}` keeps returning both raw and pretty-printed fields, unaffected by the library's new default format (`Raw`).

### Security

- Docker runtime image now runs `apt-get upgrade` on every build so OS-level packages (`libgnutls30`, `libkrb5support0`, `libgcrypt20`) receive available security patches regardless of the pinned base digest.
- Routine dependency maintenance via Dependabot: `axum` 0.8.7 → 0.8.9, `tower-http` 0.6.6 → 0.6.11, `redis` 1.0.2 → 1.2.1, `governor` 0.10.2 → 0.10.4, `thiserror` 2.0.17 → 2.0.18.

## [2.6.0] - 2026-05-21

The library's multi-provider `Providers`/`ProvidersBuilder` architecture landed this release (see root `CHANGELOG.md`), but the server itself was not migrated to it — it continues to run on the Yahoo-only fast path (`Ticker::new`/`Ticker::builder`). Only a small library API follow-up and Docker hardening were needed here.

### Changed

- `services::events::get_dividends` updated for the library's removal of `Ticker::dividend_analytics()`: analytics are now computed via the standalone `finance_query::DividendAnalytics::from_dividends(&dividends)` function. The `GET /v2/dividends/{symbol}` response shape (`dividends` + `analytics`) is unchanged.

### Security

- Docker build stage now installs `cargo-auditable` and builds the release binary via `cargo auditable build`, embedding its dependency manifest so a pulled image can be scanned later with `cargo audit bin`; build and runtime base images are pinned by digest (bumped going forward by Dependabot).

## [2.5.1] - 2026-05-06

Version bump only. This release's `Ticker::polygon()`/`Ticker::fmp()`/`Ticker::alphavantage()` adapter handles ([#133](https://github.com/Verdenroz/finance-query/pull/133)) are library-only additions — `server/` has no functional changes beyond the lockstepped version bump in `Cargo.toml`, `openapi.yaml`, and `asyncapi.yaml`.

## [2.5.0] - 2026-05-02

Version bump only. This release's Alpha Vantage, Polygon.io, and Financial Modeling Prep adapters ([#132](https://github.com/Verdenroz/finance-query/pull/132)) plus the `rand`/`rustls-webpki` RUSTSEC dependency fixes are entirely library-side — `server/` has no functional changes beyond the lockstepped version bump in `Cargo.toml`, `openapi.yaml`, and `asyncapi.yaml`.

## [2.4.3] - 2026-03-27

### Fixed

- `server/Dockerfile`'s dummy dependency-caching stage now stubs out `[[bench]]` targets (`indicators`, `backtesting`, `ticker`, `tickers`, `finance`, `stream`) and copies `benches/` into the build context. Cargo validates bench source paths during workspace resolution even when the benches aren't built, so the image build had started failing once those targets were declared in the root `Cargo.toml`.

## [2.4.2] - 2026-03-24

No server-specific changes in this release; version bumped in lockstep with the library.

## [2.4.1] - 2026-03-18

### Added

- **GraphQL API** (`async-graphql`): new `POST /graphql` query/mutation endpoint, `GET /graphql` interactive IDE, and a `graphql-ws` subscription endpoint for real-time price updates, running alongside the existing REST and WebSocket surfaces
  - `QueryRoot` resolvers and typed GraphQL models covering quotes, charts, market data, and news
  - `SubscriptionRoot` streams live price updates over the same connection semantics as `WS /v2/stream`
  - Query depth capped at 10 and complexity at 500 initially, then relaxed to depth 20 / complexity 2000 after real-world queries needed more headroom
  - The GraphiQL IDE replaced an earlier Apollo Sandbox/GraphQL Playground integration as the served UI at `GET /graphql`
- **`start`/`end` Unix timestamp params** on `GET /v2/chart/{symbol}` and the GraphQL `chart` field: when `start` is provided it overrides `range` and uses absolute date boundaries (`end` defaults to now); requesting `end` without `start` returns a 400
- `server/.env.template` and docs updated for the new GraphQL routes (`docs/server/graphql-api-reference.md`, `mkdocs.yml`, `Makefile` docs target)

### Changed

- **Major internal refactor**: business logic extracted out of the single `server/src/main.rs` (previously ~1550 lines) into a dedicated `server/src/services/` module (chart, quote, financials, market, search, risk, edgar, crypto, fred, feeds, news, options, holders, indicators, analysis, transcripts, events, metadata) — each service function returns `serde_json::Value`, `main.rs` now wires routes to services rather than containing the fetch/cache/serialize logic inline
- Default `RATE_LIMIT_PER_MINUTE` raised to 600
- `GET /v2/chart/{symbol}` and the GraphQL `chart` field both route through a consolidated chart service that handles the cached named-`range` path and the uncached absolute-date (`start`/`end`) path with a parallel events fetch

### Fixed

- OpenAPI spec corrected: `GET /v2/quote/{symbol}` now documents a `404` (not `400`) for a not-found symbol, and the batch capital-gains response schema's `capital_gains` key corrected to `capitalGains` to match the server's actual camelCase JSON output

## [2.4.0] - 2026-03-09

No server-specific changes in this release; version bumped in lockstep with the library (this release's substantial changes were confined to the backtesting engine in the core library — see root `CHANGELOG.md`).

## [2.3.0] - 2026-02-25

### Added

- **`GET /v2/fear-and-greed`** — CNN Fear & Greed index via alternative.me, keyless
- **`GET /v2/fred/series/{id}`** and **`GET /v2/fred/treasury-yields?year=<u32>`** — FRED macro time series and the US Treasury yield curve; both disabled unless `FRED_API_KEY` is configured (`server/.env.template` documents the FRED API Terms of Use obligation for public deployments)
- **`GET /v2/crypto/coins?vs_currency=usd&count=50`** and **`GET /v2/crypto/coins/{id}`** — CoinGecko top-N and single-coin market data, keyless
- **`GET /v2/feeds?sources=<csv>&form_type=<str>`** — RSS/Atom news feed aggregation across 30+ named sources
- **`GET /v2/risk/{symbol}?interval=&range=&benchmark=`** — VaR, Sharpe/Sortino/Calmar, beta, and max-drawdown risk analytics
- **Dividend analytics** injected into `GET /v2/dividends/{symbol}`: the response now returns `{"dividends": [...], "analytics": {...}}`, reusing the chart data already cached by the ticker rather than an extra fetch
- OpenAPI spec massively expanded to document the new typed custom-screener field surface: categorical, price/market-cap, trading/volume, short-interest, valuation, and profitability/dividend field groups for equities, replacing the old short illustrative field list

### Changed

- `POST /v2/screeners/custom` now dispatches to the library's typed `EquityScreenerQuery` / `FundScreenerQuery` builders (routed by `quote_type`) instead of the old stringly-typed `ScreenerQuery`, returning distinct 400 responses for an invalid field name vs. an invalid operator
- Updated for the library's `ScreenerType` → `Screener` and `SectorType` → `Sector` renames
- Path parameter and error-message naming made consistent: `/v2/screeners/{screener_type}` → `/v2/screeners/{screener}`, `/v2/sectors/{sector_type}` → `/v2/sectors/{sector}`, `/v2/industries/{industry_key}` → `/v2/industries/{industry}` (URL structure unchanged — this only renames the documented path-parameter identifier and adjusts error-message wording)

## [2.2.1] - 2026-02-21

### Added

- **`?patterns=true` query parameter** on `GET /v2/chart/{symbol}` and `GET /v2/charts`: injects a per-candle `patterns` array into the JSON response, computed via the library's new candlestick pattern recognition; `null` entries mean no pattern was detected on that bar, and the array aligns 1:1 with `candles` (single) or each chart's `candles` (batch).
- OpenAPI spec updated with the new query parameter and a 20-variant nullable string enum schema for pattern names.

## [2.2.0] - 2026-02-14

### Added

- **SEC EDGAR endpoints**: `GET /v2/edgar/cik/{symbol}`, `GET /v2/edgar/submissions/{symbol}`, `GET /v2/edgar/facts/{symbol}`, and `GET /v2/edgar/search` (full-text search with `q`, `forms`, `start_date`, `end_date` filters). All require the `EDGAR_EMAIL` environment variable; without it, the endpoints return `503 Service Unavailable` instead of initializing.
  - `from`/`size` pagination parameters added to `GET /v2/edgar/search` (offset + page size, capped at 100).
- **Eight new batch endpoints** mirroring the existing single-symbol routes: `GET /v2/charts`, `/v2/dividends`, `/v2/splits`, `/v2/capital-gains`, `/v2/financials`, `/v2/recommendations`, `/v2/options`, `/v2/indicators` — each accepts a comma-separated `symbols` list and returns the same success/error batch shape.
- **Rate limiting middleware**: a process-wide (not per-IP) token bucket via `governor`, configurable through `RATE_LIMIT_PER_MINUTE` (default 60 req/min). Requests over the limit get `429 Too Many Requests` with a `Retry-After` header and a `retryAfterSeconds` field in the JSON body.
- **Prometheus metrics endpoint** (`GET /v2/metrics`, text exposition format): HTTP request counts/latencies and an in-flight gauge, cache hit/miss counters, WebSocket connection/message/subscribed-symbol gauges, an error counter by error type, and a rate-limit-rejection counter — all namespaced under `finance_query_` so they group cleanly in Grafana.
- Market-open detection (used to decide cache TTL / bypass behavior) now checks against US Eastern time via `chrono-tz` instead of the server process's local timezone.

### Fixed

- Error and WebSocket metrics are now actually recorded — the error counter (by `FinanceError` variant), WebSocket message sent/received counters, and the subscribed-symbols gauge were registered but never incremented before this fix. Metrics registration is now idempotent (guarded by `Once`) instead of panicking on a second `init()` call, and the unused Yahoo-call counter/histogram were dropped since nothing populated them.
- Docker builder image pinned to `bookworm` for GLIBC compatibility.

### Changed

- Internal: EDGAR access moved off a per-`AppState` `Arc<EdgarClient>` onto the library's process-wide EDGAR singleton (`edgar::init_with_config`), removing the client field from `AppState` entirely — no behavior change for callers.
- Internal: `Ticker`/`Tickers` construction switched from `::new()` to the builder pattern (`Ticker::builder(symbol).logo().build()`) to track the library's removal of `include_logo` from `quote()`/`quotes()`; the `GET /v2/quote` and `GET /v2/quotes` `logo` query parameter behaves exactly as before.

## [2.1.0] - 2026-01-13

### Added

- **`GET /v2/spark?symbols=<csv>&interval=<str>&range=<str>`**: batch sparkline endpoint returning only close-price series for multiple symbols in a single request, optimized for watchlist-style rendering; results are cached and share the same success/error batch shape as the other batch endpoints.

### Fixed

- Server build now enables the `indicators` feature on its `finance-query` dependency (required after indicators moved into their own feature-gated module in the library), and the Docker dependency-caching layer now also stages a dummy `finance-query-cli` build — both needed to fix a broken release image build.

## [2.0.1] - 2025-12-31

### Changed

- OpenAPI and AsyncAPI specs updated to reference the newly hosted production API (`https://finance-query.com` for REST, `wss://finance-query.com/v2/stream` for the WebSocket feed) alongside the existing local development server entries.

## [2.0.0] - 2025-12-31

Initial release of the server. Everything below was built up incrementally over ~six weeks (Nov 2025 – Dec 2025) into the `finance-query-server` Axum binary; this entry summarizes the full surface as it stood at the v2.0.0 tag rather than listing each incremental commit.

### Added

- **REST API under `/v2/*`**, backed by the library's `AsyncTicker`/`Ticker` and `finance::` module:
  - Quotes: `GET /v2/quote/{symbol}` and `GET /v2/quotes?symbols=<csv>`, both with an optional `?logo=true` company-logo enrichment
  - Charts and indicators: `GET /v2/chart/{symbol}?interval=&range=`, `GET /v2/indicators/{symbol}?interval=&range=`
  - Corporate actions: `GET /v2/dividends/{symbol}`, `GET /v2/splits/{symbol}`, `GET /v2/capital-gains/{symbol}` (all `?range=`)
  - Fundamentals: `GET /v2/financials/{symbol}/{statement}?frequency=annual|quarterly` (income/balance/cashflow), `GET /v2/holders/{symbol}/{holder_type}` (major/institutional/mutualfund/insider-transactions/insider-purchases/insider-roster), `GET /v2/analysis/{symbol}/{analysis_type}` (recommendations/price-targets/upgrades-downgrades/earnings-estimate/earnings-history/revenue-estimate)
  - News, options, transcripts: `GET /v2/news`, `GET /v2/news/{symbol}`, `GET /v2/options/{symbol}?date=`, `GET /v2/recommendations/{symbol}?limit=`, `GET /v2/transcripts/{symbol}` and `GET /v2/transcripts/{symbol}/all`
  - Discovery: `GET /v2/search`, `GET /v2/lookup?q=&type=&count=&logo=&region=` (type-filtered symbol lookup, distinct from free-text search), `GET /v2/quote-type/{symbol}`
  - Market-wide data: `GET /v2/market-summary`, `GET /v2/trending?region=`, `GET /v2/indices`, `GET /v2/sectors/{sector_type}`, `GET /v2/industries/{industry_key}`, `GET /v2/currencies`, `GET /v2/exchanges`, `GET /v2/hours`
  - Screeners: `GET /v2/screeners/{screener_type}?count=` for predefined screeners (gainers/losers/actives, etc.), plus `POST /v2/screeners/custom` accepting a typed JSON body (`filters: [{field, operator, value}]` with `eq`/`gt`/`gte`/`lt`/`lte`/`btwn` operators, `sortField`, `sortType`, `size`/`offset`) for arbitrary screener queries
  - Health: `GET /v2/health`, `GET /v2/ping`
- **Real-time price streaming** — `GET /v2/stream` WebSocket endpoint with a JSON `{"subscribe": [...]}` / `{"unsubscribe": [...]}` protocol. A process-wide `StreamHub` maintains a single upstream Yahoo Finance stream shared across all connected clients and ref-counts per-symbol subscriptions, so a symbol is only subscribed once upstream no matter how many WebSocket clients are watching it.
- **`format` and `fields` query parameters**, supported on nearly every endpoint: `format` selects `raw` / `pretty` / `both` numeric formatting via the library's `ValueFormat`; `fields` is a comma-separated top-level JSON field allow-list applied after formatting, letting callers trim response payloads to just what they need.
- **Optional Redis-backed response caching** (`redis-cache` Cargo feature, enabled by default) — `server/src/cache.rs` wraps a `redis::aio::ConnectionManager` behind a `Cache` type, keyed with a `v2:` prefix to avoid clashing with the legacy Python v1 deployment sharing the same Redis instance. TTLs are per endpoint category and market-hours-aware (shorter while open, longer while closed) — e.g. quotes 10s/60s, indices 15s/180s, historical/chart/options/indicators 60s/600s, sectors 5m/1h, holders/analysis 1h flat, financials 24h flat, transcripts 7 days flat. Caching degrades to a transparent no-op when `REDIS_URL` is unset or the connection fails.
- **OpenAPI 3.0 spec** (`server/openapi.yaml`) and **AsyncAPI spec** (`server/asyncapi.yaml`) documenting the full REST and WebSocket surface.
- **Uniform error responses** — library error variants are mapped to HTTP status codes (`SymbolNotFound` → 404, `AuthenticationFailed` → 401, `RateLimited` → 429, `Timeout` → 408, `ServerError` → the upstream status code, everything else → 500) and returned as a consistent `{"error": ..., "status": ...}` JSON body.
- **Docker deployment** — multi-stage `Dockerfile` (dependency-cached build stage, slim `debian:bookworm-slim` runtime image, non-root `appuser`), a `HEALTHCHECK` against `/v2/health`, and `.env.template` documenting all runtime configuration (`PORT`, `LOG_LEVEL`/`LOG_FORMAT`, `DEFAULT_INTERVAL`/`DEFAULT_RANGE`, per-endpoint default limits, optional `REDIS_URL`).

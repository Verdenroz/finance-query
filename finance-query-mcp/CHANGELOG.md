# MCP Server Changelog

All notable changes to `fq-mcp`, the Model Context Protocol server exposing
`finance-query` to AI agents, will be documented in this file. Like the
server, it isn't published independently (it's a deployed binary, not a
crate) — its version number is bumped in lockstep with the root
[`CHANGELOG.md`](../CHANGELOG.md) via `make bump`, so version numbers and
dates here match that file. Versions with no MCP-relevant changes are
skipped entirely rather than listed as empty entries.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

Bridges every MCP tool through the same in-process GraphQL schema that powers the REST API, adding consistent field selection and pagination — and folding several paired batch tools into their singular counterparts along the way. **This includes a breaking change to the MCP tool surface**: six tools disappeared (see Changed below).

### Added

- **`get_calendar`** tool — aggregates upcoming earnings dates, ex-dividend/payment dates, and standard monthly options expirations across one or more symbols (`symbols`, `range`) into a single time-sorted list; with the `fred` feature, market-wide economic releases (CPI, NFP, GDP, …) are appended.
- **`fields` param** on most tools — comma-separated GraphQL field names, validated against the same typed `VALID_FIELDS` allow-lists REST uses (`server/src/graphql/fields.rs`); omitted falls back to a curated default response instead of the full object.
- **`limit`/`cursor` params** on every list-returning tool — MCP now always paginates (unlike REST, which stays unpaginated by default), so `get_feeds`, `get_news`, `get_dividends`, `get_chart`, `get_holders`, `search`, `get_quote`/`get_indicators` (multi-symbol), `get_options`, `get_fred_series`, `get_edgar_facts`, `get_edgar_submissions`, `get_treasury_yields`, and `get_crypto` all now return `{"items": [...], "pageInfo": {"hasNextPage": bool, "endCursor": "..."}}` even when the caller passes neither param.
- **`lang` param on quote/news/search tools** now also flows through the shared GraphQL layer for translated text fields (dictionary tier always on; ML-backed only when the deployed binary has `translation-offline` enabled).

### Changed

- **Breaking: paired batch tools were merged into their singular counterparts.** Each now accepts one *or more* comma-separated symbols in the existing single-symbol param, returning the flat single-item shape for one symbol and a `{items, errors}` batch shape for multiple:
  - `get_quotes` → folded into `get_quote` (`symbols` param)
  - `get_charts` → folded into `get_chart` (`symbols` param; `start`/`end` absolute timestamps remain single-symbol-only)
  - `get_batch_financials` → folded into `get_financials`
  - `get_batch_indicators` → folded into `get_indicators`
  - `get_batch_dividends` → folded into `get_dividends`
  - `get_crypto_coins` → renamed to `get_crypto` (no batch merge; same shape)
- Every tool's response now flows through the shared GraphQL schema (`finance-query-mcp/src/tools/gql.rs`) instead of hand-rolled fetch/serialize logic per tool, so MCP and REST responses stay structurally consistent going forward.
- `get_news` responses continue to auto-populate a `sentiment` field per article (VADER, from the `sentiment` feature) and `get_holders`/`get_analysis` now route through the resolvers that also received a debug-build stack-overflow fix on the server side (split multi-branch async closures into monomorphic per-type service functions), benefiting both transports.

### Fixed

- **`MCP_ALLOWED_HOSTS`** env var / `--allowed-hosts` CLI flag added to `fq-mcp`, merged with the always-allowed loopback hosts and passed to `StreamableHttpServerConfig::with_allowed_hosts`. The hosted server (`finance-query.com/mcp`) was returning 403 on every request because `rmcp`'s streamable HTTP transport defaults to loopback-only DNS-rebinding protection, and the public `Host` header forwarded through the Caddy reverse proxy was being rejected; default (unset) behavior is unchanged.

## [2.7.0] - 2026-06-18

### Added

- **Translation (`lang` param)** — text-bearing tools (quotes, batch quotes, news, search, and more) gain an optional `lang` param (BCP 47, e.g. `"ja"`, `"zh-Hant"`) that translates human-readable response fields; English or omitted means no translation. A built-in dictionary tier (sector names, security types, officer titles) always applies; free-form text (summaries, news titles, transcripts) additionally translates when the deployed binary is built with the optional `translation-offline` feature. The offline model is now preloaded in the background at startup (`tokio::spawn` in `main.rs`) so the first translated tool call after boot doesn't pay the cold-load cost.
- **Sentiment scoring on `get_news`** — the `sentiment` feature (offline VADER lexicon scoring, no API key/network) is now enabled in `finance-query-mcp/Cargo.toml`, so `get_news` results automatically gain a `Bullish`/`Neutral`/`Bearish` `sentiment` field per article with zero additional tool wiring.

### Changed

- The production Docker image now builds `fq-mcp` with `translation-offline` enabled (previously shipped dictionary-tier-only), swapping the offline backend from the 600 MB multilingual NLLB-200 model to per-language opus-mt bilingual models (~80–210 MB each, ~48 languages, downloaded lazily on first use and cached under `$HF_HOME/argos`) — roughly halving cold translation latency and narrowing the cross-language spread.

### Security

No publicly known run-time vulnerabilities with a CVE or RUSTSEC assignment were fixed in `finance-query-mcp` or its direct dependencies in this release. The offline translation backend downloads opus-mt models over HTTPS on first use and caches them locally; no network call is made unless `translation-offline` is enabled and a non-English language is requested.

## [2.6.1] - 2026-05-27

### Changed

- **`get_quote`/`get_quotes` responses now default to `Raw` format** (raw numeric values only) instead of `Both` (raw + pretty-formatted strings), matching the library-wide `Quote<F: Format>` default change.

### Security

No publicly known run-time vulnerabilities with a CVE or RUSTSEC assignment were fixed in `finance-query-mcp` or its direct dependencies in this release.

## [2.6.0] - 2026-05-21

### Changed

- **Docker image hardening**: the `fq-mcp` build now pins its base images (`rust:1-slim-bookworm`, `debian:bookworm-slim`) to a digest instead of a floating tag, and builds the release binary with `cargo auditable` so a pulled image can be scanned for known vulnerabilities via `cargo audit bin`.

### Fixed

- `get_dividends` now computes analytics directly from the already-fetched dividend list via `DividendAnalytics::from_dividends` instead of a separate `Ticker::dividend_analytics` call, avoiding redundant work with no change to the tool's response shape.

## [2.4.3] - 2026-03-27

### Fixed

- Docker image build was broken by the new backtesting/indicators `[[bench]]` targets registered in the workspace manifest for this release's perf work — Cargo validates bench source paths during workspace resolution even when a build only targets `fq-mcp`. The dummy-dependency-cache layer now stubs each bench target (`indicators`, `backtesting`, `ticker`, `tickers`, `finance`, `stream`) and the final build stage copies the real `benches/` directory before compiling.

## [2.4.1] - 2026-03-18

Initial release of `fq-mcp`, the Model Context Protocol server exposing `finance-query` to AI agents (Claude Code, Claude Desktop, Cursor, Windsurf, Zed, Continue, VS Code Copilot, and other MCP-compatible clients), hosted at `https://finance-query.com/mcp`.

### Added

- **Transport**: `stdio` by default (for local/editor integrations) or streamable HTTP (stateless, JSON responses) via `--transport http` / `MCP_TRANSPORT=http`, binding `MCP_ADDR` (default `0.0.0.0:3000`); ships with a `Dockerfile` and a `/health` endpoint for the HTTP deployment.
- **36 tools** covering the full library surface:
  - Quotes and corporate data: `get_quote`/`get_quotes`, `get_recommendations`, `get_splits`, `get_dividends`/`get_batch_dividends`, `get_holders`, `get_analysis`
  - Charts and technicals: `get_chart`/`get_charts`, `get_spark`, `get_indicators`/`get_batch_indicators`
  - Financials: `get_financials`/`get_batch_financials`
  - Discovery: `search`, `lookup`, `screener`
  - News and market context: `get_news`, `get_feeds`, `get_market_summary`, `get_fear_and_greed`, `get_trending`, `get_indices`, `get_market_hours`, `get_sector`, `get_industry`
  - SEC EDGAR: `get_edgar_facts`, `get_edgar_submissions`, `get_edgar_search` (require `EDGAR_EMAIL`)
  - Other analytics: `get_risk`, `get_crypto_coins`, `get_options`, `get_fred_series` (requires `FRED_API_KEY`), `get_treasury_yields` (keyless), `get_transcripts`
- **`get_chart` absolute date range**: optional `start`/`end` Unix-timestamp params that override `range` when set, routed through the same chunked `chart_range` path as the library and REST API.
- Optional env-gated integrations: `EDGAR_EMAIL` (SEC EDGAR) and `FRED_API_KEY` (FRED series; `get_treasury_yields` needs no key) — tools degrade gracefully with a warning at startup when unset rather than failing to boot.
- Auto-generated MCP tools reference page (`docs/server/mcp-reference.md`) built from the live tool/parameter definitions.

### Fixed

- HTTP transport routing behind the production Caddy reverse proxy: the service was first moved from `/mcp` to serving at the router root (`uri strip_prefix /mcp` in Caddy) to match the proxy's path rewriting, then switched from `nest_service("/", ...)` to `fallback_service(...)` so `/health` no longer gets swallowed by the MCP handler.

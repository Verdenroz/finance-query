# FMP catalog measurement — 2026-09-28

The local `finance-query` implementation was exercised with real account keys
through its public Rust APIs. No rows were ingested into Zaned. Keys were read
from the local environment and are absent from the artifacts.

## Fixed input

The 24 symbols were frozen in `examples/fmp_catalog_benchmark.rs` before the run:

`AAPL MSFT GOOGL GOOG AMZN META NVDA TSLA JPM V XOM JNJ WMT KO DIS BRK-B BABA TSM ASML ABNB SNOW UBER TWTR LEH`

This intentionally includes separate share classes, ADRs and delisted examples.
FMP spelling is used. The sample is not a random statistical estimate of the
whole market, and returned data was not used to choose the denominator.

## Results

| Operation | Requests | Elapsed | Result |
| --- | ---: | ---: | --- |
| Individual FMP profiles, concurrency 5 | 24 | 4.172 s | 23/24 returned; LEH not found |
| FMP stock directory | 1 | 16.043 s | 93,805 rows; 23/24 fixture symbols present |
| FMP active directory | 1 | 11.044 s | 70,271 rows; 22/24 fixture symbols present |
| FMP bulk part 0 | 1 | 180.004 s | Failed at configured transfer deadline; no successful result |
| FMP bulk part 3 | 1 | 180.042 s | Failed at configured transfer deadline; no successful result |
| Polygon 2026-09-28, first 10 CS active + first 10 inactive | 2 | 1.167 s | 10+10 records; both pages have continuations |
| Polygon 2003-09-10, same bounded queries | 2 | 0.933 s | Empty pages; historical coverage not established |

All 23 successful individual profiles had nonempty name, CIK, CUSIP, ISIN,
exchange and IPO date, plus active status. This is **95.83% fixture response
coverage**, and 23/24 overall presence for each listed field. It proves neither
identifier correctness nor historical identity. TWTR was returned with its
provider classification; it was absent from the active directory. LEH was
absent from both directories and the individual profile call.

Every directory row had a name; neither directory returned exchange data. They
are global lists, so these are not counts of US common stocks. Neither supplies
the identifiers or dates needed for a resolved Zaned stock row.

The current Polygon samples had composite FIGI on 6/10 active and 5/10 inactive
records. Unknown identity must therefore be an explicit collector outcome.
The empty 2003 pages cannot establish the earliest supported date or prove an
empty historical stock universe.

Bulk coverage is **unknown**, not zero. No complete part was returned. The raw
bulk JSON says `other_request_error` because the first executable grouped
timeout/network errors under that label; elapsed time equals its 180-second
timeout. The example now labels those error variants separately. A prior direct
format probe confirmed HTTP 200, `text/csv`, and the header fields; another
direct full transfer explicitly timed out at 90 seconds. Full live CSV decoding
remains unverified, while CSV/JSON parsing and failure handling passed mocked
HTTP tests.

## Method and limits

- Rust development build, macOS, ordinary local network, one run per operation.
  Timing starts after provider construction and includes fetch, decode and
  summary preparation; excludes compilation and JSON file output.
- Fresh provider instance per command, 300 configured requests/minute,
  180-second request timeout, no provider fallback, no configured retries.
  Ordinary profile concurrency was five; other modes were sequential.
- Counts are explicit one-request public operations, with mock tests confirming
  the one-call behavior; these are not provider billing-dashboard measurements.
  Ticker construction makes no network call on this configured path.
- The single/directory/Polygon probes overlapped the later portion of the part-0
  transfer. Part 3 ran without other provider downloads. Builds ran during some
  measurements. Treat timings as observed samples, not controlled median or
  production throughput claims. A transfer-byte metric was not captured by the
  public API benchmark.
- These artifacts contain 32 Rust API requests. Four preceding format/access
  preflight requests make 36 total provider requests for this task. Parts 1/2,
  full Polygon pagination and the full FMP delisted list were not downloaded.
- FMP's [FAQ](https://site.financialmodelingprep.com/faqs) requires profile-bulk
  calls to be spaced at least 60 seconds apart; both timed attempts met that.
  Its [part explanation](https://site.financialmodelingprep.com/it/faqs?code=marketPerformance)
  specifies parts 0 through 3. One part would not prove full coverage even if it
  succeeded. No live run here demonstrates a bulk speed advantage.

## Repeat

Export keys locally without committing them. From the finance-query root:

```sh
cargo run --locked --example fmp_catalog_benchmark --features 'polygon fmp' -- single
cargo run --locked --example fmp_catalog_benchmark --features 'polygon fmp' -- directories
cargo run --locked --example fmp_catalog_benchmark --features 'polygon fmp' -- bulk 0
cargo run --locked --example fmp_catalog_benchmark --features 'polygon fmp' -- polygon 2026-09-28
```

Run comparisons sequentially on an otherwise quiet connection; wait at least
60 seconds between bulk starts. Capture JSON and retain unsuccessful outcomes.
Keep the fixture unchanged. A future full-universe benchmark must enumerate all
four parts and all required directory pages, with explicit time/request bounds
and unresolved counts, before claiming provider-wide completeness.

For Zaned's initial bounded collector, use ordinary profiles for missing fields
and keep bulk opt-in until a complete transfer is measured successfully.

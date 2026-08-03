# OpenFIGI (identifier mapping)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["openfigi"] }
    ```

[OpenFIGI](https://www.openfigi.com/api) resolves a CUSIP, ISIN, SEDOL, or FIGI to the instruments carrying it — the missing step for any dataset that identifies holdings by CUSIP rather than ticker, 13F filings being the obvious case. No API key is required.

## Where It Lives

`openfigi` is a **crate-level module**, not a routed provider:

```rust
use finance_query::openfigi;
```

Identifier resolution is not tied to a symbol handle and maps onto no `Capability`, so it sits alongside [`edgar`](edgar.md) and [`fred`](fred.md) at the crate root rather than behind `Providers::builder()`.

## Resolving One Identifier

```rust
use finance_query::openfigi;

for listing in openfigi::resolve_cusip("037833100").await? {
    println!(
        "{:?} on {:?} ({:?})",
        listing.ticker, listing.exchange_code, listing.security_type
    );
}
```

`resolve_isin` and `resolve_sedol` are the same shape, and `resolve(SecurityIdKind::…, id)` takes the kind explicitly — including `SecurityIdKind::Figi` to find a FIGI's siblings and `SecurityIdKind::Ticker` to go the other way.

!!! note "One identifier, many instruments"
    A CUSIP or ISIN identifies a *security*, which trades on many venues, so resolution returns a **list**. Entries sharing a `composite_figi` are the same security on different venues; `share_class_figi` groups share classes across countries. Filter on `exchange_code == "US"` for the country composite.

An identifier that is well-formed but matches nothing returns an **empty list**. Only a malformed identifier is an error.

## Resolving Many

OpenFIGI accepts 10 identifiers per request without a key. `resolve_many` batches automatically:

```rust
use finance_query::openfigi::{self, SecurityIdKind};

let cusips = ["037833100", "594918104", "02079K305"];
let results = openfigi::resolve_many(SecurityIdKind::Cusip, &cusips).await?;

for (cusip, listings) in cusips.iter().zip(&results) {
    match listings.iter().find(|l| l.exchange_code.as_deref() == Some("US")) {
        Some(l) => println!("{cusip} -> {:?}", l.ticker),
        None => println!("{cusip} -> no US listing"),
    }
}
```

The result is **positional**: element `i` answers `ids[i]`, with an empty list where nothing matched. The adapter validates that OpenFIGI returned exactly as many results as jobs sent, and errors rather than risk pairing answers to the wrong identifiers.

## `SecurityMapping` Fields

| Field | Description |
|-------|-------------|
| `figi` | This instrument's Financial Instrument Global Identifier |
| `ticker` | Ticker symbol as the venue lists it |
| `name` | Security name, usually the issuer |
| `exchange_code` | `"US"` for the country composite, otherwise a venue code |
| `composite_figi` | FIGI of the country-level composite this rolls up to |
| `share_class_figi` | FIGI shared by every listing of this share class worldwide |
| `security_type` | e.g. `"Common Stock"`, `"ETP"` |
| `market_sector` | e.g. `"Equity"`, `"Corp"`, `"Govt"` |

## Rate Limits

Keyless: **25 requests per minute**, 10 identifiers per request. The client paces itself to stay inside that.

A free key from [openfigi.com/api](https://www.openfigi.com/api) raises both limits substantially. Export it and it is picked up automatically, per request, with no other change:

```bash
export OPENFIGI_API_KEY="your-key"
```

## Next Steps

- [EDGAR](edgar.md) — SEC filings, which identify holdings by CUSIP
- [Providers Overview](index.md) — the routed providers

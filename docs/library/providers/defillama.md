# DefiLlama (DeFi TVL)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["defi"] }
    ```

The library covered CeFi crypto — exchange quotes, aggregated market caps — but nothing on-chain. [DefiLlama](https://defillama.com/docs/api) is the FOSS standard for DeFi: keyless, no registration, stable endpoints.

## Two Surfaces

DefiLlama data splits by what it is *about*:

| Data | Where it lives | Why |
|------|---------------|-----|
| Protocol TVL and its history | `CryptoCoin::tvl()` / `.tvl_history()`, routed through `Capability::CRYPTO` | It is about one named thing, like a quote |
| Chain rankings, stablecoin supplies | `finance_query::defi::chains()` / `stablecoins()` | Market-wide; there is no symbol to hang it off |

## Protocol TVL

```rust
use finance_query::{Capability, Provider, Providers};

let providers = Providers::builder()
    .route(Capability::CRYPTO, [Provider::DefiLlama])
    .build()
    .await?;

let aave = providers.crypto("aave");
let tvl = aave.tvl().await?;

println!("{} — ${:?}", tvl.name.unwrap_or_default(), tvl.tvl);
println!("1d {:?}%  7d {:?}%", tvl.change_1d_percent, tvl.change_7d_percent);
for allocation in &tvl.tvl_by_chain {
    println!("  {}: ${:.0}", allocation.chain, allocation.tvl);
}

let history = aave.tvl_history().await?;   // oldest first
```

!!! warning "The handle id is a protocol slug, not a coin id"
    `providers.crypto("aave")` reads its id as a **DefiLlama protocol slug** for the TVL methods and as a coin id for `quote()`. Most slugs happen to match their CoinGecko id, but not all do — check [defillama.com](https://defillama.com/) for the slug in the protocol's URL. Ids are lowercased and hyphenated before use, so `"Curve DEX"` resolves to `curve-dex`.

!!! note "DefiLlama serves no prices"
    `fetch_crypto_quote` reports `NotSupported`, so a `CRYPTO` route that includes DefiLlama should also include an exchange or aggregator for `quote()` to resolve:
    `.route(Capability::CRYPTO, [Provider::DefiLlama, Provider::CoinGecko])`

### Per-chain allocations

DefiLlama's `currentChainTvls` mixes genuine chain keys (`"Ethereum"`) with breakdown keys of the *same* capital (`"Ethereum-borrowed"`, `"pool2"`, `"staking"`). Summing everything would double-count, so `tvl_by_chain` keeps only keys naming a chain the protocol actually reports, sorted largest first.

The headline `tvl` is the latest history snapshot, not a sum of those allocations.

### Change percentages

`change_1d_percent` and `change_7d_percent` are computed from the history against the closest snapshot **at or before** that instant. They are `None` when the protocol has no snapshot old enough to compare against — a new protocol reports no 7-day change rather than a fabricated one.

## Market-Wide Views

```rust
use finance_query::defi;

// Chains ranked by total value locked
for chain in defi::chains().await?.into_iter().take(10) {
    println!("{:<15} ${:?}", chain.name, chain.tvl);
}

// Stablecoins ranked by circulating supply
for coin in defi::stablecoins().await?.into_iter().take(10) {
    println!(
        "{:<10} {:?} ({:?})",
        coin.symbol.unwrap_or_default(),
        coin.circulating,
        coin.peg_mechanism
    );
}
```

Both come back sorted largest-first; DefiLlama itself returns them unordered.

!!! warning "Stablecoin supplies are denominated in the pegged asset"
    `circulating` is expressed in whatever `peg_type` names — `peggedUSD`, `peggedEUR`, and so on. Read `peg_type` before summing across coins, or you will add euros to dollars. `circulating_prev_day` / `_week` / `_month` are provided for change calculations.

## Rate Limits

DefiLlama publishes no quota for the free API but throttles abusive clients. The adapter paces at 5 requests/second, with a longer 45-second timeout than usual — the protocol and stablecoin payloads are large.

## Next Steps

- [CoinGecko](coingecko.md) — prices and market caps to pair with TVL
- [Crypto Domain](../crypto.md) — the `CryptoCoin` handle
- [Providers Overview](index.md) — routing and fallback

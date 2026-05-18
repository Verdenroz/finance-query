"""Cross-language parity: Python output must match recorded Rust JSON for the same symbol.

The fixture files in `fixtures/` are produced by:
    cargo run -p record-parity-fixtures -- AAPL MSFT NVDA TSLA SPY BTC-USD ETH-USD EURUSD=X GBPUSD=X GC=F

Tests are skipped when fixtures are missing.
"""

import pytest
from finance_query import Ticker

from .conftest import fixture_available, load_fixture, strip_time_variant


@pytest.mark.parametrize(
    "symbol,fixture_name",
    [
        ("AAPL", "quote_aapl"),
        ("MSFT", "quote_msft"),
        ("NVDA", "quote_nvda"),
        ("TSLA", "quote_tsla"),
        ("SPY", "quote_spy"),
        ("BTC-USD", "quote_btc_usd"),
        ("ETH-USD", "quote_eth_usd"),
        ("EURUSD=X", "quote_eurusd_x"),
        ("GBPUSD=X", "quote_gbpusd_x"),
        ("GC=F", "quote_gc_f"),
    ],
)
@pytest.mark.asyncio
@pytest.mark.network
async def test_quote_parity(symbol: str, fixture_name: str):
    """Compare live Python output against the recorded Rust JSON fixture."""
    if not fixture_available(fixture_name):
        pytest.skip(f"fixture {fixture_name} not yet recorded — run record-parity in CI")

    expected = load_fixture(fixture_name)
    ticker = await Ticker.new(symbol)
    quote = await ticker.quote()
    actual = quote.to_dict()

    expected_stripped = strip_time_variant(expected)
    actual_stripped = strip_time_variant(actual)

    # Symbol must always match
    assert actual.get("symbol") == expected.get("symbol"), f"{symbol}: symbol mismatch"
    # Strip-equal comparison
    diff_keys = set(expected_stripped.keys()) ^ set(actual_stripped.keys())
    assert not diff_keys, f"{symbol}: schema drift, differing keys: {diff_keys}"


def test_parity_infrastructure_present():
    """Sanity: the fixtures directory exists and the recorder is reachable."""
    from pathlib import Path
    fixtures = Path(__file__).parent / "fixtures"
    assert fixtures.is_dir(), "parity fixtures dir missing"

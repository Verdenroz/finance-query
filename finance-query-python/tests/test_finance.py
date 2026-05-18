"""Tests for the `finance_query.finance` submodule.

Covers presence/wiring of the top-level free functions and the
`FearGreedLabel` enum. Network-backed assertions are marked so they can be
skipped in CI runs that don't allow outbound HTTP.
"""

import pytest

from finance_query import FearGreedLabel, Region, Screener, finance


def test_finance_submodule_has_functions():
    expected = {"search", "screener", "trending", "fear_and_greed"}
    actual = {name for name in dir(finance) if not name.startswith("_")}
    missing = expected - actual
    assert not missing, f"finance submodule missing functions: {missing}"


def test_finance_submodule_exposes_response_types():
    # The wrapper response/quote types should be reachable via the submodule
    # so users can isinstance-check or read getters off them.
    expected = {"ScreenerResults", "ScreenerQuote", "SearchQuote", "TrendingQuote", "FearAndGreed"}
    actual = {name for name in dir(finance) if not name.startswith("_")}
    missing = expected - actual
    assert not missing, f"finance submodule missing types: {missing}"


def test_fear_greed_label_variants():
    # Sanity-check the 5 known variants are reachable as Python attributes.
    assert FearGreedLabel.ExtremeFear is not None
    assert FearGreedLabel.Fear is not None
    assert FearGreedLabel.Neutral is not None
    assert FearGreedLabel.Greed is not None
    assert FearGreedLabel.ExtremeGreed is not None
    # Equality should follow Python's enum semantics (eq_int on pyclass).
    assert FearGreedLabel.Fear == FearGreedLabel.Fear
    assert FearGreedLabel.Fear != FearGreedLabel.Greed


@pytest.mark.asyncio
@pytest.mark.network
async def test_search_returns_results():
    results = await finance.search("Apple")
    assert isinstance(results, list)


@pytest.mark.asyncio
@pytest.mark.network
async def test_screener_returns_results():
    res = await finance.screener(Screener.DayGainers, 5)
    # PyScreenerResults wraps Vec<ScreenerQuote>; expose .quotes getter.
    assert hasattr(res, "quotes")
    assert isinstance(res.quotes, list)


@pytest.mark.asyncio
@pytest.mark.network
async def test_trending_returns_results():
    results = await finance.trending(Region.UnitedStates)
    assert isinstance(results, list)


@pytest.mark.asyncio
@pytest.mark.network
async def test_trending_default_region():
    results = await finance.trending()
    assert isinstance(results, list)


@pytest.mark.asyncio
@pytest.mark.network
async def test_fear_and_greed_returns_value():
    fng = await finance.fear_and_greed()
    assert 0 <= fng.value <= 100
    assert fng.classification in {
        FearGreedLabel.ExtremeFear,
        FearGreedLabel.Fear,
        FearGreedLabel.Neutral,
        FearGreedLabel.Greed,
        FearGreedLabel.ExtremeGreed,
    }


def test_finance_submodule_has_all_functions():
    expected = {
        "search", "screener", "trending", "fear_and_greed",
        "lookup", "market_summary", "hours", "sector",
        "currencies", "news", "industry", "exchanges",
    }
    actual = {name for name in dir(finance) if not name.startswith("_")}
    assert expected.issubset(actual), f"missing: {expected - actual}"

"""Parity test fixtures — JSON files produced by `cargo run -p record-parity-fixtures`."""

import json
from pathlib import Path

FIXTURES_DIR = Path(__file__).parent / "fixtures"


def fixture_path(name: str) -> Path:
    """Path to a parity fixture (does NOT verify existence)."""
    return FIXTURES_DIR / f"{name}.json"


def load_fixture(name: str) -> dict:
    """Load a parity fixture; raises FileNotFoundError if missing."""
    return json.loads(fixture_path(name).read_text())


def fixture_available(name: str) -> bool:
    """Check whether a fixture file exists on disk."""
    return fixture_path(name).is_file()


# Time-variant fields excluded from strict equality comparisons.
TIME_VARIANT_FIELDS = {
    "current_price",
    "regular_market_change",
    "regular_market_change_percent",
    "regular_market_time",
    "regular_market_volume",
    "last_traded_at",
    "pre_market_time",
    "post_market_time",
}


def strip_time_variant(d):
    """Recursively strip time-varying fields from a dict/list for stable comparison."""
    if isinstance(d, dict):
        return {k: strip_time_variant(v) for k, v in d.items() if k not in TIME_VARIANT_FIELDS}
    if isinstance(d, list):
        return [strip_time_variant(x) for x in d]
    return d

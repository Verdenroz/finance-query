"""Tests for Python enum exposure of Rust constants."""

from finance_query import Interval, TimeRange, Frequency


def test_interval_variants_exist():
    assert Interval.OneMinute is not None
    assert Interval.OneDay is not None
    assert Interval.OneWeek is not None
    assert Interval.OneMonth is not None
    assert Interval.ThreeMonths is not None


def test_time_range_variants_exist():
    assert TimeRange.OneDay is not None
    assert TimeRange.OneMonth is not None
    assert TimeRange.OneYear is not None
    assert TimeRange.YearToDate is not None
    assert TimeRange.Max is not None


def test_frequency_variants_exist():
    assert Frequency.Annual is not None
    assert Frequency.Quarterly is not None


def test_enum_equality():
    assert Interval.OneDay == Interval.OneDay
    assert Interval.OneDay != Interval.OneMinute


def test_enum_repr():
    assert "OneDay" in repr(Interval.OneDay) or "Interval" in repr(Interval.OneDay)


from finance_query import (
    StatementType,
    Region,
    ValueFormat,
    Sector,
    Screener,
    ExchangeCode,
    Industry,
)


def test_statement_type_variants():
    assert StatementType.Income is not None
    assert StatementType.Balance is not None
    assert StatementType.CashFlow is not None


def test_value_format_variants():
    assert ValueFormat.Raw is not None
    assert ValueFormat.Pretty is not None
    assert ValueFormat.Both is not None


def test_region_country_level():
    assert Region.UnitedStates is not None
    assert Region.France is not None
    assert Region.Japan if hasattr(Region, "Japan") else Region.UnitedKingdom is not None


def test_sector_at_least_one_variant():
    assert Sector.Technology is not None
    assert len([attr for attr in dir(Sector) if not attr.startswith("_")]) > 5


def test_screener_at_least_one_variant():
    assert Screener.MostActives is not None
    assert len([attr for attr in dir(Screener) if not attr.startswith("_")]) > 3


def test_exchange_code_at_least_one_variant():
    assert ExchangeCode.Nms is not None
    assert len([attr for attr in dir(ExchangeCode) if not attr.startswith("_")]) > 3


def test_industry_at_least_one_variant():
    assert Industry.Semiconductors is not None
    assert len([attr for attr in dir(Industry) if not attr.startswith("_")]) > 3


from finance_query import Provider, SentimentLabel, FearGreedLabel


def test_enums_are_hashable():
    """Enum mirrors must be usable as set/dict members (require __hash__)."""
    # macro-generated mirror
    assert Interval.OneDay in {Interval.OneDay, Interval.OneMinute}
    # hand-written mirrors
    assert Provider.Yahoo in {Provider.Yahoo, Provider.Edgar}
    assert SentimentLabel.Bullish in {SentimentLabel.Bullish, SentimentLabel.Bearish}
    assert FearGreedLabel.Fear in {
        FearGreedLabel.ExtremeFear,
        FearGreedLabel.Fear,
        FearGreedLabel.Neutral,
        FearGreedLabel.Greed,
        FearGreedLabel.ExtremeGreed,
    }
    # usable as dict keys
    counts = {Interval.OneDay: 1, FearGreedLabel.Fear: 2}
    assert counts[Interval.OneDay] == 1

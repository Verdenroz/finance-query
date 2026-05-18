"""Smoke tests run against an installed wheel (not maturin develop build).

These verify the wheel imports cleanly and basic types are usable.
NO network calls — only structural checks on the loaded module.
"""

import finance_query


def test_module_version():
    assert isinstance(finance_query.__version__, str)
    assert finance_query.__version__


def test_enum_imports():
    assert finance_query.Interval.OneDay is not None
    assert finance_query.TimeRange.OneMonth is not None
    assert finance_query.Frequency.Annual is not None
    assert finance_query.StatementType.Income is not None


def test_error_imports():
    assert issubclass(finance_query.NetworkError, finance_query.FinanceQueryError)
    assert issubclass(finance_query.RateLimitError, finance_query.FinanceQueryError)
    assert issubclass(finance_query.SymbolNotFound, finance_query.FinanceQueryError)
    assert issubclass(finance_query.ParseError, finance_query.FinanceQueryError)
    assert issubclass(finance_query.ConfigError, finance_query.FinanceQueryError)


def test_ticker_class_exists():
    assert finance_query.Ticker is not None
    assert hasattr(finance_query.Ticker, "new")
    assert hasattr(finance_query.Ticker, "builder")


def test_tickers_class_exists():
    assert finance_query.Tickers is not None
    assert hasattr(finance_query.Tickers, "new")


def test_finance_submodule_exists():
    assert finance_query.finance is not None
    assert hasattr(finance_query.finance, "search")
    assert hasattr(finance_query.finance, "screener")
    assert hasattr(finance_query.finance, "fear_and_greed")


def test_enable_logging_callable():
    finance_query.enable_logging(level="WARN")

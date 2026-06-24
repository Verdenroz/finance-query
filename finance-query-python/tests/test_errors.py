"""Tests for the exception hierarchy and error mapping."""

import pytest
from finance_query import (
    FinanceQueryError,
    NetworkError,
    RateLimitError,
    SymbolNotFound,
    ParseError,
    ConfigError,
)


def test_exceptions_importable_and_subclass_base():
    assert issubclass(NetworkError, FinanceQueryError)
    assert issubclass(RateLimitError, FinanceQueryError)
    assert issubclass(SymbolNotFound, FinanceQueryError)
    assert issubclass(ParseError, FinanceQueryError)
    assert issubclass(ConfigError, FinanceQueryError)


def test_symbol_not_found_message_carries_symbol():
    # When raised from Python (no Rust mapping), the message is whatever was passed.
    try:
        raise SymbolNotFound("UNKNOWN-SYMBOL")
    except SymbolNotFound as e:
        assert "UNKNOWN-SYMBOL" in str(e)


def test_rate_limit_constructable():
    # Verify Python-side raising works; Rust-side population of retry_after
    # is exercised via integration tests in a later task.
    err = RateLimitError("rate limited")
    assert isinstance(err, FinanceQueryError)
    assert "rate limited" in str(err)

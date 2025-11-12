from unittest.mock import AsyncMock

import pytest
from fastapi import HTTPException

from src.models.earnings_transcript import EarningsCallListing
from src.services.earnings_transcript.get_earnings_transcript import (
    _parse_transcript,
    get_earnings_calls_list,
    get_earnings_transcript,
)


@pytest.fixture
def mock_finance_client():
    """Create a mock YahooFinanceClient"""
    return AsyncMock()


@pytest.fixture
def sample_earnings_calls_response():
    """Sample Yahoo Finance earnings calls list response"""
    return [
        {"eventId": "event123", "quarter": "Q4", "year": 2024, "title": "Q4 2024 Earnings Call", "url": "https://example.com/q4"},
        {"eventId": "event456", "quarter": "Q3", "year": 2024, "title": "Q3 2024 Earnings Call", "url": "https://example.com/q3"},
        {"eventId": "event789", "quarter": "Q2", "year": 2024, "title": "Q2 2024 Earnings Call", "url": "https://example.com/q2"},
    ]


@pytest.fixture
def sample_quote_type_response():
    """Sample Yahoo Finance quote type response with quartrId"""
    return {"quoteType": {"result": [{"quartrId": "company123", "symbol": "AAPL"}]}}


@pytest.fixture
def sample_transcript_response():
    """Sample Yahoo Finance transcript response"""
    return {
        "transcriptContent": {
            "speaker_mapping": [
                {"speaker": "1", "speaker_data": {"name": "Tim Cook", "role": "CEO", "company": "Apple Inc."}},
                {"speaker": "2", "speaker_data": {"name": "Luca Maestri", "role": "CFO", "company": "Apple Inc."}},
            ],
            "transcript": {
                "paragraphs": [
                    {"speaker": "1", "text": "Thank you for joining today's call. Our Q4 results were strong."},
                    {"speaker": "2", "text": "Revenue grew 8% year-over-year to $94.9 billion."},
                    {"speaker": "1", "text": "We're excited about our product pipeline for 2025."},
                ]
            },
        },
        "transcriptMetadata": {
            "fiscalYear": 2024,
            "fiscalPeriod": "Q4",
            "title": "Q4 2024 Earnings Call",
            "date": 1704067200,  # 2024-01-01
            "transcriptId": "transcript123",
            "eventType": "Earnings Call",
            "isLatest": True,
        },
    }


# Test parser function
def test_parse_transcript():
    """Test parsing transcript from Yahoo Finance response"""
    call_info = EarningsCallListing(event_id="event123", quarter="Q4", year=2024, title="Q4 2024 Earnings Call", url="https://example.com")

    transcript_data = {
        "transcriptContent": {
            "speaker_mapping": [
                {"speaker": "1", "speaker_data": {"name": "Tim Cook", "role": "CEO", "company": "Apple Inc."}},
                {"speaker": "2", "speaker_data": {"name": "Luca Maestri", "role": "CFO", "company": "Apple Inc."}},
            ],
            "transcript": {
                "paragraphs": [
                    {"speaker": "1", "text": "Thank you for joining."},
                    {"speaker": "2", "text": "Revenue grew 8%."},
                ]
            },
        },
        "transcriptMetadata": {
            "fiscalYear": 2024,
            "fiscalPeriod": "Q4",
            "title": "Q4 2024 Earnings Call",
            "date": 1704067200,
            "transcriptId": "transcript123",
            "eventType": "Earnings Call",
            "isLatest": True,
        },
    }

    result = _parse_transcript("AAPL", transcript_data, call_info)

    assert result.symbol == "AAPL"
    assert result.quarter == "Q4"
    assert result.year == 2024
    assert result.title == "Q4 2024 Earnings Call"
    assert len(result.speakers) == 2
    assert result.speakers[0].name == "Tim Cook"
    assert result.speakers[0].role == "CEO"
    assert len(result.paragraphs) == 2
    assert result.paragraphs[0].speaker == "Tim Cook"
    assert result.paragraphs[0].text == "Thank you for joining."
    assert result.metadata["eventId"] == "event123"
    assert result.metadata["isLatest"] is True


def test_parse_transcript_missing_speaker_metadata():
    """Test parsing transcript with missing speaker metadata"""
    call_info = EarningsCallListing(event_id="event123", quarter="Q1", year=2024, title="Q1 Call", url="https://example.com")

    transcript_data = {
        "transcriptContent": {
            "speaker_mapping": [
                {"speaker": "1", "speaker_data": {"name": "John Doe"}},  # Missing role and company
            ],
            "transcript": {"paragraphs": [{"speaker": "1", "text": "Hello everyone."}]},
        },
        "transcriptMetadata": {"fiscalYear": 2024, "fiscalPeriod": "Q1", "title": "Q1 Call"},
    }

    result = _parse_transcript("TEST", transcript_data, call_info)

    assert len(result.speakers) == 1
    assert result.speakers[0].name == "John Doe"
    assert result.speakers[0].role is None
    assert result.speakers[0].company is None


def test_parse_transcript_empty_paragraphs():
    """Test parsing transcript filters out empty paragraphs"""
    call_info = EarningsCallListing(event_id="event123", quarter="Q1", year=2024, title="Q1 Call", url="https://example.com")

    transcript_data = {
        "transcriptContent": {
            "speaker_mapping": [{"speaker": "1", "speaker_data": {"name": "Speaker 1"}}],
            "transcript": {
                "paragraphs": [
                    {"speaker": "1", "text": "Valid text."},
                    {"speaker": "1", "text": ""},  # Empty text
                    {"speaker": "1", "text": "Another valid text."},
                ]
            },
        },
        "transcriptMetadata": {},
    }

    result = _parse_transcript("TEST", transcript_data, call_info)

    # Should only have 2 paragraphs (empty one filtered out)
    assert len(result.paragraphs) == 2
    assert result.paragraphs[0].text == "Valid text."
    assert result.paragraphs[1].text == "Another valid text."


# Integration tests with mocked finance client


async def test_get_earnings_calls_list(mock_finance_client, sample_earnings_calls_response, bypass_cache):
    """Test getting earnings calls list"""
    mock_finance_client.get_earnings_calls_list.return_value = sample_earnings_calls_response

    result = await get_earnings_calls_list(mock_finance_client, "AAPL")

    assert result.symbol == "AAPL"
    assert result.total == 3
    assert len(result.earnings_calls) == 3
    # Should be sorted by year and quarter (most recent first)
    assert result.earnings_calls[0].quarter == "Q4"
    assert result.earnings_calls[0].year == 2024
    assert result.earnings_calls[0].event_id == "event123"


async def test_get_earnings_calls_list_empty(mock_finance_client, bypass_cache):
    """Test error handling when no earnings calls found"""
    mock_finance_client.get_earnings_calls_list.return_value = []

    with pytest.raises(HTTPException) as exc_info:
        await get_earnings_calls_list(mock_finance_client, "INVALID")

    assert exc_info.value.status_code == 404
    assert "No earnings calls found" in exc_info.value.detail


async def test_get_earnings_calls_list_api_error(mock_finance_client, bypass_cache):
    """Test error handling when API call fails"""
    mock_finance_client.get_earnings_calls_list.side_effect = Exception("API error")

    with pytest.raises(HTTPException) as exc_info:
        await get_earnings_calls_list(mock_finance_client, "ERROR")

    assert exc_info.value.status_code == 500
    assert "Failed to fetch earnings calls list" in exc_info.value.detail


async def test_get_earnings_transcript_most_recent(
    mock_finance_client, sample_earnings_calls_response, sample_quote_type_response, sample_transcript_response, bypass_cache
):
    """Test getting most recent earnings transcript"""
    mock_finance_client.get_earnings_calls_list.return_value = sample_earnings_calls_response
    mock_finance_client.get_quote_type.return_value = sample_quote_type_response
    mock_finance_client.get_earnings_transcript.return_value = sample_transcript_response

    result = await get_earnings_transcript(mock_finance_client, "AAPL")

    assert result.symbol == "AAPL"
    assert result.quarter == "Q4"
    assert result.year == 2024
    assert len(result.speakers) == 2
    assert len(result.paragraphs) == 3
    assert result.speakers[0].name == "Tim Cook"


async def test_get_earnings_transcript_with_quarter_filter(
    mock_finance_client, sample_earnings_calls_response, sample_quote_type_response, sample_transcript_response, bypass_cache
):
    """Test getting earnings transcript with quarter filter"""
    mock_finance_client.get_earnings_calls_list.return_value = sample_earnings_calls_response
    mock_finance_client.get_quote_type.return_value = sample_quote_type_response
    mock_finance_client.get_earnings_transcript.return_value = sample_transcript_response

    result = await get_earnings_transcript(mock_finance_client, "AAPL", quarter="Q3")

    # Should get Q3 call (event456)
    assert result.symbol == "AAPL"
    # Verify the correct call was fetched
    mock_finance_client.get_earnings_transcript.assert_called_once()
    call_args = mock_finance_client.get_earnings_transcript.call_args
    assert call_args.args[0] == "event456"  # Q3's eventId


async def test_get_earnings_transcript_with_year_filter(
    mock_finance_client, sample_earnings_calls_response, sample_quote_type_response, sample_transcript_response, bypass_cache
):
    """Test getting earnings transcript with year filter"""
    mock_finance_client.get_earnings_calls_list.return_value = sample_earnings_calls_response
    mock_finance_client.get_quote_type.return_value = sample_quote_type_response
    mock_finance_client.get_earnings_transcript.return_value = sample_transcript_response

    result = await get_earnings_transcript(mock_finance_client, "AAPL", year=2024)

    # Should get most recent 2024 call (Q4)
    assert result.symbol == "AAPL"
    mock_finance_client.get_earnings_transcript.assert_called_once()
    call_args = mock_finance_client.get_earnings_transcript.call_args
    assert call_args.args[0] == "event123"  # Q4's eventId


async def test_get_earnings_transcript_with_both_filters(
    mock_finance_client, sample_earnings_calls_response, sample_quote_type_response, sample_transcript_response, bypass_cache
):
    """Test getting earnings transcript with both quarter and year filters"""
    mock_finance_client.get_earnings_calls_list.return_value = sample_earnings_calls_response
    mock_finance_client.get_quote_type.return_value = sample_quote_type_response
    mock_finance_client.get_earnings_transcript.return_value = sample_transcript_response

    result = await get_earnings_transcript(mock_finance_client, "AAPL", quarter="Q2", year=2024)

    assert result.symbol == "AAPL"
    mock_finance_client.get_earnings_transcript.assert_called_once()
    call_args = mock_finance_client.get_earnings_transcript.call_args
    assert call_args.args[0] == "event789"  # Q2's eventId


async def test_get_earnings_transcript_quarter_normalization(
    mock_finance_client, sample_earnings_calls_response, sample_quote_type_response, sample_transcript_response, bypass_cache
):
    """Test quarter normalization (e.g., '2' -> 'Q2')"""
    mock_finance_client.get_earnings_calls_list.return_value = sample_earnings_calls_response
    mock_finance_client.get_quote_type.return_value = sample_quote_type_response
    mock_finance_client.get_earnings_transcript.return_value = sample_transcript_response

    # Test with just the number
    result = await get_earnings_transcript(mock_finance_client, "AAPL", quarter="2")

    assert result.symbol == "AAPL"
    mock_finance_client.get_earnings_transcript.assert_called_once()
    call_args = mock_finance_client.get_earnings_transcript.call_args
    assert call_args.args[0] == "event789"  # Q2's eventId


async def test_get_earnings_transcript_no_matching_call(mock_finance_client, sample_earnings_calls_response, bypass_cache):
    """Test error when no call matches filters"""
    mock_finance_client.get_earnings_calls_list.return_value = sample_earnings_calls_response

    with pytest.raises(HTTPException) as exc_info:
        await get_earnings_transcript(mock_finance_client, "AAPL", quarter="Q1", year=2023)

    assert exc_info.value.status_code == 404
    assert "No earnings call found" in exc_info.value.detail


async def test_get_earnings_transcript_no_company_id(mock_finance_client, sample_earnings_calls_response, bypass_cache):
    """Test error handling when no company ID found"""
    mock_finance_client.get_earnings_calls_list.return_value = sample_earnings_calls_response
    mock_finance_client.get_quote_type.return_value = {"quoteType": {"result": []}}

    with pytest.raises(HTTPException) as exc_info:
        await get_earnings_transcript(mock_finance_client, "AAPL")

    assert exc_info.value.status_code == 404
    assert "Could not find company ID" in exc_info.value.detail


async def test_get_earnings_transcript_missing_quartr_id(mock_finance_client, sample_earnings_calls_response, bypass_cache):
    """Test error handling when quartrId is missing"""
    mock_finance_client.get_earnings_calls_list.return_value = sample_earnings_calls_response
    mock_finance_client.get_quote_type.return_value = {"quoteType": {"result": [{"symbol": "AAPL"}]}}  # Missing quartrId

    with pytest.raises(HTTPException) as exc_info:
        await get_earnings_transcript(mock_finance_client, "AAPL")

    assert exc_info.value.status_code == 404
    assert "No company ID (quartrId) found" in exc_info.value.detail


async def test_get_earnings_transcript_api_error(mock_finance_client, sample_earnings_calls_response, sample_quote_type_response, bypass_cache):
    """Test error handling when transcript API call fails"""
    mock_finance_client.get_earnings_calls_list.return_value = sample_earnings_calls_response
    mock_finance_client.get_quote_type.return_value = sample_quote_type_response
    mock_finance_client.get_earnings_transcript.side_effect = Exception("Transcript API error")

    with pytest.raises(HTTPException) as exc_info:
        await get_earnings_transcript(mock_finance_client, "AAPL")

    assert exc_info.value.status_code == 500
    assert "Failed to fetch earnings transcript" in exc_info.value.detail


async def test_get_earnings_transcript_calls_api_with_correct_params(
    mock_finance_client, sample_earnings_calls_response, sample_quote_type_response, sample_transcript_response, bypass_cache
):
    """Test that APIs are called with correct parameters"""
    mock_finance_client.get_earnings_calls_list.return_value = sample_earnings_calls_response
    mock_finance_client.get_quote_type.return_value = sample_quote_type_response
    mock_finance_client.get_earnings_transcript.return_value = sample_transcript_response

    await get_earnings_transcript(mock_finance_client, "AAPL")

    # Verify get_earnings_calls_list was called
    mock_finance_client.get_earnings_calls_list.assert_called_once_with("AAPL")

    # Verify get_quote_type was called
    mock_finance_client.get_quote_type.assert_called_once_with("AAPL")

    # Verify get_earnings_transcript was called with event ID and company ID
    mock_finance_client.get_earnings_transcript.assert_called_once()
    call_args = mock_finance_client.get_earnings_transcript.call_args
    assert call_args.args[0] == "event123"  # eventId of most recent call
    assert call_args.args[1] == "company123"  # quartrId from quote_type response

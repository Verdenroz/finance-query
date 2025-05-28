import pytest
from unittest.mock import AsyncMock, patch
from fastapi import HTTPException

from src.utils.retry import retry


class TestRetry:
    """Test suite for the retry decorator."""

    async def test_successful_primary(self):
        """Test that primary function is called and returns correctly."""
        primary = AsyncMock(return_value="success")
        fallback = AsyncMock()

        decorated = retry(fallback)(primary)
        result = await decorated("arg1", kwarg1="value1")

        assert result == "success"
        primary.assert_called_once_with("arg1", kwarg1="value1")
        fallback.assert_not_called()

    async def test_retry_then_success(self):
        """Test that primary is retried after exception."""
        primary = AsyncMock(side_effect=[ValueError("error"), "success"])
        fallback = AsyncMock()

        decorated = retry(fallback, retries=1)(primary)
        result = await decorated("arg1", kwarg1="value1")

        assert result == "success"
        assert primary.call_count == 2
        fallback.assert_not_called()

    async def test_fallback_after_retries(self):
        """Test fallback is called after all retries are exhausted."""
        primary = AsyncMock(side_effect=ValueError("error"))
        fallback = AsyncMock(return_value="fallback result")

        decorated = retry(fallback, retries=2)(primary)
        result = await decorated("arg1", kwarg1="value1")

        assert result == "fallback result"
        assert primary.call_count == 3  # Initial + 2 retries
        fallback.assert_called_once()

    async def test_http_exception_not_retried(self):
        """Test that HTTPException is immediately raised without retry."""
        http_error = HTTPException(status_code=404, detail="Not found")
        primary = AsyncMock(side_effect=http_error)
        fallback = AsyncMock()

        decorated = retry(fallback, retries=2)(primary)

        with pytest.raises(HTTPException) as exc_info:
            await decorated("arg1")

        assert exc_info.value == http_error
        primary.assert_called_once()
        fallback.assert_not_called()

    async def test_fallback_parameter_filtering(self):
        """Test that only parameters accepted by fallback are passed to it."""
        primary = AsyncMock(side_effect=ValueError("error"))

        # Fallback that only accepts some parameters
        async def fallback_fn(kwarg1):
            return f"fallback with {kwarg1}"

        decorated = retry(fallback_fn)(primary)
        result = await decorated(kwarg1="value1", extra_kwarg="ignored")

        assert result == "fallback with value1"
        primary.assert_called_once()

    async def test_default_parameters(self):
        """Test that default parameters in primary are correctly handled."""
        primary = AsyncMock(side_effect=ValueError("error"))
        fallback = AsyncMock()

        # Add signatures to both mocks with default parameters
        async def func_with_defaults(arg1, kwarg1="default"):
            pass

        primary.__signature__ = pytest.importorskip("inspect").signature(func_with_defaults)
        # Also give fallback a signature that includes kwarg1 so it can receive it
        fallback.__signature__ = pytest.importorskip("inspect").signature(func_with_defaults)

        decorated = retry(fallback)(primary)
        await decorated("arg1")

        fallback.assert_called_once()
        # The default should be passed to fallback
        assert "kwarg1" in fallback.call_args[1] and fallback.call_args[1]["kwarg1"] == "default"

    @patch("src.utils.retry.logging.getLogger")
    async def test_logging(self, mock_get_logger):
        """Test that appropriate logging occurs."""
        mock_logger = AsyncMock()
        mock_get_logger.return_value = mock_logger

        primary = AsyncMock(side_effect=ValueError("test error"))
        fallback = AsyncMock(return_value="fallback result")

        decorated = retry(fallback)(primary)
        await decorated("arg1")

        # Check exception logging
        assert mock_logger.exception.called
        # Check fallback logging
        assert mock_logger.info.called
        assert "Switching to fallback" in mock_logger.info.call_args[0][0]

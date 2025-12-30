import pytest
from fastapi.testclient import TestClient

from src.main import app


@pytest.fixture
def client():
    return TestClient(app)


def test_get_earnings_transcript_invalid_symbol_pattern(client):
    """Test validation error for invalid symbol pattern"""
    response = client.get("/v1/earnings-transcript/invalid-symbol")

    assert response.status_code == 422
    assert "detail" in response.json()

import pytest
from fastapi.testclient import TestClient
from src.main import app

VERSION = "v1"


@pytest.fixture(scope="session")
def test_client():
    """
    TestClient fixture that can be reused across all tests.
    """
    with TestClient(app) as client:
        yield client

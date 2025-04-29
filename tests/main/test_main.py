from datetime import datetime


async def test_health_endpoint(test_client):
    """Test health check endpoint returns correct structure and status"""
    response = test_client.get("/health")

    assert response.status_code == 200

    data = response.json()

    # Verify basic structure
    assert "status" in data
    assert "timestamp" in data
    assert "services" in data

    # Verify status is either healthy or degraded
    assert data["status"] in ["healthy", "degraded"]

    # Verify timestamp is valid ISO format
    assert datetime.fromisoformat(data["timestamp"])

    # Verify services section
    services = data["services"]
    assert "status" in services
    assert isinstance(services["status"], str)
    assert services["status"].endswith("succeeded")

    # Check for all expected services
    expected_services = {
        "Indices",
        "Market Actives",
        "Market Losers",
        "Market Gainers",
        "Market Sectors",
        "Sector for a symbol",
        "Detailed Sector",
        "General News",
        "News for equity",
        "News for ETF",
        "Full Quotes",
        "Simple Quotes",
        "Similar Equities",
        "Similar ETFs",
        "Historical day prices",
        "Historical month prices",
        "Historical year prices",
        "Historical five year prices",
        "Search",
        "Summary Analysis",
    }

    # Every service should have a status
    for service in expected_services:
        assert service in services
        assert "status" in services[service]
        assert services[service]["status"] in ["succeeded", "FAILED"]
        if services[service]["status"] == "FAILED":
            assert "ERROR" in services[service]


async def test_ping_endpoint(test_client):
    """Test ping endpoint returns correct structure and headers"""
    response = test_client.get("/ping")

    assert response.status_code == 200

    # Verify response headers
    assert response.headers["Cache-Control"] == "no-cache, no-store, must-revalidate"

    data = response.json()

    # Verify response structure
    assert "status" in data
    assert "timestamp" in data

    # Verify status value
    assert data["status"] == "healthy"

    # Verify timestamp is valid ISO format
    assert datetime.fromisoformat(data["timestamp"])

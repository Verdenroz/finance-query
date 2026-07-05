from tests.conftest import VERSION


def get_metrics_body(test_client) -> str:
    response = test_client.get(f"/{VERSION}/metrics")
    assert response.status_code == 200
    assert response.headers["content-type"].startswith("text/plain")
    return response.text


def test_metrics_endpoint_exposes_all_metrics(test_client):
    body = get_metrics_body(test_client)

    assert "finance_query_v1_http_requests_in_flight" in body
    assert "finance_query_v1_redis_connected" in body
    assert "finance_query_v1_redis_errors_total" in body
    assert "finance_query_v1_http_request_duration_seconds" in body


def test_requests_counter_uses_route_template(test_client):
    response = test_client.get(f"/{VERSION}/health")
    assert response.status_code == 200

    body = get_metrics_body(test_client)

    # The exposition format renders labels alphabetically.
    assert f'finance_query_v1_http_requests_total{{endpoint="/{VERSION}/health",method="GET",status="200"}}' in body
    assert f'finance_query_v1_http_request_duration_seconds_bucket{{endpoint="/{VERSION}/health",le="0.001",method="GET"}}' in body


def test_endpoint_label_is_route_template_for_path_params(test_client):
    # Invalid quarter -> 422 before the handler runs; routing has already
    # resolved, so the metric label must be the template, not the raw path.
    response = test_client.get(f"/{VERSION}/earnings-transcript/AAPL/QX/2024")
    assert response.status_code == 422

    body = get_metrics_body(test_client)

    template = f"/{VERSION}/earnings-transcript/{{symbol}}/{{quarter}}/{{year}}"
    assert f'endpoint="{template}"' in body
    assert 'endpoint="/v1/earnings-transcript/AAPL/QX/2024"' not in body


def test_unmatched_paths_share_single_label(test_client):
    response = test_client.get(f"/{VERSION}/definitely-not-a-real-route-xyz")
    assert response.status_code == 404

    body = get_metrics_body(test_client)

    assert 'endpoint="unmatched"' in body
    assert 'endpoint="/v1/definitely-not-a-real-route-xyz"' not in body


def test_metrics_endpoint_is_not_self_observed(test_client):
    # Scrape twice: the second body reflects the first scrape's request.
    get_metrics_body(test_client)
    body = get_metrics_body(test_client)

    assert f'endpoint="/{VERSION}/metrics"' not in body

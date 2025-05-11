from fastapi import HTTPException
from orjson import orjson

from utils.dependencies import fetch


async def get_yahoo_sector(symbol: str, cookies: dict, crumb: str) -> str | None:
    summary_data = await _fetch_yahoo_data(symbol, cookies, crumb)
    summary_result = summary_data.get("quoteSummary", {}).get("result", [{}])[0]
    profile = summary_result.get("assetProfile", {})
    return profile.get("sector", None)


async def _fetch_yahoo_data(symbol: str, cookies: dict, crumb: str) -> dict:
    """
    Fetch raw data from Yahoo Finance API using cookies and crumb.

    :raises HTTPException: with code 404 if symbol is not found
    """
    summary_url = f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/{symbol}"
    summary_params = {"modules": "assetProfile", "crumb": crumb}
    headers = {
        "Cookie": "; ".join(f"{k}={v}" for k, v in cookies.items()),
        "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
        "Accept": "application/json",
    }

    summary_response = await fetch(url=summary_url, params=summary_params, headers=headers, return_response=True)

    if summary_response.status == 404:
        raise HTTPException(status_code=404, detail=f"Symbol not found: {symbol}")

    response_text = await summary_response.text()
    summary_data = orjson.loads(response_text)
    return summary_data

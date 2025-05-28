from typing import Optional

from fastapi import HTTPException
from orjson import orjson

from .fetch_client import CurlFetchClient


class YahooFinanceClient(CurlFetchClient):
    """
    Specialised Yahoo wrapper — everything else comes from CurlFetchClient.
    """

    def __init__(
        self,
        cookies: dict[str, str],
        crumb: str,
        proxy: Optional[str] = None,
        timeout: int = 10,
    ):
        super().__init__(timeout=timeout, proxy=proxy)
        self.session.cookies.update(cookies)
        self.crumb = crumb

    async def _yahoo_request(self, url: str, **kw):
        kw.setdefault("params", {})["crumb"] = self.crumb
        kw.setdefault("headers", {})[
            "User-Agent"
        ] = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36"

        # Ensure cookies are properly formatted if they're provided as a dictionary
        if "cookies" in kw and isinstance(kw["cookies"], dict):
            kw.setdefault("headers", {})["Cookie"] = "; ".join(f"{k}={v}" for k, v in kw["cookies"].items())
            del kw["cookies"]

        resp = await self.fetch(url, **kw, return_response=True)

        if resp.status_code == 401:
            raise HTTPException(401, "Yahoo auth failed")
        if resp.status_code == 404:
            raise HTTPException(404, "Yahoo symbol not found")
        if resp.status_code == 429:
            raise HTTPException(429, "Yahoo rate-limit exceeded")
        if resp.status_code >= 400:
            raise HTTPException(resp.status_code, resp.text)
        return resp

    async def _json(self, url: str, **kw):
        resp = await self._yahoo_request(url, **kw)
        try:
            return orjson.loads(resp.text)
        except Exception as e:
            # If parsing fails, raise a more informative exception
            raise HTTPException(status_code=500, detail=f"Failed to parse JSON response from {url}: {str(e)}") from e

    async def get_quote(self, symbol: str):
        """
        Fetch detailed quote summary data for a single symbol.
        """
        summary_url = f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/{symbol}"
        summary_params = {
            "modules": "assetProfile,price,summaryDetail,defaultKeyStatistics,calendarEvents,quoteUnadjustedPerformanceOverview",
        }
        return await self._json(summary_url, params=summary_params)

    async def get_simple_quotes(self, symbols: list[str]):
        """
        Fetch simplified quotes for multiple symbols in batch.
        """
        url = "https://query1.finance.yahoo.com/v7/finance/quote"
        params = {
            "symbols": ",".join(symbols),
            "modules": "assetProfile,price,summaryDetail,defaultKeyStatistics,calendarEvents,quoteUnadjustedPerformanceOverview",
        }
        return await self._json(url, params=params)

    async def get_chart(self, symbol: str, interval: str, range_: str):
        """
        Fetch chart data for a symbol.
        """
        return await self._json(
            f"https://query1.finance.yahoo.com/v8/finance/chart/{symbol}",
            params={"interval": interval, "range": range_},
        )

    async def search(self, query: str, hits: int = 6):
        """
        Search for quotes and news.
        """
        return await self._json(
            "https://query1.finance.yahoo.com/v1/finance/search",
            params={"q": query, "quotesCount": hits},
        )

    async def get_similar_quotes(self, symbol: str, limit: int):
        """
        Get similar quotes for a symbol.
        """
        return await self._json(f"https://query2.finance.yahoo.com/v6/finance/recommendationsbysymbol/{symbol}", params={"count": limit})

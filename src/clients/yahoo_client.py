import re
import time
from typing import Optional

from fastapi import HTTPException
from lxml import html as lxml_html
from orjson import orjson

from src.utils.logging import get_logger, log_external_api_call

from .fetch_client import CurlFetchClient

logger = get_logger(__name__)


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
        kw.setdefault("headers", {})["User-Agent"] = (
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36"
        )

        # Ensure cookies are properly formatted if they're provided as a dictionary
        if "cookies" in kw and isinstance(kw["cookies"], dict):
            kw.setdefault("headers", {})["Cookie"] = "; ".join(f"{k}={v}" for k, v in kw["cookies"].items())
            del kw["cookies"]

        # Extract endpoint from URL for logging
        endpoint = url.split("/")[-1] if "/" in url else url
        start_time = time.perf_counter()

        try:
            resp = await self.fetch(url, **kw, return_response=True)
            duration_ms = (time.perf_counter() - start_time) * 1000
            success = resp.status_code < 400
            log_external_api_call(logger, "Yahoo Finance", endpoint, duration_ms, success=success)
        except Exception:
            duration_ms = (time.perf_counter() - start_time) * 1000
            log_external_api_call(logger, "Yahoo Finance", endpoint, duration_ms, success=False)
            raise

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

    async def get_fundamentals_timeseries(self, symbol: str, period1: int, period2: int, types: list[str]):
        """
        Fetch fundamentals timeseries data (financial statements, etc.).

        Args:
            symbol: Stock symbol
            period1: Start Unix timestamp
            period2: End Unix timestamp
            types: List of fundamental types (e.g., ['annualTotalRevenue', 'quarterlyTotalRevenue'])
        """
        return await self._json(
            f"https://query1.finance.yahoo.com/ws/fundamentals-timeseries/v1/finance/timeseries/{symbol}",
            params={
                "merge": "false",
                "padTimeSeries": "true",
                "period1": period1,
                "period2": period2,
                "type": ",".join(types),
                "lang": "en-US",
                "region": "US",
            },
        )

    async def get_quote_summary(self, symbol: str, modules: list[str]):
        """
        Fetch quote summary data with specified modules.

        Args:
            symbol: Stock symbol
            modules: List of modules to fetch (e.g., ['institutionOwnership', 'majorHoldersBreakdown'])
        """
        return await self._json(
            f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/{symbol}",
            params={
                "modules": ",".join(modules),
                "corsDomain": "finance.yahoo.com",
                "formatted": "false",
            },
        )

    async def get_quote_type(self, symbol: str):
        """
        Fetch quote type data including company ID (quartrId) for a symbol.

        Args:
            symbol: Stock symbol

        Returns:
            Dict containing quoteType data including quartrId
        """
        url = f"https://query1.finance.yahoo.com/v1/finance/quoteType/{symbol}"
        return await self._json(url)

    async def get_earnings_calls_list(self, symbol: str):
        """
        Scrape the earnings calls page to get list of available earnings call transcripts.

        Args:
            symbol: Stock symbol

        Returns:
            List of dicts with eventId, quarter, year, title, url
        """
        url = f"https://finance.yahoo.com/quote/{symbol}/earnings-calls/"

        # Fetch the HTML page
        start_time = time.perf_counter()
        try:
            resp = await self.fetch(url, return_response=True)
            duration_ms = (time.perf_counter() - start_time) * 1000
            success = resp.status_code < 400
            log_external_api_call(logger, "Yahoo Finance", "earnings-calls-page", duration_ms, success=success)
        except Exception:
            duration_ms = (time.perf_counter() - start_time) * 1000
            log_external_api_call(logger, "Yahoo Finance", "earnings-calls-page", duration_ms, success=False)
            raise

        if resp.status_code >= 400:
            raise HTTPException(resp.status_code, f"Failed to fetch earnings calls page: {resp.text[:200]}")

        # Parse HTML
        tree = lxml_html.fromstring(resp.text)

        # Find all links containing "earnings_call"
        all_links = tree.xpath("//a/@href")
        earnings_links = [link for link in all_links if "earnings_call" in link]

        # Extract event IDs and quarter/year info
        eventid_pattern = r"earnings_call-(\d+)"
        quarter_year_pattern = r"-([Qq]\d)-(\d{4})-earnings_call"

        calls = []
        seen_event_ids = set()

        for link in earnings_links:
            event_match = re.search(eventid_pattern, link)
            if event_match:
                event_id = event_match.group(1)

                # Skip duplicates
                if event_id in seen_event_ids:
                    continue
                seen_event_ids.add(event_id)

                # Extract quarter and year
                qy_match = re.search(quarter_year_pattern, link)
                quarter = qy_match.group(1).upper() if qy_match else None
                year = int(qy_match.group(2)) if qy_match else None

                # Extract title from link text (try to find the corresponding link element)
                title = f"{quarter} {year}" if quarter and year else "Earnings Call"

                calls.append({"eventId": event_id, "quarter": quarter, "year": year, "title": title, "url": f"https://finance.yahoo.com{link}"})

        return calls

    async def get_earnings_transcript(self, event_id: str, company_id: str):
        """
        Fetch earnings call transcript from Yahoo Finance.

        Args:
            event_id: Event ID for the earnings call
            company_id: Company ID (quartrId) for the symbol

        Returns:
            Dict containing transcript content and metadata
        """
        url = "https://finance.yahoo.com/xhr/transcript"
        params = {"eventType": "earnings_call", "quartrId": company_id, "eventId": event_id, "lang": "en-US", "region": "US"}

        return await self._json(url, params=params)

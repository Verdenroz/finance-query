import asyncio
from typing import Any, Literal, Optional

from curl_cffi import requests
from fastapi import HTTPException

DEFAULT_TIMEOUT = 10


class CurlFetchClient:
    """
    A wrapper around curl_cffi's requests.Session to provide
    a consistent interface for making HTTP requests.
    """

    def __init__(
        self,
        *,
        session: Optional[requests.Session] = None,
        timeout: int = DEFAULT_TIMEOUT,
        proxy: Optional[str] = None,
        default_headers: Optional[dict[str, str]] = None,
    ) -> None:
        if default_headers is None:
            default_headers = {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
                "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
                "Accept-Language": "en-US,en;q=0.9",
                "Accept-Encoding": "gzip, deflate, br",
                "sec-ch-ua": '"Chromium";v="122", "Google Chrome";v="122"',
                "sec-ch-ua-mobile": "?0",
                "sec-ch-ua-platform": '"Windows"',
            }
        self.session = session or requests.Session(impersonate="chrome")
        self.timeout = timeout
        self.session.headers.update(default_headers or {})
        self.proxies = {"http": proxy, "https": proxy} if proxy else None

    def request(
        self,
        url: str,
        method: Literal["GET", "POST", "PUT", "DELETE", "OPTIONS", "HEAD", "TRACE", "PATCH", "QUERY"] = "GET",
        *,
        params: dict[str, Any] | None = None,
        data: dict[str, Any] | None = None,
        headers: dict[str, str] | None = None,
        proxy: Optional[str] = None,
    ) -> requests.Response:
        """
        Synchronous wrapper for the request method.
        :param url: the URL to fetch.
        :param method: the HTTP method to use.
        :param params: the URL parameters to include in the request.
        :param data: the data to include in the request body.
        :param headers: the headers to include in the request.
        :param proxy: Optional proxy URL to use for this request (overrides instance proxy).

        :return: a requests.Response object.
        """
        # Use provided proxy or fallback to instance-level proxy
        proxies = {"http": proxy, "https": proxy} if proxy else self.proxies

        try:
            return self.session.request(
                method=method,
                url=url,
                params=params,
                json=data,
                headers=headers,
                timeout=self.timeout,
                proxies=proxies,
            )
        except requests.RequestsError as exc:
            raise HTTPException(500, f"HTTP request failed: {exc}") from exc

    async def fetch(
        self,
        url: str,
        method: Literal["GET", "POST", "PUT", "DELETE", "OPTIONS", "HEAD", "TRACE", "PATCH", "QUERY"] = "GET",
        *,
        params: dict[str, Any] | None = None,
        data: dict[str, Any] | None = None,
        headers: dict[str, str] | None = None,
        return_response: bool = False,
        proxy: Optional[str] = None,
    ):
        """
        Asynchronous wrapper for the request method.
        :param url: the URL to fetch.
        :param method: the HTTP method to use.
        :param params: the URL parameters to include in the request.
        :param data: the data to include in the request body.
        :param headers: the headers to include in the request.
        :param return_response: whether to return the response object or the response text.
        :param proxy: Optional proxy URL to use for this request (overrides instance proxy).

        :return: the response text or the response object.
        """
        resp: requests.Response = await asyncio.to_thread(self.request, url, method=method, params=params, data=data, headers=headers, proxy=proxy)
        return resp if return_response else resp.text

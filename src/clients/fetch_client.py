import asyncio
from typing import Optional, Dict, Any, Literal

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
            default_headers: Optional[Dict[str, str]] = None,
    ) -> None:
        self.session = session or requests.Session(impersonate="chrome")
        self.timeout = timeout
        self.session.headers.update(default_headers or {})
        if proxy:
            self.session.proxies = {"http": proxy, "https": proxy}

    def request(
            self,
            url: str,
            method: Literal["GET", "POST", "PUT", "DELETE", "OPTIONS", "HEAD", "TRACE", "PATCH", "QUERY"] = "GET",
            *,
            params: Dict[str, Any] | None = None,
            data: Dict[str, Any] | None = None,
            headers: Dict[str, str] | None = None,
    ) -> requests.Response:
        """
        Synchronous wrapper for the request method.
        :param url: the URL to fetch.
        :param method: the HTTP method to use.
        :param params: the URL parameters to include in the request.
        :param data: the data to include in the request body.
        :param headers: the headers to include in the request.

        :return: a requests.Response object.
        """
        if not url:
            raise ValueError("Missing URL")

        try:
            return self.session.request(
                method=method,
                url=url,
                params=params,
                json=data,
                headers=headers,
                timeout=self.timeout,
            )
        except requests.RequestsError as exc:
            raise HTTPException(500, f"HTTP request failed: {exc}") from exc

    async def fetch(
            self,
            url: str,
            method: Literal["GET", "POST", "PUT", "DELETE", "OPTIONS", "HEAD", "TRACE", "PATCH", "QUERY"] = "GET",
            *,
            params: Dict[str, Any] | None = None,
            data: Dict[str, Any] | None = None,
            headers: Dict[str, str] | None = None,
            return_response: bool = False,
    ):
        """
        Asynchronous wrapper for the request method.
        :param url: the URL to fetch.
        :param method: the HTTP method to use.
        :param params: the URL parameters to include in the request.
        :param data: the data to include in the request body.
        :param headers: the headers to include in the request.
        :param return_response: whether to return the response object or the response text.

        :return: the response text or the response object.
        """
        resp: requests.Response = await asyncio.to_thread(
            self.request, url, method=method, params=params, data=data, headers=headers
        )
        return resp if return_response else resp.text

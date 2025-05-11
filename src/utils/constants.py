import os

import aiohttp
from dotenv import load_dotenv

load_dotenv()

proxy_auth = (
    aiohttp.BasicAuth(os.environ.get("PROXY_USER"), os.environ.get("PROXY_PASSWORD"))
    if os.environ.get("PROXY_USER") and os.environ.get("PROXY_PASSWORD")
    else None
)

proxy = os.environ.get("PROXY_URL", None)

default_headers = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
    "Accept-Language": "en-US,en;q=0.9",
    "Accept-Encoding": "gzip, deflate, br",
    "sec-ch-ua": '"Chromium";v="122", "Google Chrome";v="122"',
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": '"Windows"',
}

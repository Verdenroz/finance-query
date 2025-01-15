import os

import aiohttp
from dotenv import load_dotenv

load_dotenv()

proxy_auth = aiohttp.BasicAuth(os.environ.get('PROXY_USER'), os.environ.get('PROXY_PASSWORD')) if os.environ.get('PROXY_USER') and os.environ.get('PROXY_PASSWORD') else None

proxy = os.environ.get('PROXY_URL', None)

headers = {
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3',
    'Accept-Encoding': 'gzip, deflate'
}
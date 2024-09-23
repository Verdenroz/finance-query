import os

import aiohttp
from dotenv import load_dotenv

load_dotenv()

proxy_auth = aiohttp.BasicAuth(os.environ.get('PROXY_USER'), os.environ.get('PROXY_PASSWORD')) if os.environ.get('PROXY_USER') and os.environ.get('PROXY_PASSWORD') else None

proxy = os.environ.get('PROXY_URL', None)
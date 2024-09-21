import os

import aiohttp
from dotenv import load_dotenv

load_dotenv()

proxy_auth = aiohttp.BasicAuth(os.environ['PROXY_USER'], os.environ['PROXY_PASSWORD'])

proxy = os.environ['PROXY_URL']

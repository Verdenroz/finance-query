import os
from contextlib import asynccontextmanager

from dotenv import load_dotenv
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from mangum import Mangum

from src.redis import r
from src.routes import (quotes_router, indices_router, movers_router, historical_prices_router,
                        similar_stocks_router, finance_news_router, indicators_router, search_router,
                        sectors_router, sockets_router, stream_router)
from src.security import RateLimitMiddleware
from src.session_manager import get_global_session, close_global_session

load_dotenv()


@asynccontextmanager
async def lifespan(app: FastAPI):
    session = await get_global_session()
    if os.getenv('PROXY_TOKEN') and os.getenv('USE_PROXY', 'False') == 'True':
        async with session.get("https://api.ipify.org/") as ip_response:
            ip = await ip_response.text()
            api_url = "https://api.brightdata.com/zone/whitelist"
            proxy_header_token = {
                "Authorization": f"Bearer {os.getenv('PROXY_TOKEN')}",
                "Content-Type": "application/json"
            }
            payload = {"ip": ip}
            await session.post(api_url, headers=proxy_header_token, json=payload)
    yield
    await session.delete(api_url, headers=proxy_header_token, json=payload)
    await close_global_session()
    await r.close()

app = FastAPI(
    title="FinanceQuery",
    version="1.4.7",
    description="FinanceQuery is a simple API to query financial data."
                " It provides endpoints to get quotes, historical prices, indices,"
                " market movers, similar stocks, finance news, indicators, search, and sectors."
                " Please use FinanceQueryDemoAWSHT as the demo API key which is limited to 2000 requests/day."
                " You are free to deploy your own instance of FinanceQuery to AWS and use your own API key."
                " If you are testing locally you can use the local server and will not need a key."
    ,
    servers=[
        {"url": "https://43pk30s7aj.execute-api.us-east-2.amazonaws.com/prod", "description": "AWS server"},
        {"url": "http://127.0.0.1:8000", "description": "Local server"}
    ],
    contact={
        "name": "Harvey Tseng",
        "email": "harveytseng2@gmail.com"
    },
    license_info={
        "name": "MIT License",
        "identifier": "MIT",
    },
    lifespan=lifespan
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],  # Allows all methods
    allow_headers=["*"],  # Allows all headers
    expose_headers=["X-RateLimit-Limit", "X-RateLimit-Remaining", "X-RateLimit-Reset"]
)

if os.getenv('USE_SECURITY', 'False') == 'True':
    app.add_middleware(RateLimitMiddleware)

app.include_router(quotes_router, prefix="/v1")
app.include_router(historical_prices_router, prefix="/v1")
app.include_router(indicators_router, prefix="/v1")
app.include_router(indices_router, prefix="/v1")
app.include_router(movers_router, prefix="/v1")
app.include_router(similar_stocks_router, prefix="/v1")
app.include_router(finance_news_router, prefix="/v1")
app.include_router(search_router, prefix="/v1")
app.include_router(sectors_router, prefix="/v1")
app.include_router(stream_router, prefix="/v1")
app.include_router(sockets_router)

handler = Mangum(app)

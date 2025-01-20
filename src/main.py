import asyncio
import datetime
import os
import time
from contextlib import asynccontextmanager

from dotenv import load_dotenv
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from mangum import Mangum
from starlette.responses import Response

from src.di import get_global_session, close_global_session, get_global_rate_limit_manager
from src.redis import r
from src.routes import (quotes_router, indices_router, movers_router, historical_prices_router,
                        similar_quotes_router, finance_news_router, indicators_router, search_router,
                        sectors_router, sockets_router, stream_router, hours_router)
from src.schemas.sector import Sector
from src.schemas.time_series import TimePeriod, Interval
from src.security import RateLimitMiddleware
from src.services import scrape_indices, scrape_actives, scrape_losers, scrape_gainers, get_sectors, \
    get_sector_for_symbol, get_sector_details, scrape_general_news, scrape_news_for_quote, scrape_quotes, \
    scrape_similar_quotes, get_historical, get_search, scrape_simple_quotes
from src.services.indicators.get_summary_analysis import get_summary_analysis

load_dotenv()


@asynccontextmanager
async def lifespan(app: FastAPI):
    session = await get_global_session()
    api_url = None
    proxy_header_token = None
    payload = None
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
    if api_url and proxy_header_token and payload:
        await session.delete(api_url, headers=proxy_header_token, json=payload)
    await close_global_session()
    await r.close()


app = FastAPI(
    title="FinanceQuery",
    version="1.5.9",
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
    app.add_middleware(RateLimitMiddleware, rate_limit_manager=get_global_rate_limit_manager())


@app.get("/health",
         response_model=dict,
         description="Detailed health check endpoint",
         tags=["Health Check"],
         responses={
             200: {
                 "description": "Successful Response",
                 "content": {
                     "application/json": {
                         "example": {
                             "status": "healthy",
                             "timestamp": "2023-10-01T12:34:56.789Z",
                             "redis": {
                                 "status": "healthy",
                                 "latency_ms": 1.23
                             },
                             "scraping": {
                                 "Scraping status": "21/21 succeeded",
                                 "Indices": {"status": "succeeded"},
                                 "Market Actives": {"status": "succeeded"},
                                 "Market Losers": {"status": "succeeded"},
                                 "Market Gainers": {"status": "succeeded"},
                                 "Market Sectors": {"status": "succeeded"},
                                 "Sector for a symbol": {"status": "succeeded"},
                                 "Detailed Sector": {"status": "succeeded"},
                                 "General News": {"status": "succeeded"},
                                 "News for equity": {"status": "succeeded"},
                                 "News for ETF": {"status": "succeeded"},
                                 "Full Quotes": {"status": "succeeded"},
                                 "Simple Quotes": {"status": "succeeded"},
                                 "Similar Equities": {"status": "succeeded"},
                                 "Similar ETFs": {"status": "succeeded"},
                                 "Historical day prices": {"status": "succeeded"},
                                 "Historical week prices": {"status": "succeeded"},
                                 "Historical month prices": {"status": "succeeded"},
                                 "Historical year prices": {"status": "succeeded"},
                                 "Historical five year prices": {"status": "succeeded"},
                                 "Search": {"status": "succeeded"},
                                 "Summary Analysis": {"status": "succeeded"}
                             }
                         }
                     }
                 }
             }
         }
         )
async def health():
    """
        Comprehensive health check endpoint that verifies:
        - Basic API health
        - Redis connectivity
        - System time
        - Service dependencies
        """
    indices_task = scrape_indices()
    actives_task = scrape_actives()
    losers_task = scrape_losers()
    gainers_task = scrape_gainers()
    sectors_task = get_sectors()
    sector_by_symbol_task = get_sector_for_symbol("NVDA")
    sector_by_name_task = get_sector_details(Sector.TECHNOLOGY)
    news_task = scrape_general_news()
    news_by_symbol_task = scrape_news_for_quote("NVDA")
    scrape_etf_news_task = scrape_news_for_quote("QQQ")
    quotes_task = scrape_quotes(["NVDA", "QQQ", "GTLOX"])
    simple_quotes_task = scrape_simple_quotes(["NVDA", "QQQ", "GTLOX"])
    similar_equity_task = scrape_similar_quotes("NVDA")
    similar_etf_task = scrape_similar_quotes("QQQ")
    historical_data_task_day = get_historical("NVDA", TimePeriod.DAY, Interval.ONE_MINUTE)
    historical_data_task_week = get_historical("NVDA", TimePeriod.SEVEN_DAYS, Interval.FIVE_MINUTES)
    historical_data_task_month = get_historical("NVDA", TimePeriod.YTD, Interval.DAILY)
    historical_data_task_year = get_historical("NVDA", TimePeriod.YEAR, Interval.DAILY)
    historical_data_task_five_years = get_historical("NVDA", TimePeriod.FIVE_YEARS, Interval.MONTHLY)
    search_task = get_search("NVDA")
    summary_analysis_task = get_summary_analysis("NVDA", Interval.DAILY)

    tasks = [
        ("Indices", indices_task),
        ("Market Actives", actives_task),
        ("Market Losers", losers_task),
        ("Market Gainers", gainers_task),
        ("Market Sectors", sectors_task),
        ("Sector for a symbol", sector_by_symbol_task),
        ("Detailed Sector", sector_by_name_task),
        ("General News", news_task),
        ("News for equity", news_by_symbol_task),
        ("News for ETF", scrape_etf_news_task),
        ("Full Quotes", quotes_task),
        ("Simple Quotes", simple_quotes_task),
        ("Similar Equities", similar_equity_task),
        ("Similar ETFs", similar_etf_task),
        ("Historical day prices", historical_data_task_day),
        ("Historical week prices", historical_data_task_week),
        ("Historical month prices", historical_data_task_month),
        ("Historical year prices", historical_data_task_year),
        ("Historical five year prices", historical_data_task_five_years),
        ("Search", search_task),
        ("Summary Analysis", summary_analysis_task)
    ]

    results = await asyncio.gather(*[task for _, task in tasks], return_exceptions=True)

    health_report = {
        "status": "healthy",
        "timestamp": datetime.datetime.utcnow().isoformat(),
        "redis": {
        },
        "scraping": {
            "Scraping status": "21/21 succeeded",
        }
    }

    # Check Redis
    try:
        start_time = time.time()
        redis_ping = await r.ping()
        health_report["redis"] = {
            "status": "healthy" if redis_ping else "unhealthy",
            "latency_ms": round((time.time() - start_time) * 1000, 2)
        }
    except Exception as e:
        health_report["dependencies"] = {
            "redis": {
                "status": "unhealthy",
                "error": str(e)
            }
        }
        health_report["status"] = "degraded"

    total = len(tasks)
    succeeded = 0
    for (name, task), result in zip(tasks, results):
        if isinstance(result, Exception):
            health_report["scraping"][name] = {"status": "FAILED", "ERROR": str(result)}
        else:
            health_report["scraping"][name] = {"status": "succeeded"}
            succeeded += 1

    scraping_status = f"{succeeded}/{total} succeeded"
    health_report["scraping"]["Scraping status"] = scraping_status

    return health_report


@app.get("/ping",
         response_model=dict[str, str],
         description="Basic health check endpoint",
         tags=["Health Check"],
         responses={
             200: {
                 "description": "Successful Response",
                 "content": {
                     "application/json": {
                         "example": {
                             "status": "healthy",
                             "timestamp": "2023-10-01T12:34:56.789Z"
                         }
                     }
                 }
             }
         }
         )
async def ping(response: Response):
    """
    Simple health check endpoint to verify the API is up and running.
    Returns timestamp and status.
    """
    response.headers["Cache-Control"] = "no-cache, no-store, must-revalidate"
    return {
        "status": "healthy",
        "timestamp": datetime.datetime.utcnow().isoformat()
    }


app.include_router(quotes_router, prefix="/v1")
app.include_router(historical_prices_router, prefix="/v1")
app.include_router(indicators_router, prefix="/v1")
app.include_router(indices_router, prefix="/v1")
app.include_router(movers_router, prefix="/v1")
app.include_router(similar_quotes_router, prefix="/v1")
app.include_router(finance_news_router, prefix="/v1")
app.include_router(search_router, prefix="/v1")
app.include_router(sectors_router, prefix="/v1")
app.include_router(stream_router, prefix="/v1")
app.include_router(hours_router)
app.include_router(sockets_router)

handler = Mangum(app)

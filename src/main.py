import asyncio
import datetime
import os
import time
from collections import defaultdict
from contextlib import asynccontextmanager

from curl_cffi import requests
from dotenv import load_dotenv
from fastapi import FastAPI
from fastapi.encoders import jsonable_encoder
from fastapi.exceptions import RequestValidationError
from fastapi.middleware.cors import CORSMiddleware
from fastapi_injectable import cleanup_all_exit_stacks, register_app
from mangum import Mangum
from redis import Redis
from starlette import status
from starlette.responses import JSONResponse, Response

from src.connections import ConnectionManager, RedisConnectionManager
from src.context import RequestContextMiddleware
from src.models import Interval, Sector, TimeRange, ValidationErrorResponse
from src.routes import (
    finance_news_router,
    historical_prices_router,
    hours_router,
    indicators_router,
    indices_router,
    movers_router,
    quotes_router,
    search_router,
    sectors_router,
    similar_quotes_router,
    sockets_router,
    stream_router,
)
from src.security import RateLimitMiddleware
from src.services import (
    get_actives,
    get_gainers,
    get_historical,
    get_indices,
    get_losers,
    get_quotes,
    get_search,
    get_sector_details,
    get_sector_for_symbol,
    get_sectors,
    get_similar_quotes,
    get_simple_quotes,
    get_technical_indicators,
    scrape_general_news,
    scrape_news_for_quote,
)
from utils.constants import default_headers
from utils.dependencies import (
    RedisClient,
    YahooCookies,
    YahooCrumb,
    remove_proxy_whitelist,
    setup_proxy_whitelist,
)
from utils.yahoo_auth import setup_yahoo_auth, cleanup_yahoo_auth

load_dotenv()


@asynccontextmanager
async def lifespan(app: FastAPI):
    """
    FastAPI lifespan context manager that handles setup and cleanup.
    """
    await register_app(app)
    redis = None
    proxy_data = None

    # Set up curl_cffi session with Chrome impersonation
    app.state.session = requests.Session(impersonate="chrome")
    if default_headers:
        for key, value in default_headers.items():
            if value is not None:
                app.state.session.headers[key] = value

    # Set up auth refresh configuration
    app.state.auth_refresh_interval = 86400  # 24 hours in seconds

    try:
        # Setup proxy if needed
        if os.getenv("PROXY_TOKEN") and os.getenv("USE_PROXY", "False") == "True":
            proxy_data = await setup_proxy_whitelist()
            # Configure proxy for the curl_cffi session
            if os.getenv("PROXY_URL"):
                app.state.session.proxies = {
                    "http": os.getenv("PROXY_URL"),
                    "https": os.getenv("PROXY_URL")
                }

        # Setup Redis if configured
        if os.getenv("REDIS_URL"):
            redis = Redis.from_url(os.getenv("REDIS_URL"))
            app.state.redis = redis
            app.state.connection_manager = RedisConnectionManager(redis)
        else:
            app.state.redis = None
            app.state.connection_manager = ConnectionManager()

        # Setup Yahoo Finance authentication
        await setup_yahoo_auth(app)

        yield
    finally:
        app.state.connection_manager.close()

        # Clean up Yahoo authentication
        await cleanup_yahoo_auth(app)

        # Clean up proxy configuration
        await remove_proxy_whitelist(proxy_data)

        # Clean up Redis
        if redis:
            redis.close()

        await cleanup_all_exit_stacks()


app = FastAPI(
    title="FinanceQuery",
    version="1.7.1",
    description="FinanceQuery is a free and open-source API for financial data, retrieving data from web scraping & Yahoo Finance's Unofficial API.",
    servers=[
        {"url": "https://finance-query.onrender.com", "description": "Render server"},
        {"url": "https://43pk30s7aj.execute-api.us-east-2.amazonaws.com/prod", "description": "AWS server"},
        {"url": "http://127.0.0.1:8000", "description": "Local server"},
    ],
    contact={"name": "Harvey Tseng", "email": "harveytseng2@gmail.com"},
    license_info={
        "name": "MIT License",
        "identifier": "MIT",
    },
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],  # Allows all methods
    allow_headers=["*"],  # Allows all headers
    expose_headers=["X-RateLimit-Limit", "X-RateLimit-Remaining", "X-RateLimit-Reset"],
)

app.add_middleware(RequestContextMiddleware)

if os.getenv("USE_SECURITY", "False") == "True":
    app.add_middleware(RateLimitMiddleware)


@app.exception_handler(RequestValidationError)
async def request_validation_error_formatter(request, exc):
    reformatted_message = defaultdict(list)
    for pydantic_error in exc.errors():
        loc, msg = pydantic_error["loc"], pydantic_error["msg"]
        filtered_loc = loc[1:] if loc[0] in ("body", "query", "path") else loc
        field_string = ".".join(map(str, filtered_loc))  # nested fields with dot-notation
        reformatted_message[field_string].append(msg)

    error_response = ValidationErrorResponse(errors=reformatted_message)

    return JSONResponse(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, content=jsonable_encoder(error_response))


@app.get(
    path="/health",
    description="Detailed health check endpoint, checking the status of the API and its dependencies.",
    tags=["Health Check"],
    responses={
        200: {
            "description": "Successful Response",
            "content": {
                "application/json": {
                    "example": {
                        "status": "healthy",
                        "timestamp": "2025-02-13T18:27:37.508568",
                        "redis": {"status": "healthy", "latency_ms": 13.1},
                        "services": {
                            "status": "20/20 succeeded",
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
                            "Historical month prices": {"status": "succeeded"},
                            "Historical year prices": {"status": "succeeded"},
                            "Historical five year prices": {"status": "succeeded"},
                            "Search": {"status": "succeeded"},
                            "Summary Analysis": {"status": "succeeded"},
                        },
                    }
                }
            },
        }
    },
)
async def health(r: RedisClient, cookies: YahooCookies, crumb: YahooCrumb):
    """
    Comprehensive health check endpoint that verifies:
    - Basic API health
    - Redis connectivity
    - System time
    - Service dependencies
    """
    indices_task = get_indices(cookies, crumb)
    actives_task = get_actives()
    losers_task = get_losers()
    gainers_task = get_gainers()
    sectors_task = get_sectors()
    sector_by_symbol_task = get_sector_for_symbol("NVDA", cookies, crumb)
    sector_by_name_task = get_sector_details(Sector.TECHNOLOGY)
    news_task = scrape_general_news()
    news_by_symbol_task = scrape_news_for_quote("NVDA")
    scrape_etf_news_task = scrape_news_for_quote("QQQ")
    quotes_task = get_quotes(["NVDA", "QQQ", "GTLOX"], cookies, crumb)
    simple_quotes_task = get_simple_quotes(["NVDA", "QQQ", "GTLOX"], cookies, crumb)
    similar_equity_task = get_similar_quotes("NVDA", cookies, crumb)
    similar_etf_task = get_similar_quotes("QQQ", cookies, crumb)
    historical_data_task_day = get_historical("NVDA", TimeRange.DAY, Interval.ONE_MINUTE)
    historical_data_task_month = get_historical("NVDA", TimeRange.YTD, Interval.DAILY)
    historical_data_task_year = get_historical("NVDA", TimeRange.YEAR, Interval.DAILY)
    historical_data_task_five_years = get_historical("NVDA", TimeRange.FIVE_YEARS, Interval.MONTHLY)
    search_task = get_search("NVDA")
    summary_analysis_task = get_technical_indicators("NVDA", Interval.DAILY)

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
        ("Historical month prices", historical_data_task_month),
        ("Historical year prices", historical_data_task_year),
        ("Historical five year prices", historical_data_task_five_years),
        ("Search", search_task),
        ("Summary Analysis", summary_analysis_task),
    ]

    results = await asyncio.gather(*[task for _, task in tasks], return_exceptions=True)

    health_report = {
        "status": "healthy",
        "timestamp": datetime.datetime.now().isoformat(),
        "redis": {"status": "healthy", "latency_ms": 0},
        "services": {
            "status": "20/20 succeeded",
        },
    }

    # Check Redis
    if r:
        try:
            start_time = time.time()
            redis_ping = r.ping()
            health_report["redis"] = {
                "status": "healthy" if redis_ping else "unhealthy",
                "latency_ms": round((time.time() - start_time) * 1000, 2),
            }
        except Exception as e:
            health_report["dependencies"] = {"redis": {"status": "unhealthy", "error": str(e)}}
            health_report["status"] = "degraded"

    if not r:
        del health_report["redis"]

    total = len(tasks)
    succeeded = 0
    for (name, _), result in zip(tasks, results, strict=False):
        if isinstance(result, Exception):
            health_report["services"][name] = {"status": "FAILED", "ERROR": str(result)}
        else:
            health_report["services"][name] = {"status": "succeeded"}
            succeeded += 1

    service_status = f"{succeeded}/{total} succeeded"
    health_report["services"]["status"] = service_status

    return health_report


@app.get(
    path="/ping",
    description="Check if the server is reachable",
    tags=["Health Check"],
    responses={
        200: {
            "description": "Successful Response",
            "content": {"application/json": {"example": {"status": "healthy", "timestamp": "2023-10-01T12:34:56.789Z"}}},
        }
    },
)
async def ping(response: Response):
    """
    Simple health check endpoint to verify the API is up and running.
    Returns timestamp and status.
    """
    response.headers["Cache-Control"] = "no-cache, no-store, must-revalidate"
    return {"status": "healthy", "timestamp": datetime.datetime.now().isoformat()}


app.include_router(sockets_router)
app.include_router(hours_router, tags=["Hours"])
app.include_router(quotes_router, prefix="/v1", tags=["Quotes"])
app.include_router(historical_prices_router, prefix="/v1", tags=["Historical Prices"])
app.include_router(movers_router, prefix="/v1", tags=["Market Movers"])
app.include_router(similar_quotes_router, prefix="/v1", tags=["Quotes"])
app.include_router(finance_news_router, prefix="/v1", tags=["News"])
app.include_router(indices_router, prefix="/v1", tags=["Indices"])
app.include_router(sectors_router, prefix="/v1", tags=["Sectors"])
app.include_router(search_router, prefix="/v1", tags=["Search"])
app.include_router(indicators_router, prefix="/v1", tags=["Technical Indicators"])
app.include_router(stream_router, prefix="/v1", tags=["SSE"])

handler = Mangum(app)

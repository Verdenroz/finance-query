import asyncio
import datetime
import os
import time
from collections import defaultdict
from contextlib import asynccontextmanager

import requests
from dotenv import load_dotenv
from fastapi import FastAPI, Depends
from fastapi.encoders import jsonable_encoder
from fastapi.exceptions import RequestValidationError
from fastapi.middleware.cors import CORSMiddleware
from fastapi_injectable import cleanup_all_exit_stacks, register_app
from mangum import Mangum
from redis import Redis
from starlette import status
from starlette.responses import Response, JSONResponse

from src.connections import RedisConnectionManager
from src.context import RequestContextMiddleware
from src.dependencies import get_redis, _get_auth_data
from src.routes import (quotes_router, indices_router, movers_router, historical_prices_router,
                        similar_quotes_router, finance_news_router, indicators_router, search_router,
                        sectors_router, sockets_router, stream_router, hours_router)
from src.schemas import ValidationErrorResponse, Sector, TimePeriod, Interval
from src.security import RateLimitMiddleware
from src.services import (
    scrape_indices, scrape_actives, scrape_losers, scrape_gainers, get_sectors,
    get_sector_for_symbol, get_sector_details, scrape_general_news, scrape_news_for_quote, scrape_quotes,
    scrape_similar_quotes, get_historical, get_search, scrape_simple_quotes, get_summary_analysis
)

load_dotenv()


@asynccontextmanager
async def lifespan(app: FastAPI):
    """
    FastAPI lifespan context manager that handles proxy setup and cleanup.
    """
    await register_app(app)
    redis = None
    redis_connection_manager = None


    api_url = None
    proxy_header_token = None
    payload = None
    try:
        if os.getenv('PROXY_TOKEN') and os.getenv('USE_PROXY', 'False') == 'True':
            ip_response = requests.get("https://api.ipify.org/")
            ip = ip_response.text
            api_url = "https://api.brightdata.com/zone/whitelist"
            proxy_header_token = {
                "Authorization": f"Bearer {os.getenv('PROXY_TOKEN')}",
                "Content-Type": "application/json"
            }
            payload = {"ip": ip}
            requests.post(api_url, headers=proxy_header_token, json=payload)

        if os.getenv('USE_REDIS') == 'True':
            if os.getenv('REDIS_URL') is None:
                raise ValueError("REDIS_URL environment variable is not set.")

            redis = Redis.from_url(os.getenv('REDIS_URL'))
            redis_connection_manager = RedisConnectionManager(redis)
            app.state.redis = redis
            app.state.connection_manager = redis_connection_manager

        cookies, crumb = await _get_auth_data()
        app.state.cookies = cookies
        app.state.crumb = crumb

        yield
    finally:
        if api_url and proxy_header_token and payload:
            requests.delete(api_url, headers=proxy_header_token, json=payload)
        if redis:
            await redis.close()
            await redis_connection_manager.close()

        await cleanup_all_exit_stacks()


app = FastAPI(
    title="FinanceQuery",
    version="1.5.11",
    description="FinanceQuery is a simple API to query financial data."
                " It provides endpoints to get quotes, historical prices, indices,"
                " market movers, similar stocks, finance news, indicators, search, and sectors."
                " Please note if an admin key is not set, a rate limit of 2000/day will be applied to the request's ip"
                " address."
                " You are free to deploy your own instance of FinanceQuery to AWS and create your onw admin API key."
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

app.add_middleware(RequestContextMiddleware)

if os.getenv('USE_SECURITY', 'False') == 'True':
    app.add_middleware(RateLimitMiddleware)


@app.exception_handler(RequestValidationError)
async def request_validation_error_formatter(request, exc):
    reformatted_message = defaultdict(list)
    for pydantic_error in exc.errors():
        loc, msg = pydantic_error["loc"], pydantic_error["msg"]
        filtered_loc = loc[1:] if loc[0] in ("body", "query", "path") else loc
        field_string = ".".join(filtered_loc)  # nested fields with dot-notation
        reformatted_message[field_string].append(msg)

    error_response = ValidationErrorResponse(errors=reformatted_message)

    return JSONResponse(
        status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
        content=jsonable_encoder(error_response)
    )


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
async def health(r=Depends(get_redis)):
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
        redis_ping = r.ping()
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


@app.get(
    path="/ping",
    description="Check if the server is reachable",
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


app.include_router(sockets_router)
app.include_router(hours_router)
app.include_router(quotes_router, prefix="/v1")
app.include_router(historical_prices_router, prefix="/v1")
app.include_router(movers_router, prefix="/v1")
app.include_router(similar_quotes_router, prefix="/v1")
app.include_router(finance_news_router, prefix="/v1")
app.include_router(indices_router, prefix="/v1")
app.include_router(sectors_router, prefix="/v1")
app.include_router(search_router, prefix="/v1")
app.include_router(indicators_router, prefix="/v1")
app.include_router(stream_router, prefix="/v1")

handler = Mangum(app)

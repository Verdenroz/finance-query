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
from src.middleware import LoggingMiddleware, RateLimitMiddleware
from src.models import Interval, Sector, TimeRange, ValidationErrorResponse
from src.models.analysis import AnalysisType
from src.models.financials import Frequency, StatementType
from src.models.holders import HolderType
from src.routes import (
    earnings_transcript_router,
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
from src.routes.analysis import router as analysis_router
from src.routes.financials import router as financials_router
from src.routes.holders import router as holders_router
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
from src.services.analysis.get_analysis import get_analysis_data
from src.services.earnings_transcript.get_earnings_transcript import get_earnings_calls_list, get_earnings_transcript
from src.services.financials.get_financials import get_financial_statement
from src.services.holders.get_holders import get_holders_data
from src.utils.dependencies import (
    FinanceClient,
    RedisClient,
    remove_proxy_whitelist,
    setup_proxy_whitelist,
)
from src.utils.logging import configure_logging, get_logger
from src.utils.market import MarketSchedule
from src.utils.yahoo_auth import YahooAuthManager

load_dotenv()

# Configure logging first, before any other initialization
configure_logging()
logger = get_logger(__name__)

yahoo_auth_manager = YahooAuthManager()
curl_session = requests.Session(impersonate="chrome")


@asynccontextmanager
async def lifespan(app: FastAPI):
    """
    Creates shared resources (curl session, Redis, Yahoo auth) and
    ensures they are cleaned up when the server stops.
    """
    logger.info("Application startup initiated")
    try:
        await register_app(app)

        app.state.session = curl_session
        app.state.yahoo_auth_manager = yahoo_auth_manager

        proxy_data = None
        if os.getenv("PROXY_URL") and os.getenv("USE_PROXY", "False") == "True":
            logger.info("Setting up proxy configuration")
            try:
                proxy_data = await setup_proxy_whitelist()
                curl_session.proxies = {"http": os.getenv("PROXY_URL"), "https": os.getenv("PROXY_URL")}
                logger.info("Proxy configuration completed")
            except Exception as e:
                logger.critical("Failed to initialize proxy configuration", extra={"error": str(e)}, exc_info=True)
                raise

        # Redis (optional)
        if os.getenv("REDIS_URL"):
            logger.info("Initializing Redis connection", extra={"redis_url": os.getenv("REDIS_URL")})
            try:
                redis = Redis.from_url(os.getenv("REDIS_URL"))
                # Test the connection
                redis.ping()
                app.state.redis = redis
                app.state.connection_manager = RedisConnectionManager(redis)
                logger.info("Redis connection established")
            except Exception as e:
                logger.critical("Failed to initialize Redis connection", extra={"error": str(e)}, exc_info=True)
                raise
        else:
            logger.info("Redis not configured, using in-memory connection manager")
            redis = None
            app.state.redis = None
            app.state.connection_manager = ConnectionManager()

        # Prime Yahoo auth once so the first user request is fast
        logger.info("Initializing Yahoo authentication")
        try:
            await yahoo_auth_manager.refresh(proxy=os.getenv("PROXY_URL") if os.getenv("USE_PROXY", "False") == "True" else None)
            logger.info("Yahoo authentication initialized")
        except Exception as e:
            logger.critical("Failed to initialize Yahoo authentication", extra={"error": str(e)}, exc_info=True)
            raise

    except Exception as e:
        logger.critical("Critical failure during application startup", extra={"error": str(e)}, exc_info=True)
        raise

    logger.info("Application startup completed")
    try:
        yield
    except Exception as e:
        logger.critical("Critical application failure during lifespan", extra={"error": str(e), "error_type": type(e).__name__}, exc_info=True)
        raise
    finally:
        logger.info("Application shutdown initiated")
        try:
            await cleanup_all_exit_stacks()
            await app.state.connection_manager.close()
            if proxy_data:
                await remove_proxy_whitelist(proxy_data)
            if redis:
                redis.close()
            logger.info("Application shutdown completed")
        except Exception as e:
            logger.critical("Critical failure during application shutdown", extra={"error": str(e), "error_type": type(e).__name__}, exc_info=True)


app = FastAPI(
    title="FinanceQuery",
    version="1.10.1",
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

app.add_middleware(LoggingMiddleware)
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
                            "status": "28/28 succeeded",
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
                            "Technical Indicators": {"status": "succeeded"},
                            "Market Hours": {"status": "succeeded"},
                            "Annual Income Statement": {"status": "succeeded"},
                            "Quarterly Balance Sheet": {"status": "succeeded"},
                            "Institutional Holders": {"status": "succeeded"},
                            "Analyst Recommendations": {"status": "succeeded"},
                            "Price Targets": {"status": "succeeded"},
                            "Earnings Calls List": {"status": "succeeded"},
                            "Earnings Transcript": {"status": "succeeded"},
                        },
                    }
                }
            },
        }
    },
)
async def health(r: RedisClient, finance_client: FinanceClient):
    """
    Comprehensive health check endpoint that verifies:
    - Basic API health
    - Redis connectivity
    - System time
    - Service dependencies
    """
    logger.info("Health check initiated")
    indices_task = get_indices(finance_client)
    actives_task = get_actives()
    losers_task = get_losers()
    gainers_task = get_gainers()
    sectors_task = get_sectors()
    sector_by_symbol_task = get_sector_for_symbol(finance_client, "NVDA")
    sector_by_name_task = get_sector_details(Sector.TECHNOLOGY)
    news_task = scrape_general_news()
    news_by_symbol_task = scrape_news_for_quote("NVDA")
    scrape_etf_news_task = scrape_news_for_quote("QQQ")
    quotes_task = get_quotes(finance_client, ["NVDA", "QQQ", "GTLOX"])
    simple_quotes_task = get_simple_quotes(finance_client, ["NVDA", "QQQ", "GTLOX"])
    similar_equity_task = get_similar_quotes(finance_client, "NVDA")
    similar_etf_task = get_similar_quotes(finance_client, "QQQ")
    historical_data_task_day = get_historical(finance_client, "NVDA", TimeRange.DAY, Interval.ONE_MINUTE)
    historical_data_task_month = get_historical(finance_client, "NVDA", TimeRange.YTD, Interval.DAILY)
    historical_data_task_year = get_historical(finance_client, "NVDA", TimeRange.YEAR, Interval.DAILY)
    historical_data_task_five_years = get_historical(finance_client, "NVDA", TimeRange.FIVE_YEARS, Interval.MONTHLY)
    search_task = get_search(finance_client, "NVDA")
    summary_analysis_task = get_technical_indicators(finance_client, "NVDA", Interval.DAILY)
    market_hours_task = asyncio.create_task(asyncio.to_thread(MarketSchedule().get_market_status))
    income_statement_task = get_financial_statement(finance_client, "NVDA", StatementType.INCOME_STATEMENT, Frequency.ANNUAL)
    balance_sheet_task = get_financial_statement(finance_client, "NVDA", StatementType.BALANCE_SHEET, Frequency.QUARTERLY)
    institutional_holders_task = get_holders_data(finance_client, "NVDA", HolderType.INSTITUTIONAL)
    recommendations_task = get_analysis_data(finance_client, "NVDA", AnalysisType.RECOMMENDATIONS)
    price_targets_task = get_analysis_data(finance_client, "NVDA", AnalysisType.PRICE_TARGETS)
    earnings_calls_list_task = get_earnings_calls_list(finance_client, "AAPL")
    earnings_transcript_task = get_earnings_transcript(finance_client, "AAPL")

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
        ("Technical Indicators", summary_analysis_task),
        ("Market Hours", market_hours_task),
        ("Annual Income Statement", income_statement_task),
        ("Quarterly Balance Sheet", balance_sheet_task),
        ("Institutional Holders", institutional_holders_task),
        ("Analyst Recommendations", recommendations_task),
        ("Price Targets", price_targets_task),
        ("Earnings Calls List", earnings_calls_list_task),
        ("Earnings Transcript", earnings_transcript_task),
    ]

    results = await asyncio.gather(*[task for _, task in tasks], return_exceptions=True)

    health_report = {
        "status": "healthy",
        "timestamp": datetime.datetime.now().isoformat(),
        "redis": {"status": "healthy", "latency_ms": 0},
        "services": {
            "status": "28/28 succeeded",
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

    logger.info("Health check completed", extra={"status": health_report["status"], "services": service_status})
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
app.include_router(financials_router, prefix="/v1", tags=["Financials"])
app.include_router(holders_router, prefix="/v1", tags=["Holders"])
app.include_router(analysis_router, prefix="/v1", tags=["Analysis"])
app.include_router(earnings_transcript_router, prefix="/v1", tags=["Earnings Transcripts"])

handler = Mangum(app)

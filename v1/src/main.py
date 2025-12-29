import datetime
import os
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
from starlette.responses import JSONResponse

from src.connections import ConnectionManager, RedisConnectionManager
from src.context import RequestContextMiddleware
from src.middleware import LoggingMiddleware, RateLimitMiddleware
from src.models import ValidationErrorResponse
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
from src.utils.dependencies import (
    remove_proxy_whitelist,
    setup_proxy_whitelist,
)
from src.utils.logging import configure_logging, get_logger
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
    version="1.10.4",
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


async def _health_response():
    """Simple health check - status, version, and timestamp."""
    return {
        "status": "healthy",
        "version": "1.10.4",
        "timestamp": datetime.datetime.now(datetime.UTC).isoformat(),
    }


async def _ping_response():
    """Simple ping - just returns pong."""
    return {"message": "pong"}


# Root health endpoints (for Docker healthcheck)
@app.get(path="/health", tags=["Health Check"], description="Health check endpoint")
async def health():
    return await _health_response()


@app.get(path="/ping", tags=["Health Check"], description="Ping endpoint")
async def ping():
    return await _ping_response()


# Version-prefixed health endpoints (for nginx routing)
@app.get(path="/v1/health", tags=["Health Check"], description="Health check endpoint")
async def health_v1():
    return await _health_response()


@app.get(path="/v1/ping", tags=["Health Check"], description="Ping endpoint")
async def ping_v1():
    return await _ping_response()


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

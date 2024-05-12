from fastapi import FastAPI
from mangum import Mangum
from src.routes import (quotes_router, indices_router, movers_router, historical_prices_router,
                        similar_stocks_router, finance_news_router, indicators_router, search_router,
                        sectors_router)


app = FastAPI()

app.include_router(quotes_router, prefix="/v1")

app.include_router(historical_prices_router, prefix="/v1")

app.include_router(indicators_router, prefix="/v1")

app.include_router(indices_router, prefix="/v1")

app.include_router(movers_router, prefix="/v1")

app.include_router(similar_stocks_router, prefix="/v1")

app.include_router(finance_news_router, prefix="/v1")

app.include_router(search_router, prefix="/v1")

app.include_router(sectors_router, prefix="/v1")

handler = Mangum(app)

from fastapi import FastAPI
from mangum import Mangum
from src.routes import (quotes_router, indices_router, movers_router, historical_prices_router,
                        similar_stocks_router, finance_news_router, indicators_router)


app = FastAPI()

app.include_router(quotes_router)

app.include_router(historical_prices_router)

app.include_router(indicators_router)

app.include_router(indices_router)

app.include_router(movers_router)

app.include_router(similar_stocks_router)

app.include_router(finance_news_router)

handler = Mangum(app)

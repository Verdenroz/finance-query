from fastapi import FastAPI
from mangum import Mangum
from src.routes import quotes_router, indices_router, movers_router, historical_prices_router

app = FastAPI()

app.include_router(quotes_router)

app.include_router(historical_prices_router)

app.include_router(indices_router)

app.include_router(movers_router)

handler = Mangum(app)

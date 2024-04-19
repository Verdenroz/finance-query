from fastapi import FastAPI
from mangum import Mangum
from .routes.indices import router as indices_router
from .routes.movers import router as movers_router
from .routes.quotes import router as quotes_router

app = FastAPI()

app.include_router(quotes_router)

app.include_router(indices_router)

app.include_router(movers_router)

handler = Mangum(app)

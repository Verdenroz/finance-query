from fastapi import FastAPI
from mangum import Mangum
from src.routes.indices import router as indices_router
from src.routes.movers import router as movers_router

app = FastAPI()

app.include_router(indices_router)

app.include_router(movers_router)

handler = Mangum(app)

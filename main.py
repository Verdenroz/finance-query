from fastapi import FastAPI
from mangum import Mangum
from routes.indices import router as indices_router
from routes.actives import router as actives_router
from routes.gainers import router as gainers_router
from routes.losers import router as losers_router

app = FastAPI()

app.include_router(indices_router)

app.include_router(actives_router)

app.include_router(gainers_router)

app.include_router(losers_router)

handler = Mangum(app)

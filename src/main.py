# MIT License
#
# Copyright (c) 2024 Harvey Tseng
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.


from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from mangum import Mangum

from src.routes import (quotes_router, indices_router, movers_router, historical_prices_router,
                        similar_stocks_router, finance_news_router, indicators_router, search_router,
                        sectors_router, sockets_router, stream_router)

app = FastAPI(
    title="FinanceQuery",
    version="1.4.0",
    description="FinanceQuery is a simple API to query financial data."
                " It provides endpoints to get quotes, historical prices, indices,"
                " market movers, similar stocks, finance news, indicators, search, and sectors."
                " Please use FinanceQueryDemoAWSHT as the demo API key which is limited to 500 requests/day."
                " You are free to deploy your own instance of FinanceQuery to AWS and use your own API key."
                " If you are testing locally you can use the local server and will not need a key."
    ,
    servers=[
        {"url": "https://43pk30s7aj.execute-api.us-east-2.amazonaws.com/prod", "description": "Production server"},
        {"url": "http://127.0.0.1:8000", "description": "Local server"}
    ],
    contact={
        "name": "Harvey Tseng",
        "email": "harveytseng2@gmail.com"
    },
    license_info={
        "name": "MIT License",
        "identifier": "MIT",
    }
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Allows all origins (needed for Android app but should be restricted for web apps)
    allow_credentials=True,
    allow_methods=["*"],  # Allows all methods
    allow_headers=["*"],  # Allows all headers
)


app.include_router(quotes_router, prefix="/v1")

app.include_router(historical_prices_router, prefix="/v1")

app.include_router(indicators_router, prefix="/v1")

app.include_router(indices_router, prefix="/v1")

app.include_router(movers_router, prefix="/v1")

app.include_router(similar_stocks_router, prefix="/v1")

app.include_router(finance_news_router, prefix="/v1")

app.include_router(search_router, prefix="/v1")

app.include_router(sectors_router, prefix="/v1")

app.include_router(stream_router, prefix="/v1")

app.include_router(sockets_router)

handler = Mangum(app)

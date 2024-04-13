from pydantic import BaseModel, Field
from decimal import Decimal

class Index(BaseModel):
    name: str = Field(..., example="S&P 500", description="Name of the index")
    value: Decimal = Field(..., example=4300.00, description="Current value of the index")
    change: str = Field(..., example="+10.00", description="Change in the index value")
    percentChange: str = Field(..., example="+0.23%", description="Percentage change in the index value")

class MarketMover(BaseModel):
    symbol: str = Field(..., example="AAPL", description="Stock symbol")
    name: str = Field(..., example="Apple Inc.", description="Company name")
    price: Decimal = Field(..., example=145.00, description="Last traded price of the stock")
    change: str = Field(..., example="+1.00", description="Change in the stock price")
    percentChange: str = Field(..., example="+0.69%", description="Percentage change in the stock price")
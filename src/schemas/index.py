from pydantic import BaseModel, Field
from decimal import Decimal


class Index(BaseModel):
    name: str = Field(..., example="S&P 500", description="Name of the index")
    value: Decimal = Field(..., example=4300.00, description="Current value of the index")
    change: str = Field(..., example="+10.00", description="Change in the index value")
    percent_change: str = Field(..., example="+0.23%", description="Percentage change in the index value")

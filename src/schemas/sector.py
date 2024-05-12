from pydantic import BaseModel, Field


class Sector(BaseModel):
    sector: str = Field(..., title="Sector name", serialization_alias="Sector")
    day_return: str = Field(..., title="Day return", serialization_alias="1d")
    ytd_return: str = Field(..., title="Year to date return", serialization_alias="YTD")
    year_return: str = Field(..., title="Year return", serialization_alias="1y")
    three_year_return: str = Field(..., title="Three year return", serialization_alias="3y")
    five_year_return: str = Field(..., title="Five year return", serialization_alias="5y")
from enum import Enum
from typing import List, Union

from pydantic import BaseModel, Field, RootModel


class Sector(Enum):
    BASIC_MATERIALS = "Basic Materials"
    COMMUNICATION = "Communication Services"
    CONSUMER_CYCLICAL = "Consumer Cyclical"
    CONSUMER_DEFENSIVE = "Consumer Defensive"
    ENERGY = "Energy"
    FINANCIAL_SERVICES = "Financial Services"
    HEALTHCARE = "Healthcare"
    INDUSTRIALS = "Industrials"
    REAL_ESTATE = "Real Estate"
    TECHNOLOGY = "Technology"
    UTILITIES = "Utilities"


class MarketSector(BaseModel):
    sector: str = Field(
        default=...,
        title="Sector name",
        serialization_alias="sector"
    )
    day_return: str = Field(
        default=...,
        title="Day return",
        serialization_alias="dayReturn"
    )
    ytd_return: str = Field(
        default=...,
        title="Year to date return",
        serialization_alias="ytdReturn"
    )
    year_return: str = Field(
        default=...,
        title="Year return",
        serialization_alias="yearReturn"
    )
    three_year_return: str = Field(
        default=...,
        title="Three year return",
        serialization_alias="threeYearReturn"
    )
    five_year_return: str = Field(
        default=...,
        title="Five year return",
        serialization_alias="fiveYearReturn"
    )

    def dict(self, *args, **kwargs):
        base_dict = super().model_dump(*args, **kwargs, exclude_none=True, by_alias=True)
        return {k: v for k, v in base_dict.items() if v is not None}


class MarketSectorDetails(MarketSector):
    market_cap: str = Field(
        default=...,
        title="Market capitalization",
        serialization_alias="marketCap"
    )
    market_weight: str = Field(
        default=...,
        title="Market weight",
        serialization_alias="marketWeight"
    )
    industries: int = Field(
        default=...,
        title="Number of industries",
        serialization_alias="industries"
    )
    companies: int = Field(
        default=...,
        title="Number of companies",
        serialization_alias="companies"
    )
    top_industries: List[str] = Field(
        default=...,
        title="Top industries in the sector",
        serialization_alias="topIndustries"
    )
    top_companies: List[str] = Field(
        default=...,
        title="Top companies in the sector",
        serialization_alias="topCompanies"
    )

    def dict(self, *args, **kwargs):
        return super().dict(*args, **kwargs, exclude_none=True, by_alias=True)


class SectorsResponse(RootModel):
    root: Union[List[MarketSector], MarketSector, MarketSectorDetails]

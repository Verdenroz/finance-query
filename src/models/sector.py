from enum import Enum

from pydantic import BaseModel, Field, AliasChoices


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
    )
    day_return: str = Field(
        default=...,
        title="Day return",
        alias="dayReturn",
        validation_alias=AliasChoices("dayReturn", "day_return")
    )
    ytd_return: str = Field(
        default=...,
        title="Year to date return",
        alias="ytdReturn",
        validation_alias=AliasChoices("ytdReturn", "ytd_return")
    )
    year_return: str = Field(
        default=...,
        title="Year return",
        alias="yearReturn",
        validation_alias=AliasChoices("yearReturn", "year_return")
    )
    three_year_return: str = Field(
        default=...,
        title="Three year return",
        alias="threeYearReturn",
        validation_alias=AliasChoices("threeYearReturn", "three_year_return")
    )
    five_year_return: str = Field(
        default=...,
        title="Five year return",
        alias="fiveYearReturn",
        validation_alias=AliasChoices("fiveYearReturn", "five_year_return")
    )

    def dict(self, *args, **kwargs):
        return super().model_dump(*args, **kwargs, exclude_none=True, by_alias=True)


class MarketSectorDetails(MarketSector):
    market_cap: str = Field(
        default=...,
        title="Market capitalization",
        alias="marketCap",
        validation_alias=AliasChoices("marketCap", "market_cap")
    )
    market_weight: str = Field(
        default=...,
        title="Market weight",
        alias="marketWeight",
        validation_alias=AliasChoices("marketWeight", "market_weight")
    )
    industries: int = Field(
        default=...,
        title="Number of industries",
        alias="industries"
    )
    companies: int = Field(
        default=...,
        title="Number of companies",
        alias="companies"
    )
    top_industries: list[str] = Field(
        default=...,
        title="Top industries in the sector",
        alias="topIndustries",
        validation_alias=AliasChoices("topIndustries", "top_industries")
    )
    top_companies: list[str] = Field(
        default=...,
        title="Top companies in the sector",
        alias="topCompanies",
        validation_alias=AliasChoices("topCompanies", "top_companies")
    )

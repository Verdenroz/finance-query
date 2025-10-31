from datetime import datetime
from enum import Enum
from typing import Any, Optional

from pydantic import BaseModel, Field


class AnalysisType(str, Enum):
    RECOMMENDATIONS = "recommendations"
    UPGRADES_DOWNGRADES = "upgrades_downgrades"
    PRICE_TARGETS = "price_targets"
    EARNINGS_ESTIMATE = "earnings_estimate"
    REVENUE_ESTIMATE = "revenue_estimate"
    EARNINGS_HISTORY = "earnings_history"
    SUSTAINABILITY = "sustainability"


class RecommendationData(BaseModel):
    """Analyst recommendations data"""

    period: str = Field(default=..., description="Time period for recommendations")
    strong_buy: Optional[int] = Field(default=None, description="Number of strong buy recommendations")
    buy: Optional[int] = Field(default=None, description="Number of buy recommendations")
    hold: Optional[int] = Field(default=None, description="Number of hold recommendations")
    sell: Optional[int] = Field(default=None, description="Number of sell recommendations")
    strong_sell: Optional[int] = Field(default=None, description="Number of strong sell recommendations")


class UpgradeDowngrade(BaseModel):
    """Individual analyst upgrade/downgrade"""

    firm: str = Field(default=..., description="Analyst firm name")
    to_grade: Optional[str] = Field(default=None, description="New rating grade")
    from_grade: Optional[str] = Field(default=None, description="Previous rating grade")
    action: Optional[str] = Field(default=None, description="Action taken (upgrade/downgrade)")
    date: Optional[datetime] = Field(default=None, description="Date of the action")


class PriceTarget(BaseModel):
    """Analyst price targets"""

    current: Optional[float] = Field(default=None, description="Current price")
    mean: Optional[float] = Field(default=None, description="Mean price target")
    median: Optional[float] = Field(default=None, description="Median price target")
    low: Optional[float] = Field(default=None, description="Low price target")
    high: Optional[float] = Field(default=None, description="High price target")


class EarningsEstimate(BaseModel):
    """Earnings estimate data"""

    estimates: dict[str, Any] = Field(
        default=...,
        description="Earnings estimates data with periods as keys and estimate details as values",
        examples=[{"2024-12-31": {"avg": 6.5, "low": 6.0, "high": 7.0, "year_ago_eps": 5.5}}],
    )


class RevenueEstimate(BaseModel):
    """Revenue estimate data"""

    estimates: dict[str, Any] = Field(
        default=...,
        description="Revenue estimates data with periods as keys and estimate details as values",
        examples=[{"2024-12-31": {"avg": 400000000000, "low": 380000000000, "high": 420000000000, "year_ago_revenue": 365000000000}}],
    )


class EarningsHistoryItem(BaseModel):
    """Historical earnings data item"""

    date: datetime = Field(default=..., description="Earnings date")
    eps_actual: Optional[float] = Field(default=None, description="Actual EPS")
    eps_estimate: Optional[float] = Field(default=None, description="Estimated EPS")
    surprise: Optional[float] = Field(default=None, description="EPS surprise")
    surprise_percent: Optional[float] = Field(default=None, description="EPS surprise percentage")


class SustainabilityScores(BaseModel):
    """ESG (Environmental, Social, Governance) scores"""

    scores: dict[str, Any] = Field(
        default=...,
        description="Sustainability scores and metrics",
        examples=[{"environmentScore": 75, "socialScore": 80, "governanceScore": 85, "totalEsg": 80}],
    )


class AnalysisData(BaseModel):
    """Complete analysis data for a symbol"""

    symbol: str = Field(default=..., min_length=1, examples=["AAPL"], description="Stock symbol")
    analysis_type: AnalysisType = Field(default=..., examples=["recommendations"], description="Type of analysis data")

    # Optional fields based on analysis type
    recommendations: Optional[list[RecommendationData]] = Field(default=None, description="Analyst recommendations")
    upgrades_downgrades: Optional[list[UpgradeDowngrade]] = Field(default=None, description="Analyst upgrades and downgrades")
    price_targets: Optional[PriceTarget] = Field(default=None, description="Analyst price targets")
    earnings_estimate: Optional[EarningsEstimate] = Field(default=None, description="Earnings estimates")
    revenue_estimate: Optional[RevenueEstimate] = Field(default=None, description="Revenue estimates")
    earnings_history: Optional[list[EarningsHistoryItem]] = Field(default=None, description="Historical earnings data")
    sustainability: Optional[SustainabilityScores] = Field(default=None, description="ESG sustainability scores")

from typing import Optional
from pydantic import BaseModel, Field


class AnalystPriceTargets(BaseModel):
    """Analyst price targets for a stock"""
    symbol: str = Field(default=..., examples=["AAPL"], description="Stock symbol")
    current: Optional[float] = Field(default=None, examples=[150.00], description="Current price")
    low: Optional[float] = Field(default=None, examples=[120.00], description="Low price target")
    high: Optional[float] = Field(default=None, examples=[180.00], description="High price target")
    mean: Optional[float] = Field(default=None, examples=[150.00], description="Mean price target")
    median: Optional[float] = Field(default=None, examples=[148.00], description="Median price target")


class EstimateData(BaseModel):
    """Individual estimate data for a period"""
    period: str = Field(default=..., examples=["0q"], description="Period identifier (0q, +1q, 0y, +1y)")
    number_of_analysts: Optional[int] = Field(default=None, examples=[25], description="Number of analysts providing estimates")
    avg: Optional[float] = Field(default=None, examples=[2.50], description="Average estimate")
    low: Optional[float] = Field(default=None, examples=[2.20], description="Low estimate")
    high: Optional[float] = Field(default=None, examples=[2.80], description="High estimate")
    year_ago_eps: Optional[float] = Field(default=None, examples=[2.10], description="Year ago EPS")
    growth: Optional[float] = Field(default=None, examples=[0.19], description="Growth rate")


class EarningsEstimate(BaseModel):
    """Earnings estimates from analysts"""
    symbol: str = Field(default=..., examples=["AAPL"], description="Stock symbol")
    estimates: list[EstimateData] = Field(default_factory=list, description="List of earnings estimates by period")


class RevenueEstimate(BaseModel):
    """Revenue estimates from analysts"""
    symbol: str = Field(default=..., examples=["AAPL"], description="Stock symbol")
    estimates: list[EstimateData] = Field(default_factory=list, description="List of revenue estimates by period")


class EarningsHistoryItem(BaseModel):
    """Individual earnings history item"""
    quarter: str = Field(default=..., examples=["2024-03-31"], description="Quarter end date in ISO format")
    eps_estimate: Optional[float] = Field(default=None, examples=[2.50], description="EPS estimate")
    eps_actual: Optional[float] = Field(default=None, examples=[2.65], description="Actual EPS")
    eps_difference: Optional[float] = Field(default=None, examples=[0.15], description="Difference between actual and estimate")
    surprise_percent: Optional[float] = Field(default=None, examples=[6.0], description="Surprise percentage")


class EarningsHistory(BaseModel):
    """Historical earnings data"""
    symbol: str = Field(default=..., examples=["AAPL"], description="Stock symbol")
    history: list[EarningsHistoryItem] = Field(default_factory=list, description="List of historical earnings")


class EPSTrendItem(BaseModel):
    """Individual EPS trend data for a period"""
    period: str = Field(default=..., examples=["0q"], description="Period identifier")
    current: Optional[float] = Field(default=None, examples=[2.50], description="Current EPS estimate")
    seven_days_ago: Optional[float] = Field(default=None, examples=[2.48], description="EPS estimate 7 days ago")
    thirty_days_ago: Optional[float] = Field(default=None, examples=[2.45], description="EPS estimate 30 days ago")
    sixty_days_ago: Optional[float] = Field(default=None, examples=[2.42], description="EPS estimate 60 days ago")
    ninety_days_ago: Optional[float] = Field(default=None, examples=[2.40], description="EPS estimate 90 days ago")


class EPSTrend(BaseModel):
    """EPS trend over time"""
    symbol: str = Field(default=..., examples=["AAPL"], description="Stock symbol")
    trends: list[EPSTrendItem] = Field(default_factory=list, description="List of EPS trends by period")


class EPSRevisionItem(BaseModel):
    """Individual EPS revision data for a period"""
    period: str = Field(default=..., examples=["0q"], description="Period identifier")
    up_last_7days: Optional[int] = Field(default=None, examples=[5], description="Number of upward revisions in last 7 days")
    up_last_30days: Optional[int] = Field(default=None, examples=[12], description="Number of upward revisions in last 30 days")
    down_last_7days: Optional[int] = Field(default=None, examples=[2], description="Number of downward revisions in last 7 days")
    down_last_30days: Optional[int] = Field(default=None, examples=[3], description="Number of downward revisions in last 30 days")


class EPSRevisions(BaseModel):
    """EPS revisions by analysts"""
    symbol: str = Field(default=..., examples=["AAPL"], description="Stock symbol")
    revisions: list[EPSRevisionItem] = Field(default_factory=list, description="List of EPS revisions by period")


class GrowthEstimateItem(BaseModel):
    """Individual growth estimate data for a period"""
    period: str = Field(default=..., examples=["0q"], description="Period identifier")
    stock: Optional[float] = Field(default=None, examples=[0.15], description="Stock growth estimate")
    industry: Optional[float] = Field(default=None, examples=[0.12], description="Industry growth estimate")
    sector: Optional[float] = Field(default=None, examples=[0.13], description="Sector growth estimate")
    index: Optional[float] = Field(default=None, examples=[0.10], description="Index growth estimate")


class GrowthEstimates(BaseModel):
    """Growth estimates comparison"""
    symbol: str = Field(default=..., examples=["AAPL"], description="Stock symbol")
    estimates: list[GrowthEstimateItem] = Field(default_factory=list, description="List of growth estimates by period")

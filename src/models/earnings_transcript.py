from datetime import datetime
from typing import Any, Optional

from pydantic import BaseModel, Field, field_serializer


class EarningsTranscript(BaseModel):
    """Earnings call transcript data from defeatbeta-api"""

    symbol: str = Field(..., description="Stock symbol")
    quarter: str = Field(..., description="Quarter (e.g., 'Q1 2024')")
    year: int = Field(..., description="Year of the earnings call")
    date: datetime = Field(..., description="Date of the earnings call")
    transcript: str = Field(..., description="Full transcript text")
    participants: list[str] = Field(default_factory=list, description="List of participants")
    metadata: dict[str, Any] = Field(default_factory=dict, description="Additional metadata")

    @field_serializer("date")
    def serialize_date(self, date: datetime) -> str:
        """Serialize datetime to ISO format string"""
        return date.isoformat()


class EarningsTranscriptRequest(BaseModel):
    """Request model for earnings transcript endpoint"""

    symbol: str = Field(..., description="Stock symbol", examples=["AAPL", "TSLA", "MSFT"])
    quarter: Optional[str] = Field(None, description="Specific quarter (e.g., 'Q1')", examples=["Q1", "Q2", "Q3", "Q4"])
    year: Optional[int] = Field(None, description="Specific year", examples=[2024, 2023])

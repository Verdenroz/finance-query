from datetime import datetime
from enum import Enum
from typing import Any

from pydantic import BaseModel, Field


class Quarter(str, Enum):
    """Quarter enum for earnings calls"""

    Q1 = "Q1"
    Q2 = "Q2"
    Q3 = "Q3"
    Q4 = "Q4"


class EarningsCallListing(BaseModel):
    """Single earnings call metadata"""

    event_id: str = Field(..., description="Event ID for the earnings call", exclude=True)
    quarter: str | None = Field(None, description="Quarter (e.g., 'Q1', 'Q2')")
    year: int | None = Field(None, description="Year of the earnings call")
    title: str = Field(..., description="Title of the earnings call")
    url: str = Field(..., description="URL to the earnings call page")


class EarningsCallsList(BaseModel):
    """List of available earnings calls for a symbol"""

    symbol: str = Field(..., description="Stock symbol")
    earnings_calls: list[EarningsCallListing] = Field(..., description="List of available earnings calls")
    total: int = Field(..., description="Total number of earnings calls")


class TranscriptSpeaker(BaseModel):
    """Speaker information"""

    name: str = Field(..., description="Speaker name")
    role: str | None = Field(None, description="Speaker role/title")
    company: str | None = Field(None, description="Speaker company")


class TranscriptParagraph(BaseModel):
    """Single paragraph in the transcript"""

    speaker: str = Field(..., description="Speaker name")
    text: str = Field(..., description="Paragraph text")


class EarningsTranscript(BaseModel):
    """Full earnings call transcript"""

    symbol: str = Field(..., description="Stock symbol")
    quarter: str = Field(..., description="Quarter (e.g., 'Q1', 'Q2')")
    year: int = Field(..., description="Year of the earnings call")
    date: datetime = Field(..., description="Date of the earnings call")
    title: str = Field(..., description="Title of the earnings call")
    speakers: list[TranscriptSpeaker] = Field(..., description="List of speakers")
    paragraphs: list[TranscriptParagraph] = Field(..., description="Transcript paragraphs")
    metadata: dict[str, Any] = Field(default_factory=dict, description="Additional metadata")

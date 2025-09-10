from enum import Enum
from typing import Any, Optional
from datetime import datetime

from pydantic import BaseModel, Field


class HolderType(str, Enum):
    MAJOR = "major"
    INSTITUTIONAL = "institutional"
    MUTUALFUND = "mutualfund"
    INSIDER_TRANSACTIONS = "insider_transactions"
    INSIDER_PURCHASES = "insider_purchases"
    INSIDER_ROSTER = "insider_roster"


class MajorHoldersBreakdown(BaseModel):
    """Major holders breakdown data"""
    breakdown_data: dict[str, Any] = Field(
        default=...,
        description="Major holders breakdown with metrics as keys and values",
        examples=[{"insidersPercentHeld": 0.1, "institutionsPercentHeld": 0.85}]
    )


class InstitutionalHolder(BaseModel):
    """Individual institutional holder"""
    holder: str = Field(default=..., description="Institution name")
    shares: int = Field(default=..., description="Number of shares held")
    date_reported: datetime = Field(default=..., description="Date of last report")
    percent_out: Optional[float] = Field(default=None, description="Percentage of outstanding shares")
    value: Optional[int] = Field(default=None, description="Value of holdings")


class MutualFundHolder(BaseModel):
    """Individual mutual fund holder"""
    holder: str = Field(default=..., description="Fund name")
    shares: int = Field(default=..., description="Number of shares held")
    date_reported: datetime = Field(default=..., description="Date of last report")
    percent_out: Optional[float] = Field(default=None, description="Percentage of outstanding shares")
    value: Optional[int] = Field(default=None, description="Value of holdings")


class InsiderTransaction(BaseModel):
    """Individual insider transaction"""
    start_date: datetime = Field(default=..., description="Transaction start date")
    insider: str = Field(default=..., description="Insider name")
    position: str = Field(default=..., description="Insider position/relation")
    transaction: str = Field(default=..., description="Transaction description")
    shares: Optional[int] = Field(default=None, description="Number of shares")
    value: Optional[int] = Field(default=None, description="Transaction value")
    ownership: Optional[str] = Field(default=None, description="Ownership type (direct/indirect)")


class InsiderPurchase(BaseModel):
    """Insider purchase activity summary"""
    period: str = Field(default=..., description="Time period for the data")
    purchases_shares: Optional[int] = Field(default=None, description="Shares purchased")
    purchases_transactions: Optional[int] = Field(default=None, description="Number of purchase transactions")
    sales_shares: Optional[int] = Field(default=None, description="Shares sold")
    sales_transactions: Optional[int] = Field(default=None, description="Number of sale transactions")
    net_shares: Optional[int] = Field(default=None, description="Net shares purchased/sold")
    net_transactions: Optional[int] = Field(default=None, description="Net transactions")
    total_insider_shares: Optional[int] = Field(default=None, description="Total insider shares held")
    net_percent_insider_shares: Optional[float] = Field(default=None, description="Net % of insider shares")
    buy_percent_insider_shares: Optional[float] = Field(default=None, description="% buy shares")
    sell_percent_insider_shares: Optional[float] = Field(default=None, description="% sell shares")


class InsiderRosterMember(BaseModel):
    """Individual insider roster member"""
    name: str = Field(default=..., description="Insider name")
    position: str = Field(default=..., description="Position/relation")
    most_recent_transaction: Optional[str] = Field(default=None, description="Most recent transaction")
    latest_transaction_date: Optional[datetime] = Field(default=None, description="Latest transaction date")
    shares_owned_directly: Optional[int] = Field(default=None, description="Shares owned directly")
    shares_owned_indirectly: Optional[int] = Field(default=None, description="Shares owned indirectly")
    position_direct_date: Optional[datetime] = Field(default=None, description="Position direct date")


class HoldersData(BaseModel):
    """Complete holders data for a symbol"""
    symbol: str = Field(default=..., examples=["AAPL"], description="Stock symbol")
    holder_type: HolderType = Field(default=..., examples=["institutional"], description="Type of holder data")
    
    # Optional fields based on holder type
    major_breakdown: Optional[MajorHoldersBreakdown] = Field(default=None, description="Major holders breakdown")
    institutional_holders: Optional[list[InstitutionalHolder]] = Field(default=None, description="Institutional holders")
    mutualfund_holders: Optional[list[MutualFundHolder]] = Field(default=None, description="Mutual fund holders")
    insider_transactions: Optional[list[InsiderTransaction]] = Field(default=None, description="Insider transactions")
    insider_purchases: Optional[InsiderPurchase] = Field(default=None, description="Insider purchase activity")
    insider_roster: Optional[list[InsiderRosterMember]] = Field(default=None, description="Insider roster")

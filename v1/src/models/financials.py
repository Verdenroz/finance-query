from enum import Enum
from typing import Any

from pydantic import BaseModel, Field


class StatementType(str, Enum):
    INCOME_STATEMENT = "income"
    BALANCE_SHEET = "balance"
    CASH_FLOW = "cashflow"


class Frequency(str, Enum):
    ANNUAL = "annual"
    QUARTERLY = "quarterly"


class FinancialStatement(BaseModel):
    symbol: str = Field(default=..., examples=["AAPL"], description="Stock symbol")
    statement_type: StatementType = Field(default=..., examples=["income"], description="Type of financial statement")
    frequency: Frequency = Field(default=..., examples=["annual"], description="Frequency of the financial statement")
    statement: dict[str, dict[str, Any]] = Field(
        default=...,
        description="Financial statement data, with metrics as keys and a dictionary of dates and values",
    )

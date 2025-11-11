import time
from datetime import datetime
from typing import Any

from fastapi import HTTPException

from src.models.financials import FinancialStatement, Frequency, StatementType
from src.utils.dependencies import FinanceClient
from src.utils.yahoo_financials_constants import get_statement_fields


def _map_statement_type_to_key(statement_type: StatementType) -> str:
    """Map API statement type to internal key."""
    mapping = {
        StatementType.INCOME_STATEMENT: "income",
        StatementType.BALANCE_SHEET: "balance",
        StatementType.CASH_FLOW: "cashflow",
    }
    return mapping[statement_type]


def _parse_timeseries_data(timeseries_result: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    """
    Parse Yahoo Finance timeseries response into financial statement format.

    Args:
        timeseries_result: List of timeseries data from Yahoo Finance API

    Returns:
        Dictionary with field names as keys and date->value mappings as values
    """
    parsed_data: dict[str, dict[str, Any]] = {}

    for item in timeseries_result:
        # Get the metric name (e.g., 'annualTotalRevenue' -> 'TotalRevenue')
        metric_name = item.get("meta", {}).get("type", [""])[0]

        # Remove frequency prefix (annual/quarterly/trailing)
        for prefix in ["annual", "quarterly", "trailing"]:
            if metric_name.startswith(prefix):
                metric_name = metric_name[len(prefix) :]
                break

        # Extract timestamp data
        timestamp_data: dict[str, Any] = {}
        for datapoint in item.get(metric_name, []):
            as_of_date = datapoint.get("asOfDate")
            reported_value = datapoint.get("reportedValue", {})

            if as_of_date and reported_value is not None:
                # Convert date to ISO format if it's a datetime
                if isinstance(as_of_date, datetime):
                    date_key = as_of_date.isoformat()
                else:
                    date_key = str(as_of_date)

                # Handle raw value
                if isinstance(reported_value, dict):
                    timestamp_data[date_key] = reported_value.get("raw")
                else:
                    timestamp_data[date_key] = reported_value

        if timestamp_data:
            parsed_data[metric_name] = timestamp_data

    return parsed_data


async def get_financial_statement(finance_client: FinanceClient, symbol: str, statement_type: StatementType, freq: Frequency) -> FinancialStatement:
    """
    Get financial statement for a symbol using Yahoo Finance API directly.

    Args:
        finance_client: Yahoo Finance client
        symbol: Stock symbol
        statement_type: Type of financial statement (income, balance, cashflow)
        freq: Frequency (annual or quarterly)

    Returns:
        FinancialStatement object

    Raises:
        HTTPException: 404 if symbol not found, 500 for other errors
    """
    try:
        # Get appropriate fields for statement type
        statement_key = _map_statement_type_to_key(statement_type)
        fields = get_statement_fields(statement_key, freq.value)

        # Define time range (go back ~10 years)
        period2 = int(time.time())
        period1 = period2 - (10 * 365 * 24 * 60 * 60)  # 10 years ago

        # Fetch data from Yahoo Finance
        response = await finance_client.get_fundamentals_timeseries(
            symbol=symbol.upper(),
            period1=period1,
            period2=period2,
            types=fields,
        )

        # Parse the response
        timeseries_data = response.get("timeseries", {}).get("result", [])

        if not timeseries_data:
            raise HTTPException(status_code=404, detail=f"No {statement_type.value} data found for {symbol}")

        # Transform to our format
        parsed_statement = _parse_timeseries_data(timeseries_data)

        if not parsed_statement:
            raise HTTPException(status_code=404, detail=f"No {statement_type.value} data found for {symbol}")

        return FinancialStatement(
            symbol=symbol.upper(),
            statement_type=statement_type,
            frequency=freq,
            statement=parsed_statement,
        )

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to fetch financial statement: {str(e)}") from e

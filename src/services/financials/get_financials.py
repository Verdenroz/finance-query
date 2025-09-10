from fastapi import HTTPException

from src.models.financials import FinancialStatement, Frequency, StatementType
from src.yfinance_client.ticker import Ticker


def _map_frequency_to_yfinance(freq: Frequency) -> str:
    """Map API frequency to yfinance expected frequency values."""
    mapping = {
        Frequency.ANNUAL: "yearly",
        Frequency.QUARTERLY: "quarterly"
    }
    return mapping[freq]


async def get_financial_statement(
    symbol: str, statement_type: StatementType, freq: Frequency
) -> FinancialStatement:
    """
    Get financial statement for a symbol.
    :param symbol: the stock symbol
    :param statement_type: the type of financial statement to fetch
    :param freq: the frequency of the financial statement
    :return: a FinancialStatement object

    :raises HTTPException: with status code 404 if the symbol cannot be found, or 500 for any other error
    """
    ticker = Ticker(symbol)
    yf_freq = _map_frequency_to_yfinance(freq)

    try:
        if statement_type == StatementType.INCOME_STATEMENT:
            data = ticker.get_income_stmt(freq=yf_freq)
        elif statement_type == StatementType.BALANCE_SHEET:
            data = ticker.get_balance_sheet(freq=yf_freq)
        elif statement_type == StatementType.CASH_FLOW:
            data = ticker.get_cash_flow(freq=yf_freq)
        else:
            raise HTTPException(status_code=400, detail="Invalid statement type")

        if data is None or data.empty:
            raise HTTPException(status_code=404, detail=f"No data found for {symbol}")

        # Convert timestamp columns to string
        data.columns = data.columns.astype(str)

        return FinancialStatement(
            symbol=symbol,
            statement_type=statement_type,
            frequency=freq,
            statement=data.to_dict("index"),
        )
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e)) from e

from fastapi import HTTPException

from src.clients.defeatbeta_client import DefeatBetaClient
from src.models.financials import FinancialStatement, Frequency, StatementType


def _map_frequency_to_defeatbeta(freq: Frequency) -> str:
    """Map API frequency to defeatbeta-api expected frequency values."""
    mapping = {Frequency.ANNUAL: "annual", Frequency.QUARTERLY: "quarterly"}
    return mapping[freq]


def _map_statement_type_to_defeatbeta(statement_type: StatementType) -> str:
    """Map API statement type to defeatbeta-api expected statement type values."""
    mapping = {StatementType.INCOME_STATEMENT: "income_statement", StatementType.BALANCE_SHEET: "balance_sheet", StatementType.CASH_FLOW: "cash_flow"}
    return mapping[statement_type]


async def get_financial_statement(symbol: str, statement_type: StatementType, freq: Frequency) -> FinancialStatement:
    """
    Get financial statement for a symbol using defeatbeta-api.
    :param symbol: the stock symbol
    :param statement_type: the type of financial statement to fetch
    :param freq: the frequency of the financial statement
    :return: a FinancialStatement object

    :raises HTTPException: with status code 404 if the symbol cannot be found, or 500 for any other error
    """
    client = DefeatBetaClient()

    # Map enum values to defeatbeta-api expected strings
    defeatbeta_freq = _map_frequency_to_defeatbeta(freq)
    defeatbeta_statement_type = _map_statement_type_to_defeatbeta(statement_type)

    try:
        # Use DefeatBetaClient to fetch financial statement data
        data = await client.get_financial_statement(symbol, defeatbeta_statement_type, defeatbeta_freq)

        if data is None or not data.get("statement"):
            raise HTTPException(status_code=404, detail=f"No data found for {symbol}")

        return FinancialStatement(
            symbol=symbol,
            statement_type=statement_type,
            frequency=freq,
            statement=data["statement"],
        )
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e)) from e

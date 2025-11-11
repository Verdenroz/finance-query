from datetime import datetime
from typing import Any

from fastapi import HTTPException

from src.models.holders import (
    HoldersData,
    HolderType,
    InsiderPurchase,
    InsiderRosterMember,
    InsiderTransaction,
    InstitutionalHolder,
    MajorHoldersBreakdown,
    MutualFundHolder,
)
from src.utils.dependencies import FinanceClient

# Module mapping for Yahoo Finance quoteSummary API
HOLDER_TYPE_MODULES = {
    HolderType.MAJOR: ["majorHoldersBreakdown"],
    HolderType.INSTITUTIONAL: ["institutionOwnership"],
    HolderType.MUTUALFUND: ["fundOwnership"],
    HolderType.INSIDER_TRANSACTIONS: ["insiderTransactions"],
    HolderType.INSIDER_PURCHASES: ["netSharePurchaseActivity"],
    HolderType.INSIDER_ROSTER: ["insiderHolders"],
}

# Mapping of holder types to their data extraction and parsing configuration
HOLDER_TYPE_CONFIG = {
    HolderType.MAJOR: {
        "data_path": lambda d: d.get("majorHoldersBreakdown", {}),
        "parser": "_parse_major_breakdown",
        "field_name": "major_breakdown",
    },
    HolderType.INSTITUTIONAL: {
        "data_path": lambda d: d.get("institutionOwnership", {}).get("ownershipList", []),
        "parser": "_parse_institutional_holders",
        "field_name": "institutional_holders",
    },
    HolderType.MUTUALFUND: {
        "data_path": lambda d: d.get("fundOwnership", {}).get("ownershipList", []),
        "parser": "_parse_mutualfund_holders",
        "field_name": "mutualfund_holders",
    },
    HolderType.INSIDER_TRANSACTIONS: {
        "data_path": lambda d: d.get("insiderTransactions", {}).get("transactions", []),
        "parser": "_parse_insider_transactions",
        "field_name": "insider_transactions",
    },
    HolderType.INSIDER_PURCHASES: {
        "data_path": lambda d: d.get("netSharePurchaseActivity", {}),
        "parser": "_parse_insider_purchases",
        "field_name": "insider_purchases",
    },
    HolderType.INSIDER_ROSTER: {
        "data_path": lambda d: d.get("insiderHolders", {}).get("holders", []),
        "parser": "_parse_insider_roster",
        "field_name": "insider_roster",
    },
}


async def get_holders_data(finance_client: FinanceClient, symbol: str, holder_type: HolderType) -> HoldersData:
    """
    Get holders data for a symbol using Yahoo Finance API directly.

    Args:
        finance_client: Yahoo Finance client
        symbol: Stock symbol
        holder_type: Type of holders data to fetch

    Returns:
        HoldersData object

    Raises:
        HTTPException: 400 for invalid holder type, 404 if no data found, 500 for other errors
    """
    if holder_type not in HOLDER_TYPE_MODULES:
        raise HTTPException(status_code=400, detail="Invalid holder type")

    try:
        # Get the required modules for this holder type
        modules = HOLDER_TYPE_MODULES[holder_type]

        # Fetch data from Yahoo Finance
        response = await finance_client.get_quote_summary(symbol=symbol.upper(), modules=modules)

        # Extract the result
        result = response.get("quoteSummary", {}).get("result", [])
        if not result or len(result) == 0:
            raise HTTPException(status_code=404, detail=f"No {holder_type.value} data found for {symbol}")

        data = result[0]

        # Get configuration for this holder type
        config = HOLDER_TYPE_CONFIG.get(holder_type)
        if not config:
            raise HTTPException(status_code=400, detail=f"Invalid holder type: {holder_type}")

        # Extract data using configured path
        raw_data = config["data_path"](data)

        # Get the parser function by name
        parser_name = config["parser"]
        parser_func = globals()[parser_name]

        # Parse the data
        parsed_data = parser_func(raw_data)

        # Build HoldersData with the appropriate field
        return HoldersData(symbol=symbol.upper(), holder_type=holder_type, **{config["field_name"]: parsed_data})

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to fetch holders data: {str(e)}") from e


def _parse_major_breakdown(data: dict[str, Any]) -> MajorHoldersBreakdown:
    """Parse major holders breakdown data from Yahoo Finance API"""
    if not data:
        raise HTTPException(status_code=404, detail="No major holders breakdown data found")

    breakdown_data = {
        "insidersPercentHeld": data.get("insidersPercentHeld"),
        "institutionsPercentHeld": data.get("institutionsPercentHeld"),
        "institutionsFloatPercentHeld": data.get("institutionsFloatPercentHeld"),
        "institutionsCount": data.get("institutionsCount"),
    }

    # Remove None values
    breakdown_data = {k: v for k, v in breakdown_data.items() if v is not None}

    return MajorHoldersBreakdown(breakdown_data=breakdown_data)


def _parse_institutional_holders(holders_list: list[dict[str, Any]]) -> list[InstitutionalHolder]:
    """Parse institutional holders list from Yahoo Finance API"""
    if not holders_list:
        return []

    holders = []
    for holder_data in holders_list:
        # Convert epoch to datetime
        date_reported = holder_data.get("reportDate", {}).get("raw")
        if date_reported:
            date_reported = datetime.fromtimestamp(date_reported)
        else:
            date_reported = datetime.now()

        holder = InstitutionalHolder(
            holder=holder_data.get("organization", ""),
            shares=holder_data.get("position", {}).get("raw", 0),
            date_reported=date_reported,
            percent_out=holder_data.get("pctHeld", {}).get("raw"),
            value=holder_data.get("value", {}).get("raw"),
        )
        holders.append(holder)

    return holders


def _parse_mutualfund_holders(holders_list: list[dict[str, Any]]) -> list[MutualFundHolder]:
    """Parse mutual fund holders list from Yahoo Finance API"""
    if not holders_list:
        return []

    holders = []
    for holder_data in holders_list:
        # Convert epoch to datetime
        date_reported = holder_data.get("reportDate", {}).get("raw")
        if date_reported:
            date_reported = datetime.fromtimestamp(date_reported)
        else:
            date_reported = datetime.now()

        holder = MutualFundHolder(
            holder=holder_data.get("organization", ""),
            shares=holder_data.get("position", {}).get("raw", 0),
            date_reported=date_reported,
            percent_out=holder_data.get("pctHeld", {}).get("raw"),
            value=holder_data.get("value", {}).get("raw"),
        )
        holders.append(holder)

    return holders


def _parse_insider_transactions(transactions_list: list[dict[str, Any]]) -> list[InsiderTransaction]:
    """Parse insider transactions list from Yahoo Finance API"""
    if not transactions_list:
        return []

    transactions = []
    for trans_data in transactions_list:
        # Convert epoch to datetime
        start_date = trans_data.get("startDate", {}).get("raw")
        if start_date:
            start_date = datetime.fromtimestamp(start_date)
        else:
            start_date = datetime.now()

        transaction = InsiderTransaction(
            start_date=start_date,
            insider=trans_data.get("filerName", ""),
            position=trans_data.get("filerRelation", ""),
            transaction=trans_data.get("transactionText", ""),
            shares=trans_data.get("shares", {}).get("raw"),
            value=trans_data.get("value", {}).get("raw"),
            ownership=trans_data.get("ownership", ""),
        )
        transactions.append(transaction)

    return transactions


def _parse_insider_purchases(data: dict[str, Any]) -> InsiderPurchase:
    """Parse insider purchases data from Yahoo Finance API"""
    if not data:
        raise HTTPException(status_code=404, detail="No insider purchase data found")

    return InsiderPurchase(
        period=data.get("period", "Unknown"),
        purchases_shares=data.get("buyInfoShares", {}).get("raw"),
        purchases_transactions=data.get("buyInfoCount", {}).get("raw"),
        sales_shares=data.get("sellInfoShares", {}).get("raw"),
        sales_transactions=data.get("sellInfoCount", {}).get("raw"),
        net_shares=data.get("netInfoShares", {}).get("raw"),
        net_transactions=data.get("netInfoCount", {}).get("raw"),
        total_insider_shares=data.get("totalInsiderShares", {}).get("raw"),
        net_percent_insider_shares=data.get("netPercentInsiderShares", {}).get("raw"),
        buy_percent_insider_shares=data.get("buyPercentInsiderShares", {}).get("raw"),
        sell_percent_insider_shares=data.get("sellPercentInsiderShares", {}).get("raw"),
    )


def _parse_insider_roster(holders_list: list[dict[str, Any]]) -> list[InsiderRosterMember]:
    """Parse insider roster list from Yahoo Finance API"""
    if not holders_list:
        return []

    roster = []
    for holder_data in holders_list:
        # Convert epoch to datetime
        latest_trans_date = holder_data.get("latestTransDate", {}).get("raw")
        if latest_trans_date:
            latest_trans_date = datetime.fromtimestamp(latest_trans_date)

        position_direct_date = holder_data.get("positionDirectDate", {}).get("raw")
        if position_direct_date:
            position_direct_date = datetime.fromtimestamp(position_direct_date)

        member = InsiderRosterMember(
            name=holder_data.get("name", ""),
            position=holder_data.get("relation", ""),
            most_recent_transaction=holder_data.get("transactionDescription"),
            latest_transaction_date=latest_trans_date,
            shares_owned_directly=holder_data.get("positionDirect", {}).get("raw"),
            shares_owned_indirectly=holder_data.get("positionIndirect", {}).get("raw"),
            position_direct_date=position_direct_date,
        )
        roster.append(member)

    return roster

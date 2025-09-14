from fastapi import HTTPException
from datetime import datetime
import pandas as pd

from src.models.holders import (
    HoldersData, HolderType, MajorHoldersBreakdown, InstitutionalHolder, MutualFundHolder,
    InsiderTransaction, InsiderPurchase, InsiderRosterMember
)
import yfinance as yf


async def get_holders_data(symbol: str, holder_type: HolderType) -> HoldersData:
    """
    Get holders data for a symbol.
    :param symbol: the stock symbol
    :param holder_type: the type of holders data to fetch
    :return: a HoldersData object
    
    :raises HTTPException: with status code 404 if the symbol cannot be found, or 500 for any other error
    """
    ticker = yf.Ticker(symbol)
    
    try:
        if holder_type == HolderType.MAJOR:
            data = _parse_major_breakdown(ticker.major_holders)
            return HoldersData(
                symbol=symbol,
                holder_type=holder_type,
                major_breakdown=data
            )
        
        elif holder_type == HolderType.INSTITUTIONAL:
            data = _parse_institutional_holders(ticker.institutional_holders)
            return HoldersData(
                symbol=symbol,
                holder_type=holder_type,
                institutional_holders=data
            )
        
        elif holder_type == HolderType.MUTUALFUND:
            data = _parse_mutualfund_holders(ticker.mutualfund_holders)
            return HoldersData(
                symbol=symbol,
                holder_type=holder_type,
                mutualfund_holders=data
            )
        
        elif holder_type == HolderType.INSIDER_TRANSACTIONS:
            data = _parse_insider_transactions(ticker.insider_transactions)
            return HoldersData(
                symbol=symbol,
                holder_type=holder_type,
                insider_transactions=data
            )
        
        elif holder_type == HolderType.INSIDER_PURCHASES:
            data = _parse_insider_purchases(ticker.insider_purchases)
            return HoldersData(
                symbol=symbol,
                holder_type=holder_type,
                insider_purchases=data
            )
        
        elif holder_type == HolderType.INSIDER_ROSTER:
            data = _parse_insider_roster(ticker.insider_roster_holders)
            return HoldersData(
                symbol=symbol,
                holder_type=holder_type,
                insider_roster=data
            )
        
        else:
            raise HTTPException(status_code=400, detail="Invalid holder type")
            
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e)) from e


def _parse_major_breakdown(df: pd.DataFrame) -> MajorHoldersBreakdown:
    """Parse major holders breakdown DataFrame"""
    if df.empty:
        raise HTTPException(status_code=404, detail="No major holders breakdown data found")
    
    breakdown_data = df['Value'].to_dict()
    return MajorHoldersBreakdown(breakdown_data=breakdown_data)


def _parse_institutional_holders(df: pd.DataFrame) -> list[InstitutionalHolder]:
    """Parse institutional holders DataFrame"""
    if df.empty:
        return []
    
    holders = []
    for _, row in df.iterrows():
        holder = InstitutionalHolder(
            holder=row.get('Holder', ''),
            shares=row.get('Shares', 0),
            date_reported=row.get('Date Reported', datetime.now()),
            percent_out=row.get('% Out') if pd.notna(row.get('% Out')) else None,
            value=row.get('Value') if pd.notna(row.get('Value')) else None
        )
        holders.append(holder)
    
    return holders


def _parse_mutualfund_holders(df: pd.DataFrame) -> list[MutualFundHolder]:
    """Parse mutual fund holders DataFrame"""
    if df.empty:
        return []
    
    holders = []
    for _, row in df.iterrows():
        holder = MutualFundHolder(
            holder=row.get('Holder', ''),
            shares=row.get('Shares', 0),
            date_reported=row.get('Date Reported', datetime.now()),
            percent_out=row.get('% Out') if pd.notna(row.get('% Out')) else None,
            value=row.get('Value') if pd.notna(row.get('Value')) else None
        )
        holders.append(holder)
    
    return holders


def _parse_insider_transactions(df: pd.DataFrame) -> list[InsiderTransaction]:
    """Parse insider transactions DataFrame"""
    if df.empty:
        return []
    
    transactions = []
    for _, row in df.iterrows():
        transaction = InsiderTransaction(
            start_date=row.get('Start Date', datetime.now()),
            insider=row.get('Insider', ''),
            position=row.get('Position', ''),
            transaction=row.get('Transaction', ''),
            shares=row.get('Shares') if pd.notna(row.get('Shares')) else None,
            value=row.get('Value') if pd.notna(row.get('Value')) else None,
            ownership=row.get('Ownership') if pd.notna(row.get('Ownership')) else None
        )
        transactions.append(transaction)
    
    return transactions


def _parse_insider_purchases(df: pd.DataFrame) -> InsiderPurchase:
    """Parse insider purchases DataFrame"""
    if df.empty:
        raise HTTPException(status_code=404, detail="No insider purchase data found")
    
    # Extract period from column name
    period_col = df.columns[0] if len(df.columns) > 0 else "Insider Purchases"
    period = period_col.replace("Insider Purchases Last ", "") if "Insider Purchases Last" in period_col else "Unknown"
    
    # Map the data
    data_dict = {}
    if 'Shares' in df.columns:
        shares_data = df.set_index(df.columns[0])['Shares'].to_dict()
        data_dict = {
            'purchases_shares': shares_data.get('Purchases'),
            'sales_shares': shares_data.get('Sales'),
            'net_shares': shares_data.get('Net Shares Purchased (Sold)'),
            'total_insider_shares': shares_data.get('Total Insider Shares Held'),
            'net_percent_insider_shares': shares_data.get('% Net Shares Purchased (Sold)'),
            'buy_percent_insider_shares': shares_data.get('% Buy Shares'),
            'sell_percent_insider_shares': shares_data.get('% Sell Shares')
        }
    
    if 'Trans' in df.columns:
        trans_data = df.set_index(df.columns[0])['Trans'].to_dict()
        data_dict.update({
            'purchases_transactions': trans_data.get('Purchases'),
            'sales_transactions': trans_data.get('Sales'),
            'net_transactions': trans_data.get('Net Shares Purchased (Sold)')
        })
    
    return InsiderPurchase(period=period, **data_dict)


def _parse_insider_roster(df: pd.DataFrame) -> list[InsiderRosterMember]:
    """Parse insider roster DataFrame"""
    if df.empty:
        return []
    
    roster = []
    for _, row in df.iterrows():
        member = InsiderRosterMember(
            name=row.get('Name', ''),
            position=row.get('Position', ''),
            most_recent_transaction=row.get('Most Recent Transaction') if pd.notna(row.get('Most Recent Transaction')) else None,
            latest_transaction_date=row.get('Latest Transaction Date') if pd.notna(row.get('Latest Transaction Date')) else None,
            shares_owned_directly=row.get('Shares Owned Directly') if pd.notna(row.get('Shares Owned Directly')) else None,
            shares_owned_indirectly=row.get('Shares Owned Indirectly') if pd.notna(row.get('Shares Owned Indirectly')) else None,
            position_direct_date=row.get('Position Direct Date') if pd.notna(row.get('Position Direct Date')) else None
        )
        roster.append(member)
    
    return roster

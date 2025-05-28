from src.utils.dependencies import FinanceClient


async def get_yahoo_sector(finance_client: FinanceClient, symbol: str) -> str | None:
    summary_data = await finance_client.get_quote(symbol)
    summary_result = summary_data.get("quoteSummary", {}).get("result", [{}])[0]
    profile = summary_result.get("assetProfile", {})
    return profile.get("sector", None)

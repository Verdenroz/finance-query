from typing import Optional

from aiohttp import ClientSession

from src.constants import headers

global_session: Optional[ClientSession] = None

async def get_global_session() -> ClientSession:
    global global_session
    if global_session is None:
        global_session = ClientSession(max_field_size=30000, headers=headers)
    return global_session

async def close_global_session():
    global global_session
    if global_session is not None:
        await global_session.close()
        global_session = None
from src.constants import headers
from decimal import Decimal
from bs4 import BeautifulSoup
from fastapi.responses import JSONResponse
import requests
from src import schemas


async def scrape_indices():
    url = 'https://www.investing.com/indices/americas-indices'
    response = requests.get(url, headers=headers)

    if response.status_code == 200:
        soup = BeautifulSoup(response.content, 'html.parser')
        # Find the table element by its ID
        table = soup.find('table', {'id': 'indice_table_1'})
        indices = []
        # Check if the table is found
        if table:
            # Extract table rows
            rows = table.find_all('tr')

            # Iterate through rows
            for row in rows:
                # Extract table data from each row
                cells = row.find_all('td')
                if len(cells) > 5:  # Ensure there are enough cells
                    index_data = schemas.Index(
                        name=cells[1].text,
                        value=Decimal(cells[2].text.replace(',', '')),  # Convert string to Decimal
                        change=cells[5].text,
                        percent_change=cells[6].text,
                    )
                    indices.append(index_data)
            return indices
        else:
            return JSONResponse(status_code=500, content={"message": "Internal server error"})
    else:
        return JSONResponse(status_code=500, content={"message": "Failed to fetch data from the URL"})

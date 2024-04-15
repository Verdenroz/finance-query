from pydantic import BaseModel, Field


class News(BaseModel):
    title: str = Field(..., example="New iPhone released!", description="Title of the news article")
    link: str = Field(..., example="https://www.example.com", description="Link to the news article")
    source: str = Field(..., example="CNN", description="Source of the news article")
    time: str = Field(..., example="1 day ago", description="Time relative to current time when the news was published")

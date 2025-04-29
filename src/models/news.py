from pydantic import BaseModel, Field


class News(BaseModel):
    title: str = Field(default=..., examples=["New iPhone released!"], description="Title of the news article")
    link: str = Field(default=..., examples=["https://www.example.com"], description="Link to the news article")
    source: str = Field(default=..., examples=["CNN"], description="Source of the news article")
    img: str = Field(
        default=..., examples=["https://www.example.com/image.jpg"], description="Image of the news article"
    )
    time: str = Field(
        default=..., examples=["1 day ago"], description="Time relative to current time when the news was published"
    )

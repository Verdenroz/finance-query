from pydantic import BaseModel, Field


class ValidationErrorDetail(BaseModel):
    loc: list[str]
    msg: str
    type: str


class ValidationErrorResponse(BaseModel):
    detail: str = "Invalid request"
    errors: dict[str, list[str]] = Field(default={})

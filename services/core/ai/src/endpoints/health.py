from fastapi import APIRouter
from fastapi.responses import JSONResponse

router = APIRouter()


@router.get("/health", response_model=str)
async def health() -> JSONResponse:
    return JSONResponse(content="healthy", status_code=200)

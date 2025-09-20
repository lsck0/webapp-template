from src.endpoints.health import health


async def test_health() -> None:
    response = await health()
    assert isinstance(response, str)
    assert response == "healthy"

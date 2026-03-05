from src.endpoints.health import health


async def test_health() -> None:
    response = await health()
    assert response.body == b'"healthy"'

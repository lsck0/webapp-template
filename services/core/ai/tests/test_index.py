from src.main import index


async def test_index() -> None:
    response = await index()
    assert isinstance(response, str)
    assert response == "Hello, World!"

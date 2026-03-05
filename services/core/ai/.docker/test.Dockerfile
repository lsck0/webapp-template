FROM python:3.13-slim

WORKDIR /app

RUN pip install poetry

RUN apt-get update && apt-get install --assume-yes curl

COPY .. .

RUN poetry install

RUN poetry run flake8 src

RUN poetry run mypy src

RUN poetry run pytest

EXPOSE 80

CMD ["poetry", "run", "uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "80"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=30s \
    CMD curl --fail http://localhost:80/health || exit 1

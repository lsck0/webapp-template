FROM python:3.13-slim

WORKDIR /app

RUN pip install poetry

COPY .. .

RUN poetry install

RUN poetry run flake8 src

RUN poetry run mypy src

RUN poetry run pytest

CMD ["poetry", "run", "uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "80"]

EXPOSE 80

FROM python:3.13-slim

WORKDIR /app

RUN apt-get update && apt-get install -y curl

RUN pip install poetry

COPY .. .

RUN poetry install

CMD ["poetry", "run", "uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "80", "--reload"]

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD curl --fail --silent http://localhost/health || exit 1

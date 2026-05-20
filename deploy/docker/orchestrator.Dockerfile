FROM python:3.12-slim
RUN pip install poetry
WORKDIR /app
COPY orchestrator/pyproject.toml orchestrator/poetry.lock* ./
RUN poetry install --no-root --no-dev
COPY orchestrator/src ./src
RUN poetry install --no-dev
ENTRYPOINT ["poetry", "run", "cloakcli"]

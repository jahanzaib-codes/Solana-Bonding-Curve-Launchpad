# Multi-stage Dockerfile for Solana-Bonding-Curve-Launchpad
FROM python:3.11-slim as builder

WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN pip install --no-cache-dir --user -r requirements.txt

FROM python:3.11-slim as runner

WORKDIR /app
RUN groupadd -r launchpad && useradd -r -g launchpad launchpad

COPY --from=builder /root/.local /home/launchpad/.local
COPY --chown=launchpad:launchpad . /app

ENV PATH=/home/launchpad/.local/bin:$PATH \
    PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1

USER launchpad

ENTRYPOINT ["python", "cli.py"]
CMD ["demo"]

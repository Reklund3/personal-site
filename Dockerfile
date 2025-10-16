FROM lukemathwalker/cargo-chef:latest-rust-1.90-bookworm AS chef
WORKDIR /app
RUN apt update && apt install lld clang nodejs npm -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

ENV SQLX_OFFLINE=true

RUN cargo build --release --bin site

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update -y && \
    apt-get install -y --no-install-recommends openssl ca-certificates && \
    apt-get autoremove -y && \
    apt-get clean -y && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/site site
COPY --from=builder /app/ui/dist ui/dist
COPY configuration configuration

ENV APP_ENVIRONMENT=production

RUN groupadd -g 1001 appuser && \
    useradd -u 1001 -g 1001 appuser && \
    chown -R appuser:appuser site

USER appuser

ENTRYPOINT ["./site"]
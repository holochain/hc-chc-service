FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN curl -OL https://go.dev/dl/go1.23.5.linux-amd64.tar.gz \
  && tar -C /usr/local -xzf go1.23.5.linux-amd64.tar.gz \
  && rm go1.23.5.linux-amd64.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin hc-chc-service

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/hc-chc-service /usr/local/bin
EXPOSE 8080
ENTRYPOINT ["hc-chc-service", "--port", "8080", "--interface", "0.0.0.0"]

FROM rust:1.83.0 AS builder
WORKDIR /app

RUN curl -OL https://go.dev/dl/go1.23.5.linux-amd64.tar.gz \
  && tar -C /usr/local -xzf go1.23.5.linux-amd64.tar.gz \
  && rm go1.23.5.linux-amd64.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"
COPY . .
RUN cargo build --release --bin hc-chc-service

FROM debian:bookworm-slim AS runtime
WORKDIR /app

COPY --from=builder /app/target/release/hc-chc-service /usr/local/bin

EXPOSE 8080
ENTRYPOINT ["hc-chc-service", "--port", "8080", "--interface", "0.0.0.0"]

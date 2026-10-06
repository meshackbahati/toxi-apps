FROM rust:slim AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY templates ./templates
COPY public ./public
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /app/target/release/pulse /app/pulse
COPY templates ./templates
COPY public ./public
EXPOSE 3007
CMD ["/app/pulse"]

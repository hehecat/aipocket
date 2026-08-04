# syntax=docker/dockerfile:1
FROM rust:1-bookworm AS builder
WORKDIR /build
# Install pinned toolchain in its own layer (only invalidates when
# rust-toolchain.toml changes), so code changes don't re-download rustup.
COPY rust-toolchain.toml ./
RUN rustup toolchain install && rustup default stable
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations
ENV CARGO_TERM_COLOR=always \
    CARGO_PROFILE_RELEASE_LTO=false \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=8 \
    CARGO_PROFILE_RELEASE_STRIP=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/target \
    cargo build --release --locked -p aipocket && \
    cp /build/target/release/aipocket /build/aipocket

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/aipocket /usr/local/bin/aipocket
ENV RESULTS_DIR=/data/aipocket/results
VOLUME /data/aipocket
EXPOSE 8000
ENTRYPOINT ["aipocket"]
CMD ["serve", "--host", "0.0.0.0", "--port", "8000"]

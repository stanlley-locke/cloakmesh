FROM rust:1.75-slim AS builder
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY core ./core
COPY wasm ./wasm
COPY proto ./proto
RUN cargo build --release -p cloakmesh-core

FROM debian:bookworm-slim
RUN useradd -m -u 1000 cloak
COPY --from=builder /build/target/release/cloakmesh /usr/local/bin/cloakmesh
COPY configs/default.toml /etc/cloakmesh/default.toml
USER cloak
EXPOSE 4001
ENTRYPOINT ["cloakmesh"]
CMD ["--config", "/etc/cloakmesh/default.toml"]

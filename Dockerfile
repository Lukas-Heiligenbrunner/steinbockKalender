FROM rust:latest AS builder
COPY . .
RUN cargo build --release

FROM ubuntu:latest
COPY --from=builder ./target/release/steinbockschraubtermine ./target/release/steinbockschraubtermine

RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/* \

CMD ["/target/release/steinbockschraubtermine"]
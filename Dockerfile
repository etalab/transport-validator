FROM rust:latest as builder

RUN apt-get update && apt-get -y install musl-tools && rm -rf /var/lib/apt/lists/*
ADD . .
RUN rustup target add x86_64-unknown-linux-musl
ENV PKG_CONFIG_ALLOW_CROSS=1
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN strip /target/x86_64-unknown-linux-musl/release/main

FROM scratch
COPY --from=builder /target/x86_64-unknown-linux-musl/release/main validator
ENTRYPOINT ["/validator"]

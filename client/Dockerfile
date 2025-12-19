FROM rust:latest AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM ubuntu:24.04
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/client /usr/local/bin/client
ENV SECRETS_LOCATION=/run/secret/authorized_keys/id_ed25519 \
    SERVER_ADDR=0.0.0.0:3000
EXPOSE 22
CMD ["client", "--server"]

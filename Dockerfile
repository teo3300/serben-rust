FROM rust:1.67 as builder

WORKDIR /usr/src/rust-serben

COPY . .

RUN cargo install --path .

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/serben-rust /usr/local/bin/serben-rust

EXPOSE 8080/tcp

RUN mkdir content

CMD ["serben-rust"]
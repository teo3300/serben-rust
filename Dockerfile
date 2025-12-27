# Builder container
FROM rust:1.82 AS builder

WORKDIR /usr/src/rust-serben

COPY . .

RUN cargo install --path .

# Deploy container
FROM debian:bookworm-slim

RUN apt-get update 
RUN rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/serben-rust /usr/local/bin/serben-rust

RUN apt-get update && apt-get install -y imagemagick
RUN apt-get update && apt-get install -y pandoc
RUN apt-get update && apt-get install -y wget
# Wget used to perform healthcheck on the server

RUN mkdir content

RUN mkdir content/thumbnails

EXPOSE 8123/tcp

CMD ["serben-rust", "/content/"]

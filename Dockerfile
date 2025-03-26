FROM rust:1.70 AS builder

WORKDIR /usr/src/rust-serben

COPY . .

RUN cargo install --path .

FROM debian:bullseye-slim

RUN apt-get update 
RUN rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/serben-rust /usr/local/bin/serben-rust

RUN apt-get install imagemagick
RUN mkdir content

CMD ["serben-rust"]

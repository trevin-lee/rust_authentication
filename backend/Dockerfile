

# Use the official Rust image as the base image
FROM rust:latest

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src
COPY . .
RUN cargo build --release

CMD ["./target/release/backend"]
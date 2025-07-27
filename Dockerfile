
# syntax=docker/dockerfile:1

# Stage 1: Build the Rust application
FROM rust:1.79-slim-bookworm AS builder

WORKDIR /app

# Install openssl-dev and pkg-config for sqlx
RUN apt-get update && apt-get install -y     pkg-config     libssl-dev     postgresql-client     && rm -rf /var/lib/apt/lists/*

# Copy Cargo.toml and Cargo.lock to leverage Docker cache
COPY Cargo.toml Cargo.lock ./ 

# Build dummy project to cache dependencies
RUN mkdir src && echo "fn main() {}\n" > src/main.rs && cargo build --release

# Remove dummy project
RUN rm -rf src target/release/deps target/release/.fingerprint

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Stage 2: Create the final image
FROM debian:bookworm-slim

WORKDIR /app

# Install openssl and ca-certificates
RUN apt-get update && apt-get install -y     openssl     ca-certificates     && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/chat-server .

# Expose the port the application listens on
EXPOSE 8080

# Run the application
CMD ["./chat-server"]


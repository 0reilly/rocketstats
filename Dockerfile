# Select a Rust base image
FROM rust:latest as builder

# Create a new directory for the app
WORKDIR /usr/src/rocketstats_backend

# Copy the source code
COPY src ./src
COPY static ./static
COPY .env ./


# Copy Cargo.toml and Cargo.lock to the working directory
COPY Cargo.toml Cargo.lock ./

# Build the application
RUN cargo build --release

# Use a lightweight base image for the final stage
FROM debian:buster-slim

# Install required libraries
RUN apt-get update && \
    apt-get install -y libpq5 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/rocketstats_backend/target/release/rocketstats_backend /usr/local/bin/rocketstats_backend

# Copy the entrypoint script
COPY entrypoint.sh /app

# Set the entry point
ENTRYPOINT ["/app/entrypoint.sh"]

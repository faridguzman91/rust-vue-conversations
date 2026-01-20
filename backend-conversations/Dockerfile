<<<<<<< HEAD
# Multi-stage build for minimal image size
FROM rust:1.75 as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    audiowaveform \
    && rm -rf /var/lib/apt/lists/*

# Install SQLx CLI for database migrations
RUN cargo install sqlx-cli --no-default-features --features postgres

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
=======
FROM rust:latest

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev

# Install cargo-watch for hot-reloading
RUN cargo install cargo-watch

WORKDIR /app

# Copy dependency files first for caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs for initial build
>>>>>>> 45c06de0a4b6d46f90cdaef3f5e616cf5358a533
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only
RUN cargo build --release
<<<<<<< HEAD
RUN rm src/main.rs

# Copy real source code and rebuild
COPY src ./src

# Build the application
RUN cargo build --release

# Production stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    audiowaveform \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false conversations

# Copy the binary from builder stage
COPY --from=builder /app/target/release/backend-conversations /usr/local/bin/conversations-api

# Copy migrations if they exist
COPY migrations* /app/migrations/ 2>/dev/null || true

# Set correct ownership
RUN chown conversations:conversations /usr/local/bin/conversations-api

# Switch to app user
USER conversations

# Expose the port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info

# Run the application
CMD ["conversations-api"]
=======

# Copy real source code
COPY . .

# Overwrite dummy main.rs
RUN touch src/main.rs

# Expose backend port
EXPOSE 8080
>>>>>>> 45c06de0a4b6d46f90cdaef3f5e616cf5358a533

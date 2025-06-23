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
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only
RUN cargo build --release

# Copy real source code
COPY . .

# Overwrite dummy main.rs
RUN touch src/main.rs

# Expose backend port
EXPOSE 8080

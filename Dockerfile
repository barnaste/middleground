# Multi-stage build for optimal image size
FROM rust:1.88-slim-bookworm AS builder

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all crates
COPY api_gateway ./api_gateway
COPY auth ./auth
COPY db ./db
COPY shared ./shared
COPY ws ./ws
COPY cli ./cli
COPY source_validation ./source_validation

# Build the release binary in offline mode (uses .sqlx cache instead of live database)
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin api_gateway

# Runtime stage - minimal image
FROM debian:bookworm-slim

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from builder
COPY --from=builder /app/target/release/api_gateway /app/api_gateway

# Create a non-root user for security
RUN useradd -m -u 1001 appuser && chown -R appuser:appuser /app
USER appuser

# Expose the application port
EXPOSE 8080

# Run the binary
CMD ["/app/api_gateway"]

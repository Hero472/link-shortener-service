# Build stage
FROM rust:1.75-slim-bullseye as builder

# Install required dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /usr/src/app
COPY . .

# Build the application with release profile
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl1.1 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/api /usr/local/bin/api

# Create a non-root user
RUN useradd -m -U -s /bin/false appuser
USER appuser

# Expose the port
EXPOSE 8080

# Run the binary
CMD ["api"]

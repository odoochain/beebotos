# BeeBotOS Multi-stage Build
FROM rustlang/rust:nightly AS builder

WORKDIR /usr/src/beebotos

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    llvm \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY . .

# Build release binaries
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /usr/src/beebotos/target/release/gateway /app/
COPY --from=builder /usr/src/beebotos/target/release/beebot /app/
COPY --from=builder /usr/src/beebotos/target/release/beehub /app/

# Create data directory
RUN mkdir -p /app/data

# Expose ports
EXPOSE 8080 3000

# Default command
CMD ["./gateway"]

# BeeBotOS Justfile (alternative to Makefile)

# Default recipe
_default:
    @just --list

# Build all crates
build:
    cargo build --workspace --release

# Build in debug mode
debug:
    cargo build --workspace

# Run tests
test:
    cargo test --workspace --all-features

# Run specific test
test-filter FILTER:
    cargo test --workspace {{FILTER}}

# Format code
fmt:
    cargo fmt --all

# Check formatting
check-fmt:
    cargo fmt --all -- --check

# Run clippy
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Full check
check: check-fmt lint test

# Clean
clean:
    cargo clean
    rm -rf target/ dist/

# Documentation
doc:
    cargo doc --workspace --no-deps

# Watch mode
dev:
    cargo watch -x build -x test

# Install CLI
install:
    cargo install --path apps/cli --force

# Build Docker image
docker-build:
    docker build -t beebotos:latest -f docker/Dockerfile .

# Run with Docker Compose
docker-up:
    docker-compose -f docker/docker-compose.yml up -d

# Stop Docker containers
docker-down:
    docker-compose -f docker/docker-compose.yml down

# Deploy contracts
contract-deploy NETWORK="testnet":
    ./tools/deploy_contracts.py --network {{NETWORK}}

# Build contracts
contract-build:
    cd contracts && forge build

# Test contracts
contract-test:
    cd contracts && forge test

# Security audit
audit:
    cargo audit
    cd contracts && slither .

# Coverage
coverage:
    cargo tarpaulin --workspace --out Html --output-dir coverage

# Release release
release:
    cargo build --workspace --release --locked

# Setup dev environment
setup:
    ./scripts/setup-dev.sh

# Run local node
local-node:
    anvil --fork-url $MONAD_RPC_URL

# Generate API client
gen-api:
    openapi-generator generate -i docs/openapi.yaml -g rust -o generated/

# Benchmark
benchmark:
    ./tools/benchmark_runner.sh --all

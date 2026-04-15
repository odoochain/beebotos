# BeeBotOS Makefile

.PHONY: all build test lint fmt clean install doc docker dev help

# Default target
all: build

# Build all crates
build:
	cargo build --workspace --release

# Build with debug symbols
debug:
	cargo build --workspace

# Run all tests
test:
	cargo test --workspace --all-features

# Run integration tests only
test-integration:
	cargo test --workspace --test '*'

# Run unit tests only
test-unit:
	cargo test --workspace --lib

# Run benchmarks
bench:
	cargo bench --workspace

# Format code
fmt:
	cargo fmt --all

# Check formatting
check-fmt:
	cargo fmt --all -- --check

# Run clippy
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check everything
check: check-fmt lint test

# Generate documentation
doc:
	cargo doc --workspace --no-deps --open

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/
	rm -rf dist/

# Install locally
install:
	cargo install --path apps/cli --force

# Uninstall
uninstall:
	cargo uninstall beebot

# Build Docker images
docker:
	docker build -t beebotos:latest -f docker/Dockerfile .

# Run in Docker
docker-run:
	docker-compose -f docker/docker-compose.yml up -d

# Stop Docker containers
docker-stop:
	docker-compose -f docker/docker-compose.yml down

# Development mode (watch)
dev:
	cargo watch -x build -x test

# Setup development environment
setup:
	./scripts/setup-dev.sh

# Deploy contracts
contracts-deploy:
	./tools/deploy_contracts.py --network testnet

# Build contracts
contracts-build:
	cd contracts && forge build

# Test contracts
contracts-test:
	cd contracts && forge test

# Update dependencies
update:
	cargo update
	cd contracts && forge update

# Security audit
audit:
	cargo audit
	cd contracts && slither .

# Code coverage
coverage:
	cargo tarpaulin --workspace --out Html

# Release build
release:
	cargo build --workspace --release --locked

# Publish to crates.io (maintainers only)
publish:
	cargo publish -p beebot-core
	cargo publish -p beebot-kernel
	# ... etc

# Generate licenses
licenses:
	cargo license --json > licenses.json

# Help
help:
	@echo "BeeBotOS Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  build          - Build all crates (release)"
	@echo "  debug          - Build with debug symbols"
	@echo "  test           - Run all tests"
	@echo "  test-unit      - Run unit tests only"
	@echo "  test-integration - Run integration tests only"
	@echo "  bench          - Run benchmarks"
	@echo "  fmt            - Format code"
	@echo "  lint           - Run clippy"
	@echo "  check          - Run all checks (fmt, lint, test)"
	@echo "  doc            - Generate documentation"
	@echo "  clean          - Clean build artifacts"
	@echo "  install        - Install CLI locally"
	@echo "  uninstall      - Uninstall CLI"
	@echo "  docker         - Build Docker images"
	@echo "  docker-run     - Run with Docker Compose"
	@echo "  dev            - Development mode with watch"
	@echo "  setup          - Setup development environment"
	@echo "  contracts-deploy - Deploy smart contracts"
	@echo "  contracts-build - Build smart contracts"
	@echo "  contracts-test - Test smart contracts"
	@echo "  update         - Update dependencies"
	@echo "  audit          - Security audit"
	@echo "  coverage       - Generate coverage report"
	@echo "  release        - Production release build"
	@echo "  help           - Show this help"

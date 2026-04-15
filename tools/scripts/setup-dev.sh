#!/bin/bash
# Development environment setup script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[SETUP]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

# Check OS
OS="unknown"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    OS="windows"
fi

log "Setting up BeeBotOS development environment on $OS..."

# Check Rust
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    log "Found Rust: $RUST_VERSION"
else
    log "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Install Rust components
log "Installing Rust components..."
rustup component add rustfmt clippy

# Install target
log "Installing WASM target..."
rustup target add wasm32-wasi wasm32-unknown-unknown

# Install cargo tools
log "Installing cargo tools..."

# cargo-watch
if ! command -v cargo-watch &> /dev/null; then
    cargo install cargo-watch
fi

# cargo-expand
if ! command -v cargo-expand &> /dev/null; then
    cargo install cargo-expand
fi

# cargo-audit
if ! command -v cargo-audit &> /dev/null; then
    cargo install cargo-audit
fi

# cargo-tarpaulin (coverage)
if ! command -v cargo-tarpaulin &> /dev/null; then
    cargo install cargo-tarpaulin
fi

# Foundry (Solidity)
if ! command -v forge &> /dev/null; then
    log "Installing Foundry..."
    curl -L https://foundry.paradigm.xyz | bash
    source "$HOME/.foundry/bin/foundryup"
fi

# Node.js (for frontend)
if ! command -v node &> /dev/null; then
    warn "Node.js not found. Please install Node.js 18+ for frontend development"
fi

# Create directories
log "Creating project directories..."
mkdir -p "$PROJECT_ROOT/data"
mkdir -p "$PROJECT_ROOT/logs"
mkdir -p "$HOME/.beebotos"

# Copy config
if [ ! -f "$HOME/.beebotos/config.toml" ]; then
    log "Creating default config..."
    cp "$PROJECT_ROOT/config.example.toml" "$HOME/.beebotos/config.toml"
fi

# Git hooks
log "Setting up Git hooks..."
if [ -d "$PROJECT_ROOT/.git" ]; then
    cat > "$PROJECT_ROOT/.git/hooks/pre-commit" << 'EOF'
#!/bin/bash
# Pre-commit hook

echo "Running pre-commit checks..."

# Format check
cargo fmt --all -- --check || {
    echo "Formatting issues found. Run 'cargo fmt --all'"
    exit 1
}

# Clippy check
cargo clippy --workspace --all-targets --all-features -- -D warnings || {
    echo "Clippy warnings found"
    exit 1
}

echo "Pre-commit checks passed!"
EOF
    chmod +x "$PROJECT_ROOT/.git/hooks/pre-commit"
fi

# Build project
log "Building project..."
cd "$PROJECT_ROOT"
cargo build --workspace

# Run tests
log "Running initial tests..."
cargo test --workspace --quiet

# Print summary
echo ""
log "✨ Setup complete! ✨"
echo ""
info "Next steps:"
echo "  1. Configure: Edit ~/.beebotos/config.toml"
echo "  2. Run CLI: cargo run --bin beebot -- --help"
echo "  3. Run tests: cargo test --workspace"
echo "  4. Start dev: cargo watch -x build"
echo ""
info "Documentation:"
echo "  - README.md - Project overview"
echo "  - CONTRIBUTING.md - How to contribute"
echo "  - docs/ - Full documentation"
echo ""

#!/bin/bash
# Comprehensive check script for BeeBotOS

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

FAILED=0

log() {
    echo -e "${GREEN}[CHECK]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    FAILED=1
}

# Check Rust formatting
log "Checking Rust formatting..."
if cargo fmt --all -- --check; then
    log "✓ Rust formatting OK"
else
    error "✗ Rust formatting failed. Run 'cargo fmt --all'"
fi

# Check Rust clippy
log "Checking Rust clippy..."
if cargo clippy --workspace --all-targets --all-features -- -D warnings 2>/dev/null; then
    log "✓ Clippy OK"
else
    error "✗ Clippy warnings found"
fi

# Check Rust tests
log "Running Rust tests..."
if cargo test --workspace --all-features --quiet 2>/dev/null; then
    log "✓ Tests passed"
else
    error "✗ Tests failed"
fi

# Check documentation
log "Checking documentation..."
if cargo doc --workspace --no-deps 2>/dev/null | grep -q "warning"; then
    warn "Documentation warnings found"
else
    log "✓ Documentation OK"
fi

# Check Solidity formatting (if foundry is installed)
if command -v forge &> /dev/null; then
    log "Checking Solidity formatting..."
    if forge fmt --check 2>/dev/null; then
        log "✓ Solidity formatting OK"
    else
        warn "Solidity formatting issues. Run 'forge fmt'"
    fi
    
    log "Running Solidity tests..."
    if forge test --quiet 2>/dev/null; then
        log "✓ Solidity tests passed"
    else
        error "✗ Solidity tests failed"
    fi
else
    warn "Foundry not installed, skipping Solidity checks"
fi

# Check for TODO/FIXME in code
log "Checking for TODO/FIXME comments..."
TODO_COUNT=$(grep -r "TODO\|FIXME" --include="*.rs" crates/ | wc -l)
if [ "$TODO_COUNT" -gt 0 ]; then
    warn "Found $TODO_COUNT TODO/FIXME comments"
    grep -r "TODO\|FIXME" --include="*.rs" crates/ | head -5
else
    log "✓ No TODO/FIXME comments"
fi

# Check for unused dependencies (cargo-udeps)
if command -v cargo-udeps &> /dev/null; then
    log "Checking for unused dependencies..."
    if cargo udeps --workspace 2>/dev/null; then
        log "✓ No unused dependencies"
    else
        warn "Unused dependencies found"
    fi
else
    warn "cargo-udeps not installed"
fi

# Check license headers
log "Checking license headers..."
MISSING_LICENSE=0
for file in $(find crates -name "*.rs" -type f | head -20); do
    if ! head -1 "$file" | grep -q "SPDX-License-Identifier"; then
        MISSING_LICENSE=$((MISSING_LICENSE + 1))
    fi
done
if [ "$MISSING_LICENSE" -eq 0 ]; then
    log "✓ License headers OK"
else
    warn "$MISSING_LICENSE files missing license headers"
fi

# Summary
echo ""
if [ $FAILED -eq 0 ]; then
    log "All checks passed! ✨"
    exit 0
else
    error "Some checks failed. Please fix the issues above."
    exit 1
fi

#!/bin/bash
# BeeBotOS Cross-Compilation Script
#
# Usage: ./cross-compile.sh [target]
# 
# Supported targets:
#   x86_64      - x86_64 Linux (default)
#   aarch64     - ARM64 Linux
#   riscv64     - RISC-V 64 Linux
#   all         - Build for all targets

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Targets
TARGET_X86_64="x86_64-unknown-linux-gnu"
TARGET_AARCH64="aarch64-unknown-linux-gnu"
TARGET_RISCV64="riscv64gc-unknown-linux-gnu"

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if rustup target is installed
check_target() {
    local target=$1
    if ! rustup target list --installed | grep -q "^${target}$"; then
        log_warn "Target ${target} is not installed."
        log_info "Installing target ${target}..."
        rustup target add "${target}"
    fi
}

# Build for a specific target
build_target() {
    local target=$1
    local arch_name=$2
    
    log_info "Building for ${arch_name} (${target})..."
    
    cd "${PROJECT_ROOT}"
    
    # Check if cross-compilation tools are available
    case "${target}" in
        "aarch64-unknown-linux-gnu")
            if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
                log_error "aarch64-linux-gnu-gcc not found. Install with:"
                log_error "  Ubuntu/Debian: sudo apt-get install gcc-aarch64-linux-gnu"
                log_error "  Fedora: sudo dnf install gcc-aarch64-linux-gnu"
                return 1
            fi
            ;;
        "riscv64gc-unknown-linux-gnu")
            if ! command -v riscv64-linux-gnu-gcc &> /dev/null; then
                log_error "riscv64-linux-gnu-gcc not found. Install with:"
                log_error "  Ubuntu/Debian: sudo apt-get install gcc-riscv64-linux-gnu"
                return 1
            fi
            ;;
    esac
    
    # Check target
    check_target "${target}"
    
    # Build kernel crate
    log_info "Building beebotos-kernel..."
    cargo build --target "${target}" -p beebotos-kernel --release
    
    # Build all crates
    log_info "Building all crates..."
    cargo build --target "${target}" --release
    
    log_info "Build for ${arch_name} completed successfully!"
    echo ""
}

# Build for native architecture
build_native() {
    log_info "Building for native architecture..."
    cd "${PROJECT_ROOT}"
    cargo build --release
    log_info "Native build completed!"
}

# Show architecture info
show_arch_info() {
    log_info "Supported Architectures:"
    echo ""
    echo "  x86_64 (AMD64):"
    echo "    - Intel Core series"
    echo "    - AMD Ryzen/EPYC"
    echo "    - Target: ${TARGET_X86_64}"
    echo ""
    echo "  aarch64 (ARM64):"
    echo "    - Apple Silicon (M1/M2/M3)"
    echo "    - AWS Graviton"
    echo "    - Raspberry Pi 4/5"
    echo "    - Target: ${TARGET_AARCH64}"
    echo ""
    echo "  riscv64 (RISC-V 64):"
    echo "    - SiFive HiFive"
    echo "    - VisionFive"
    echo "    - Allwinner D1"
    echo "    - Target: ${TARGET_RISCV64}"
    echo ""
}

# Main
main() {
    local target=${1:-native}
    
    case "${target}" in
        "x86_64")
            build_target "${TARGET_X86_64}" "x86_64"
            ;;
        "aarch64"|"arm64")
            build_target "${TARGET_AARCH64}" "aarch64"
            ;;
        "riscv64"|"riscv")
            build_target "${TARGET_RISCV64}" "riscv64"
            ;;
        "all")
            log_info "Building for all targets..."
            build_target "${TARGET_X86_64}" "x86_64"
            build_target "${TARGET_AARCH64}" "aarch64"
            build_target "${TARGET_RISCV64}" "riscv64"
            log_info "All builds completed!"
            ;;
        "native"|"")
            build_native
            ;;
        "info")
            show_arch_info
            ;;
        "help"|"-h"|"--help")
            echo "BeeBotOS Cross-Compilation Script"
            echo ""
            echo "Usage: $0 [target]"
            echo ""
            echo "Targets:"
            echo "  x86_64      - Build for x86_64 Linux (AMD/Intel)"
            echo "  aarch64     - Build for ARM64 Linux (Apple Silicon, AWS Graviton, etc.)"
            echo "  riscv64     - Build for RISC-V 64 Linux"
            echo "  all         - Build for all supported targets"
            echo "  native      - Build for native architecture (default)"
            echo "  info        - Show architecture information"
            echo "  help        - Show this help message"
            echo ""
            show_arch_info
            ;;
        *)
            log_error "Unknown target: ${target}"
            echo "Run '$0 help' for usage information."
            exit 1
            ;;
    esac
}

main "$@"

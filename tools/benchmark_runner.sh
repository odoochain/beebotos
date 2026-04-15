#!/bin/bash
# Benchmark runner for BeeBotOS

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/benchmark_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +%H:%M:%S)]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Setup
setup() {
    log "Setting up benchmark environment..."
    mkdir -p "$RESULTS_DIR"
    
    # Check dependencies
    if ! command -v cargo &> /dev/null; then
        error "Rust/Cargo not found"
        exit 1
    fi
}

# Run cargo benchmarks
run_cargo_benchmarks() {
    log "Running Cargo benchmarks..."
    
    cd "$PROJECT_ROOT"
    
    # Run all benchmarks
    cargo bench --workspace 2>&1 | tee "$RESULTS_DIR/cargo_bench_$TIMESTAMP.txt"
    
    # Copy results
    if [ -d "target/criterion" ]; then
        cp -r target/criterion "$RESULTS_DIR/criterion_$TIMESTAMP"
    fi
}

# Run custom benchmarks
run_custom_benchmarks() {
    log "Running custom benchmarks..."
    
    cd "$PROJECT_ROOT"
    
    # Agent spawn rate
    log "  - Agent spawn rate..."
    cargo run --example bench_agent_spawn --release 2>&1 | tee "$RESULTS_DIR/agent_spawn_$TIMESTAMP.txt"
    
    # Message throughput
    log "  - Message throughput..."
    cargo run --example bench_message_throughput --release 2>&1 | tee "$RESULTS_DIR/message_throughput_$TIMESTAMP.txt"
    
    # Memory usage
    log "  - Memory usage..."
    cargo run --example bench_memory --release 2>&1 | tee "$RESULTS_DIR/memory_$TIMESTAMP.txt"
}

# Run system benchmarks
run_system_benchmarks() {
    log "Running system benchmarks..."
    
    # CPU info
    if command -v lscpu &> /dev/null; then
        lscpu > "$RESULTS_DIR/cpu_info_$TIMESTAMP.txt"
    fi
    
    # Memory info
    if [ -f /proc/meminfo ]; then
        cat /proc/meminfo > "$RESULTS_DIR/memory_info_$TIMESTAMP.txt"
    fi
    
    # Disk info
    df -h > "$RESULTS_DIR/disk_info_$TIMESTAMP.txt"
}

# Generate report
generate_report() {
    log "Generating report..."
    
    REPORT_FILE="$RESULTS_DIR/report_$TIMESTAMP.md"
    
    cat > "$REPORT_FILE" << EOF
# Benchmark Report

**Date:** $(date)
**Commit:** $(git rev-parse --short HEAD 2>/dev/null || echo "N/A")

## System Information

$(if [ -f "$RESULTS_DIR/cpu_info_$TIMESTAMP.txt" ]; then echo "- CPU: $(grep 'Model name' "$RESULTS_DIR/cpu_info_$TIMESTAMP.txt" | cut -d: -f2 | xargs)"; fi)
$(if [ -f "$RESULTS_DIR/memory_info_$TIMESTAMP.txt" ]; then echo "- Memory: $(grep MemTotal "$RESULTS_DIR/memory_info_$TIMESTAMP.txt" | awk '{print $2, $3}')"; fi)

## Results

### Agent Spawn Rate
\`\`\`
$(tail -20 "$RESULTS_DIR/agent_spawn_$TIMESTAMP.txt" 2>/dev/null || echo "No data")
\`\`\`

### Message Throughput
\`\`\`
$(tail -20 "$RESULTS_DIR/message_throughput_$TIMESTAMP.txt" 2>/dev/null || echo "No data")
\`\`\`

### Memory Usage
\`\`\`
$(tail -20 "$RESULTS_DIR/memory_$TIMESTAMP.txt" 2>/dev/null || echo "No data")
\`\`\`

## Files

- Full results: \`$RESULTS_DIR\`
EOF

    log "Report saved to: $REPORT_FILE"
}

# Cleanup
cleanup() {
    log "Cleaning up..."
    # Keep only last 10 benchmark runs
    ls -t "$RESULTS_DIR" | tail -n +50 | xargs -I {} rm -rf "$RESULTS_DIR/{}"
}

# Main
main() {
    local run_all=false
    local run_cargo=false
    local run_custom=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --all)
                run_all=true
                shift
                ;;
            --cargo)
                run_cargo=true
                shift
                ;;
            --custom)
                run_custom=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --all      Run all benchmarks"
                echo "  --cargo    Run Cargo benchmarks only"
                echo "  --custom   Run custom benchmarks only"
                echo "  --help     Show this help"
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Default to all if no specific option
    if [ "$run_all" = false ] && [ "$run_cargo" = false ] && [ "$run_custom" = false ]; then
        run_all=true
    fi
    
    setup
    
    if [ "$run_all" = true ] || [ "$run_cargo" = true ]; then
        run_cargo_benchmarks
    fi
    
    if [ "$run_all" = true ] || [ "$run_custom" = true ]; then
        run_custom_benchmarks
    fi
    
    run_system_benchmarks
    generate_report
    cleanup
    
    log "Benchmarks complete!"
}

main "$@"

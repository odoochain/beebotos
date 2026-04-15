# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- WebSocket transport for A2A protocol
- TEE integration foundation
- Prometheus metrics exporter
- Enhanced skill marketplace

### Changed
- Improved scheduler performance
- Reduced memory footprint by 15%

### Fixed
- Memory leak in agent runtime
- Race condition in capability delegation

## [2.0.0] - 2025-03-15

### Added
- **Kernel**: Preemptive multitasking scheduler with 5 algorithms
- **Capability System**: 11-tier permission model with time decay
- **Social Brain**: NEAT evolution and PAD emotional model
- **DAO Governance**: Hybrid human + agent governance
- **A2A Protocol**: Agent-to-agent communication
- **WASM Runtime**: Sandboxed skill execution
- **CLI**: Complete command-line interface
- **Gateway API**: REST and WebSocket endpoints
- **Web Frontend**: Leptos-based dashboard
- **Documentation**: Comprehensive guides and tutorials

### Security
- Capability-based access control
- Cryptographic signatures
- Secure enclave support (TEE)

## [1.2.0] - 2024-12-01

### Added
- Basic agent runtime
- Memory management
- Message passing

### Changed
- Refactored core types

## [1.1.0] - 2024-09-15

### Added
- Initial blockchain integration
- Basic CLI commands

## [1.0.0] - 2024-06-01

### Added
- Initial release
- Core data structures
- Basic scheduler

---

## Version Legend

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

## Tagging

```bash
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

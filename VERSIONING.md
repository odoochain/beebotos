# Versioning Strategy

## Semantic Versioning

BeeBotOS follows [Semantic Versioning 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]

Example: 2.0.0-beta.1+build.123
```

### Version Components

| Component | Meaning | Example |
|-----------|---------|---------|
| MAJOR | Breaking changes | 2.0.0 |
| MINOR | New features | 2.1.0 |
| PATCH | Bug fixes | 2.0.1 |
| PRERELEASE | Alpha/beta/rc | 2.0.0-beta.1 |
| BUILD | Build metadata | +build.20240310 |

### Release Types

| Type | Format | Stability | Use Case |
|------|--------|-----------|----------|
| Alpha | x.y.z-alpha.n | Unstable | Internal testing |
| Beta | x.y.z-beta.n | Feature-complete | Public testing |
| RC | x.y.z-rc.n | Release candidate | Final validation |
| Stable | x.y.z | Production ready | General use |

## Crate Versioning

All workspace crates follow the same major version:

```toml
# Kernel
[package]
name = "beebot-kernel"
version = "1.0.0"

# Agents
[package]
name = "beebot-agents"
version = "1.0.0"
```

### Dependency Constraints

```toml
[dependencies]
# Internal: exact version
beebot-core = { path = "../core", version = "=2.0.0" }

# External: compatible version
serde = { version = "1.0", features = ["derive"] }
```

## API Stability

### Public API

Marked with `#[stable]`:

```rust
/// Spawn a new agent
/// 
/// # Stability
/// Stable since 2.0.0
#[stable(feature = "agent_spawn", since = "1.0.0")]
pub async fn spawn_agent(config: AgentConfig) -> Result<AgentId> {
    // ...
}
```

### Experimental API

Marked with `#[unstable]`:

```rust
/// Advanced scheduling (experimental)
#[unstable(feature = "advanced_scheduling")]
pub async fn set_scheduler_algorithm(alg: Algorithm) {
    // ...
}
```

## Upgrade Policy

### Within Major Version

- **2.0.x → 2.0.y**: Drop-in replacement
- **2.0.x → 2.1.0**: May need minor changes

### Major Version Upgrades

- **1.x → 2.0**: Follow [Migration Guide](migrations/v1_to_v2.md)
- Breaking changes documented in CHANGELOG

### Deprecation Timeline

| Phase | Duration | Action |
|-------|----------|--------|
| Announce | N/A | Mark as deprecated |
| Soft Deprecation | 2 minor versions | Warning on use |
| Hard Deprecation | 1 major version | Removal |

## Release Schedule

| Release | Frequency | Example |
|---------|-----------|---------|
| Patch | Bi-weekly | 2.0.1, 2.0.2 |
| Minor | Monthly | 2.1.0, 2.2.0 |
| Major | Quarterly | 3.0.0 |

### Long-Term Support (LTS)

| Version | LTS Start | End of Life |
|---------|-----------|-------------|
| 2.0.x | 2025-03-15 | 2026-03-15 |
| 3.0.x | 2025-06-01 | 2026-06-01 |

## Version Bump Script

```bash
#!/bin/bash
# scripts/bump-version.sh

NEW_VERSION=$1

# Update all Cargo.toml files
find . -name "Cargo.toml" -exec sed -i "s/^version = .*/version = \"$NEW_VERSION\"/" {} \;

# Update CHANGELOG
sed -i "s/\[Unreleased\]/[$NEW_VERSION] - $(date +%Y-%m-%d)/" CHANGELOG.md

# Commit
git add .
git commit -m "chore(release): bump version to $NEW_VERSION"
git tag "v$NEW_VERSION"
```

## Version Checklist

Before releasing:

- [ ] Update CHANGELOG.md
- [ ] Update version in Cargo.toml files
- [ ] Update documentation
- [ ] Run full test suite
- [ ] Security audit (for major releases)
- [ ] Performance benchmarks
- [ ] Tag release
- [ ] Publish to crates.io
- [ ] Announce on social media

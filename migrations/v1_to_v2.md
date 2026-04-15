# Migration Guide: v0.9.x to v1.0

## Overview

This guide helps you migrate from BeeBotOS v0.9.x to v1.0.

## Breaking Changes

### 1. Configuration Format

**v1.x:**
```toml
[agent]
memory = "128M"
```

**v1.0:**
```toml
[agents]
default_memory_limit = 134217728
```

### 2. API Endpoints

**v1.x:**
```
POST /agents/create
```

**v1.0:**
```
POST /api/v1/agents
```

### 3. Agent ID Format

**v1.x:**
```
agent-12345678
```

**v1.0:**
```
agent_1234567890abcdef
```

## Migration Steps

### Step 1: Backup Data

```bash
cp -r ~/.beebotos/data ~/.beebotos/data-backup
```

### Step 2: Update Configuration

Convert old config to new format:

```bash
beebot migrate-config old-config.toml > new-config.toml
```

### Step 3: Stop v1.x Services

```bash
# Stop old services
systemctl stop beebotos-v1

# Or if using Docker
docker-compose -f docker-compose.v1.yml down
```

### Step 4: Install v1.0

```bash
# Download v1.0
curl -sSL https://get.beebotos.io | sh -s -- v1.0.0

# Or build from source
git checkout v1.0.0
cargo build --release
```

### Step 5: Migrate Data

```bash
# Run migration script
./scripts/migrate-v09-to-v1.sh
```

### Step 6: Start v1.0

```bash
# Start new services
systemctl start beebotos

# Or with Docker
docker-compose up -d
```

### Step 7: Verify

```bash
# Check version
beebot --version

# Test API
curl http://localhost:8080/health

# List agents
beebot list
```

## Rollback

If issues occur:

```bash
# Stop v1.0
systemctl stop beebotos

# Restore data
cp -r ~/.beebotos/data-backup/* ~/.beebotos/data/

# Start v1.x
systemctl start beebotos-v1
```

## Known Issues

1. **Agent state not preserved** - Must restart agents after migration
2. **Custom skills need recompilation** - Rebuild for WASM target
3. **Database schema changed** - Automatic migration on first run

## Support

- Discord: #migration-help
- GitHub Issues: tag with `migration`
- Email: support@beebotos.io

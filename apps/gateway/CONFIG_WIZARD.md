# BeeBotOS Gateway Configuration Wizard

The BeeBotOS Gateway includes an interactive configuration wizard that helps you set up the application on first launch or when reconfiguring.

## Quick Start

### First Time Setup

When you run the gateway for the first time without a configuration file:

```bash
cargo run --package beebotos-gateway
```

The wizard will automatically start and guide you through the configuration process.

### Interactive Configuration Mode

To explicitly start the configuration wizard:

```bash
# Using --wizard flag
cargo run --package beebotos-gateway -- --wizard

# Or using --configure flag
cargo run --package beebotos-gateway -- --configure

# Or using --reconfigure flag (forces wizard even if config exists)
cargo run --package beebotos-gateway -- --reconfigure
```

## Color Themes

The configuration wizard supports multiple color themes for accessibility and personal preference:

### Available Themes

| Theme | Description | Best For |
|-------|-------------|----------|
| `default` | Vibrant colors | Most terminals |
| `dark` | Bright colors on dark background | Dark terminal themes |
| `light` | Darker colors on light background | Light terminal themes |
| `high_contrast` | Maximum contrast | Accessibility needs |
| `minimal` | Mostly monochrome | Minimal distraction |
| `no_color` | Plain text only | CI/CD, logs, screen readers |

### Setting the Theme

#### Command Line Arguments (Highest Priority)

```bash
# Use a specific theme
cargo run --package beebotos-gateway -- --theme dark

# Disable colors completely
cargo run --package beebotos-gateway -- --no-color
```

#### Environment Variables

```bash
# Using BeeBotOS-specific variable
export BEE__WIZARD__COLOR_THEME=dark
cargo run --package beebotos-gateway

# Using standard NO_COLOR (disables all color output)
export NO_COLOR=1
cargo run --package beebotos-gateway

# Force color even in CI (if supported)
export FORCE_COLOR=1
cargo run --package beebotos-gateway
```

#### Configuration File

Add to your `config/beebotos.toml`:

```toml
[wizard]
color_theme = "dark"
show_icons = true
show_emoji = true
use_unicode = true
```

### Theme Priority Order

1. Command line arguments (`--theme`, `--no-color`)
2. Environment variables (`BEE__WIZARD__COLOR_THEME`, `NO_COLOR`)
3. Configuration file (`wizard.color_theme`)
4. Auto-detection (CI environment or non-TTY = no_color)
5. Default theme (`default`)

## Configuration Categories

The wizard covers the following configuration categories:

| Category | Description | Key Settings |
|----------|-------------|--------------|
| **Server** | HTTP server configuration | Host, port, timeout, CORS |
| **Database** | SQLite database settings | URL, connection pool, migrations |
| **JWT** | Authentication tokens | Secret, expiry, issuer |
| **Models** | AI/LLM providers | API keys, models, temperature |
| **Channels** | Messaging platforms | Lark, Discord, Telegram, etc. |
| **Blockchain** | Web3 integration | Chain ID, RPC, contracts |
| **Security** | Rate limiting & validation | Requests/sec, signatures |
| **Logging** | Logs and tracing | Level, format, rotation |
| **Metrics** | Prometheus monitoring | Endpoint, interval |
| **TLS** | HTTPS configuration | Certificates, mTLS |

## Wizard Workflow

### 1. Startup Detection

```
╔═══════════════════════════════════════════════════════════════╗
║           🐝 BeeBotOS Gateway Configuration Wizard            ║
╚═══════════════════════════════════════════════════════════════╝
```

### 2. Main Menu

```
📋 Configuration Menu:

  1.  Server Settings - HTTP server host, port, timeouts, CORS
  2.  Database Configuration - SQLite database path, connection pool settings
  3.  JWT Authentication - JWT secret, token expiration, issuer settings
  4.  AI/LLM Models - LLM providers (Kimi, OpenAI, etc.), API keys
  5.  Communication Channels - Messaging platforms (Lark, Discord, Telegram, etc.)
  6.  Blockchain/Web3 - Chain ID, RPC URL, wallet, contract addresses
  7.  Security Settings - Rate limiting, webhook validation, encryption
  8.  Logging & Tracing - Log level, format, rotation, tracing
  9.  Metrics & Monitoring - Prometheus metrics endpoint and interval
  10. TLS/SSL Configuration - HTTPS certificates and mutual TLS

  11. Configure All - Configure everything step by step
  12. Skip (use defaults) - Use minimal default settings

  0.  Exit without saving
```

### 3. Configuration Preview

Before saving, the wizard shows a preview of your configuration:

```
📋 Configuration Preview:
════════════════════════════════════════════════════════════

🌐 Server:
  Host: 0.0.0.0
  Port: 8080
  Timeout: 30s
  Max Body: 10MB

🗄️  Database:
  URL: sqlite://./data/beebotos.db
  Max Connections: 20

🔐 JWT:
  Secret: abcd...efgh (64 chars)
  Expiry: 24h

🤖 Models:
  Default Provider: kimi
  Configured Providers: kimi, openai

📱 Channels:
  ✅ lark
  ✅ discord

⛓️  Blockchain:
  Status: Disabled

🔒 Security:
  Rate Limiting: Enabled
  Webhook Verification: Enabled

════════════════════════════════════════════════════════════

Save this configuration? [Y/n]:
```

### 4. Configuration Backup

When reconfiguring, the wizard automatically backs up your existing configuration:

```
📦 Existing configuration backed up to: config/beebotos.toml.backup.20240115_143022
```

Backups are created with timestamps and stored alongside your main configuration file.

## Non-Interactive Mode

For CI/CD or automated deployments, use non-interactive mode:

```bash
# Set all required values via environment variables
export BEE__SERVER__HOST=0.0.0.0
export BEE__SERVER__PORT=8080
export BEE__DATABASE__URL=sqlite://./data/beebotos.db
export BEE__JWT__SECRET=your-secret-key-here-minimum-32-characters

# Run without wizard
cargo run --package beebotos-gateway
```

## Environment Variables Reference

All configuration options can be set via environment variables using the `BEE__` prefix:

```bash
# Server settings
BEE__SERVER__HOST=0.0.0.0
BEE__SERVER__PORT=8080

# Database
BEE__DATABASE__URL=sqlite://./data/beebotos.db

# JWT
BEE__JWT__SECRET=your-secret-key
BEE__JWT__EXPIRY_HOURS=24

# Wizard theme
BEE__WIZARD__COLOR_THEME=dark
BEE__WIZARD__SHOW_ICONS=true
BEE__WIZARD__SHOW_EMOJI=true
BEE__WIZARD__USE_UNICODE=true
```

## Troubleshooting

### Colors Not Displaying

1. Check if `NO_COLOR` environment variable is set
2. Verify terminal supports ANSI colors
3. Try forcing color with `FORCE_COLOR=1`
4. Use `--theme` flag to explicitly select a theme

### Wizard Not Starting

1. Check if `config/beebotos.toml` already exists
2. Use `--reconfigure` flag to force wizard
3. Check file permissions in `config/` directory

### Input Validation Errors

The wizard validates all inputs and provides clear error messages. Common issues:
- **Host**: Must be valid IP address or hostname
- **Port**: Must be between 1-65535
- **JWT Secret**: Minimum 32 characters recommended
- **Database URL**: Must start with `sqlite://`, `postgres://`, or `mysql://`

# Configuration Guide

FabreXLens offers flexible configuration through multiple methods, allowing you to customize behavior for different environments and use cases.

## 📋 Configuration Methods

Configuration is loaded in this priority order (highest to lowest):

1. **Command-line flags** (highest priority)
2. **Environment variables**
3. **Profile-specific config files**
4. **Default config file**
5. **Built-in defaults** (lowest priority)

## 📁 Configuration Files

### File Locations

FabreXLens looks for configuration files in these locations:

**Linux:**
- `~/.config/fabrexlens/fabrexlens.toml`
- `~/.config/fabrexlens/fabrexlens.<profile>.toml`

**macOS:**
- `~/Library/Application Support/FabreXLens/fabrexlens.toml`
- `~/Library/Application Support/FabreXLens/fabrexlens.<profile>.toml`

**Windows:**
- `%APPDATA%\FabreXLens\fabrexlens.toml`
- `%APPDATA%\FabreXLens\fabrexlens.<profile>.toml`

### File Format

Configuration files use TOML format:

```toml
# Example configuration file
[fabrex]
base_url = "https://fabrex.example.com"
timeout = 30
retries = 3

[ui]
theme = "dark"
auto_refresh = true
refresh_interval = 60

[logging]
level = "info"
file = "~/fabrexlens.log"
```

## ⚙️ Configuration Schema

### Service Configuration

#### FabreX Settings

```toml
[fabrex]
# Base URL for FabreX API
base_url = "https://fabrex.example.com"

# Request timeout in seconds
timeout = 30

# Number of retry attempts for failed requests
retries = 3

# API version to use
api_version = "v1"

# Custom headers (optional)
# headers = { Authorization = "Bearer token", "X-Custom" = "value" }
```

#### Gryf Settings

```toml
[gryf]
# Base URL for Gryf API
base_url = "https://gryf.example.com"

# Request timeout in seconds
timeout = 30

# Number of retry attempts
retries = 3

# API version
api_version = "v1"
```

#### Supernode Settings

```toml
[supernode]
# Base URL for Supernode API
base_url = "https://supernode.example.com"

# Request timeout in seconds
timeout = 30

# Number of retry attempts
retries = 3

# API version
api_version = "v1"
```

#### Redfish Settings

```toml
[redfish]
# Base URL for Redfish API
base_url = "https://bmc.example.com"

# Request timeout in seconds
timeout = 30

# Number of retry attempts
retries = 3

# Redfish protocol version
protocol_version = "1.0"
```

### UI Configuration

```toml
[ui]
# UI theme: "system", "light", "dark"
theme = "system"

# Enable auto-refresh on startup
auto_refresh = true

# Auto-refresh interval in seconds (5-300)
refresh_interval = 60

# Window size on startup
window_width = 1200
window_height = 800

# Remember window position
remember_position = true

# Enable fullscreen mode
fullscreen = false

# Font size scaling (0.5-2.0)
font_scale = 1.0
```

### Logging Configuration

```toml
[logging]
# Log level: "error", "warn", "info", "debug", "trace"
level = "info"

# Log to file (optional)
file = "~/fabrexlens.log"

# Maximum log file size in MB
max_file_size = 10

# Maximum number of log files to keep
max_files = 5

# Include timestamps in logs
timestamps = true

# Include thread IDs in logs
thread_ids = false

# Structured logging (JSON format)
structured = false
```

### Security Configuration

```toml
[security]
# Keychain service name
keychain_service = "FabreXLens"

# Token refresh threshold in minutes
token_refresh_threshold = 30

# Certificate validation mode: "strict", "warn", "disabled"
certificate_validation = "strict"

# Custom CA certificates file
ca_certificates = "/path/to/ca-bundle.crt"

# Client certificates (optional)
# client_cert = "/path/to/client.crt"
# client_key = "/path/to/client.key"
```

### Network Configuration

```toml
[network]
# HTTP proxy URL (optional)
proxy = "http://proxy.example.com:8080"

# Proxy bypass list (comma-separated)
proxy_bypass = "localhost,127.0.0.1"

# DNS timeout in seconds
dns_timeout = 5

# Connection pool size
pool_max_idle_per_host = 10

# TCP keepalive interval
tcp_keepalive = 60

# User agent string
user_agent = "FabreXLens/0.1.0"
```

## 🌍 Environment Variables

All configuration options can be overridden with environment variables using the prefix `FABREXLENS__`:

### Examples

```bash
# Service URLs
export FABREXLENS__FABREX__BASE_URL="https://prod-fabrex.example.com"
export FABREXLENS__GRYF__BASE_URL="https://prod-gryf.example.com"

# UI settings
export FABREXLENS__UI__THEME="dark"
export FABREXLENS__UI__AUTO_REFRESH="true"
export FABREXLENS__UI__REFRESH_INTERVAL="30"

# Logging
export FABREXLENS__LOGGING__LEVEL="debug"
export FABREXLENS__LOGGING__FILE="~/fabrexlens-debug.log"

# Security
export FABREXLENS__SECURITY__CERTIFICATE_VALIDATION="warn"
```

### Environment Variable Naming

Environment variables follow this pattern:
```
FABREXLENS__<SECTION>__<KEY>
```

Where:
- `<SECTION>` is the config section in UPPERCASE
- `<KEY>` is the setting name in UPPERCASE
- Nested sections use double underscores

Examples:
- `ui.theme` → `FABREXLENS__UI__THEME`
- `fabrex.base_url` → `FABREXLENS__FABREX__BASE_URL`
- `logging.max_file_size` → `FABREXLENS__LOGGING__MAX_FILE_SIZE`

## 🏷️ Profile Management

Profiles allow you to maintain different configurations for different environments:

### Creating Profiles

```bash
# Create a development profile
cat > ~/.config/fabrexlens/fabrexlens.dev.toml << EOF
[fabrex]
base_url = "https://dev-fabrex.example.com"
timeout = 60

[gryf]
base_url = "https://dev-gryf.example.com"

[ui]
theme = "light"
auto_refresh = false

[logging]
level = "debug"
EOF

# Create a production profile
cat > ~/.config/fabrexlens/fabrexlens.prod.toml << EOF
[fabrex]
base_url = "https://prod-fabrex.example.com"
timeout = 30

[gryf]
base_url = "https://prod-gryf.example.com"

[ui]
theme = "dark"
auto_refresh = true
refresh_interval = 30

[logging]
level = "warn"
EOF
```

### Using Profiles

```bash
# Use development profile
fabrexlens --profile dev

# Use production profile
fabrexlens --profile prod

# Use default profile
fabrexlens
```

### Profile Precedence

1. `--profile` flag (highest)
2. Default profile file
3. Built-in defaults (lowest)

## 🎛️ Command Line Options

### Global Options

```bash
fabrexlens [OPTIONS] [COMMAND]
```

| Option | Description | Example |
|--------|-------------|---------|
| `--config <FILE>` | Use specific config file | `--config ./custom.toml` |
| `--profile <NAME>` | Use named profile | `--profile production` |
| `--headless` | Run without GUI | `--headless` |
| `--verbose` | Enable verbose logging | `--verbose` |
| `--quiet` | Suppress non-error output | `--quiet` |
| `--version` | Show version information | `--version` |
| `--help` | Show help information | `--help` |

### Authentication Commands

```bash
fabrexlens auth-init [OPTIONS]
```

| Option | Description | Example |
|--------|-------------|---------|
| `--domain <DOMAIN>` | Service domain (fabrex, gryf, supernode, redfish) | `--domain fabrex` |
| `--scope <SCOPE>` | Credential scope name | `--scope production` |
| `--username <USER>` | Username (prompts if not provided) | `--username admin` |
| `--dry-run` | Validate without storing | `--dry-run` |

### Configuration Commands

```bash
fabrexlens config [COMMAND]
```

Commands:
- `show`: Display current configuration
- `validate`: Validate configuration file
- `migrate`: Migrate legacy configuration

## 🔧 Advanced Configuration

### Custom Certificate Authorities

For environments with custom CA certificates:

```toml
[security]
certificate_validation = "strict"
ca_certificates = "/etc/ssl/certs/ca-certificates.crt"
```

### Client Certificate Authentication

For mutual TLS authentication:

```toml
[security]
client_cert = "/path/to/client.crt"
client_key = "/path/to/client.key"
```

### Proxy Configuration

For corporate environments:

```toml
[network]
proxy = "http://proxy.company.com:8080"
proxy_bypass = "localhost,127.0.0.1,.local"
```

### High Availability Setup

For production deployments:

```toml
[fabrex]
base_url = "https://fabrex-lb.company.com"
timeout = 10
retries = 5

[gryf]
base_url = "https://gryf-lb.company.com"
timeout = 10
retries = 5

[ui]
auto_refresh = true
refresh_interval = 15

[logging]
level = "info"
structured = true
```

## 🔍 Configuration Validation

FabreXLens can validate your configuration:

```bash
# Validate current configuration
fabrexlens config validate

# Show current effective configuration
fabrexlens config show

# Validate specific file
fabrexlens config validate --file custom.toml
```

## 🚨 Configuration Troubleshooting

### Common Issues

**Configuration not loading:**
- Check file permissions: `ls -la ~/.config/fabrexlens/`
- Verify TOML syntax: Use online TOML validator
- Check file paths: Use absolute paths

**Environment variables ignored:**
- Use correct prefix: `FABREXLENS__SECTION__KEY`
- Check variable names: Use UPPERCASE for sections/keys
- Verify export: `echo $FABREXLENS__UI__THEME`

**Profile not working:**
- Check profile file exists: `ls ~/.config/fabrexlens/fabrexlens.<profile>.toml`
- Verify profile name: Use `--profile <name>` flag
- Check file permissions

### Debugging Configuration

Enable debug logging to troubleshoot configuration issues:

```bash
# Enable debug logging
export FABREXLENS__LOGGING__LEVEL=debug
fabrexlens --verbose

# Log to file
export FABREXLENS__LOGGING__FILE=~/fabrexlens-config-debug.log
fabrexlens
```

### Resetting Configuration

To reset to defaults:

```bash
# Remove config files
rm -rf ~/.config/fabrexlens/

# Clear environment variables
unset $(env | grep FABREXLENS | cut -d= -f1)
```

## 📋 Configuration Examples

### Development Environment

```toml
[fabrex]
base_url = "http://localhost:8080"
timeout = 300

[gryf]
base_url = "http://localhost:8081"
timeout = 300

[ui]
theme = "light"
auto_refresh = false

[logging]
level = "debug"
file = "~/fabrexlens-dev.log"
```

### Production Environment

```toml
[fabrex]
base_url = "https://fabrex-prod.company.com"
timeout = 30
retries = 3

[gryf]
base_url = "https://gryf-prod.company.com"
timeout = 30
retries = 3

[supernode]
base_url = "https://supernode-prod.company.com"
timeout = 30
retries = 3

[ui]
theme = "dark"
auto_refresh = true
refresh_interval = 30

[logging]
level = "warn"
structured = true
```

### CI/CD Environment

```toml
[fabrex]
base_url = "https://fabrex-ci.company.com"

[ui]
auto_refresh = false

[logging]
level = "info"
structured = true
```

---

For more help with configuration, see the [User Guide](user-guide.md) or [open an issue](https://github.com/Digital-Data-Co/FabreXLens/issues).

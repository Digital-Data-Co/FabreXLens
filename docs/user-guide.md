# User Guide

This guide will help you get the most out of FabreXLens, from initial setup to advanced usage patterns.

## 🎯 Overview

FabreXLens provides both a graphical desktop interface and command-line tools for managing GigaIO FabreX fabrics, Gryf workloads, and Supernode infrastructure. The application offers real-time monitoring, credential management, and automated workflows.

## 🚀 First Time Setup

### Prerequisites

Before using FabreXLens, ensure you have:
- Access to GigaIO FabreX, Gryf, and/or Supernode endpoints
- API credentials (username/password or tokens)
- Network connectivity to your infrastructure

### Initial Configuration

1. **Install FabreXLens** following the [Installation Guide](installation.md)

2. **Set up credentials** for each service you want to monitor:
   ```bash
   # FabreX fabrics
   fabrexlens auth-init --domain fabrex

   # Gryf workloads
   fabrexlens auth-init --domain gryf

   # Supernode infrastructure
   fabrexlens auth-init --domain supernode

   # Redfish management
   fabrexlens auth-init --domain redfish
   ```

   Each command will:
   - Prompt for your username and password
   - Store credentials securely in your system keychain
   - Validate the connection

3. **Launch the application:**
   ```bash
   fabrexlens
   ```

## 🖥️ Graphical User Interface

### Main Dashboard

When you launch FabreXLens, you'll see the main dashboard with several key areas:

#### Status Bar
- **Connection Status**: Shows connectivity to each service
- **Last Refresh**: Timestamp of the most recent data update
- **Auto-Refresh**: Toggle and interval controls

#### Navigation Tabs
- **Dashboard**: Main overview and metrics
- **Fabrics**: Detailed fabric information and management
- **Workloads**: Gryf workload monitoring
- **Supernodes**: Infrastructure node details
- **Logs**: Event history and troubleshooting

### Dashboard Overview

The main dashboard displays:

- **Fabric Summary**: Active fabrics with health status
- **Workload Statistics**: Running workloads by type
- **Node Health**: Supernode availability and performance
- **Recent Events**: Latest system activities

#### Real-time Metrics

The dashboard shows live data including:
- Active connections and throughput
- Resource utilization (CPU, memory, storage)
- Error rates and latency metrics
- Fabric topology changes

### Managing Fabrics

#### Viewing Fabric Details

1. Navigate to the **Fabrics** tab
2. Select a fabric from the list
3. View detailed information including:
   - Physical topology
   - Connected endpoints
   - Performance metrics
   - Configuration status

#### Fabric Reassignments

To reassign endpoints between fabrics:

1. Go to **Fabrics** → **Reassignment**
2. Select source fabric and endpoint
3. Choose destination fabric
4. Review the reassignment plan
5. Click **Execute** to proceed

⚠️ **Warning**: Reassignments may cause temporary service disruption. Ensure you understand the impact before proceeding.

### Workload Monitoring

The **Workloads** tab provides:

- **Active Workloads**: Currently running Gryf workloads
- **Resource Usage**: CPU, memory, and storage consumption
- **Performance Metrics**: Throughput and latency
- **Error Tracking**: Failed workloads and error patterns

### Supernode Management

Monitor your infrastructure nodes:

- **Node Status**: Online/offline status and health
- **Resource Pools**: Available capacity and allocation
- **Network Connectivity**: Link status and bandwidth
- **Hardware Health**: Temperature, power, and component status

## ⚙️ Auto-Refresh Configuration

FabreXLens can automatically refresh data at configurable intervals:

### Enabling Auto-Refresh

1. In the status bar, click the **Auto-Refresh** toggle
2. Adjust the **Interval** slider (5-300 seconds)
3. The dashboard will update automatically

### Auto-Refresh Behavior

- **Background Operation**: Refreshes don't block the UI
- **Error Handling**: Continues trying even if some services fail
- **Battery Friendly**: Reduces frequency when credentials expire
- **Manual Override**: You can still trigger manual refreshes anytime

## 🔧 Command Line Interface

FabreXLens also provides powerful CLI tools for automation and scripting.

### CLI Reference

```bash
fabrexlens [OPTIONS] [COMMAND]
```

#### Global Options

- `--config <FILE>`: Use specific config file
- `--profile <NAME>`: Use named profile
- `--headless`: Run without GUI
- `--verbose`: Enable verbose logging
- `--help`: Show help information

#### Commands

##### Authentication

```bash
# Initialize credentials
fabrexlens auth-init --domain <service> [--scope <name>]

# List stored credentials
fabrexlens auth-list

# Remove credentials
fabrexlens auth-remove --domain <service> [--scope <name>]
```

##### Data Operations

```bash
# Get fabric information
fabrexlens fabrics list
fabrexlens fabrics show <fabric-id>

# Get workload information
fabrexlens workloads list
fabrexlens workloads show <workload-id>

# Get node information
fabrexlens nodes list
fabrexlens nodes show <node-id>
```

##### Maintenance

```bash
# Health check
fabrexlens health

# Version information
fabrexlens --version

# Configuration dump
fabrexlens config show
```

### CLI Examples

#### Monitoring Script

```bash
#!/bin/bash
# Monitor fabric health every 5 minutes

while true; do
    echo "=== $(date) ==="
    fabrexlens --headless fabrics list
    echo ""
    sleep 300
done
```

#### Automated Reassignment

```bash
# Reassign endpoint using CLI
fabrexlens fabrics reassign \
    --from-fabric fabric-001 \
    --to-fabric fabric-002 \
    --endpoint endpoint-123
```

## 📊 Event Logging and Monitoring

### Event Log

FabreXLens maintains a comprehensive event log:

- **Refresh Events**: Data update successes/failures
- **Authentication Events**: Credential validation and renewal
- **Reassignment Events**: Workflow completions and errors
- **System Events**: Application status and warnings

### Log Levels

- **INFO**: Normal operations and status updates
- **WARN**: Non-critical issues that may need attention
- **ERROR**: Failures that require intervention
- **DEBUG**: Detailed diagnostic information

### Exporting Logs

```bash
# Export recent logs to file
fabrexlens logs export --output logs.txt --since "1 hour ago"

# Stream logs in real-time
fabrexlens logs tail --follow
```

## 🔒 Security Best Practices

### Credential Management

- **Regular Rotation**: Update credentials periodically
- **Scope Limitation**: Use scoped credentials when possible
- **Keychain Security**: Rely on system keychain for storage
- **Access Auditing**: Monitor authentication events

### Network Security

- **HTTPS Only**: Ensure all endpoints use HTTPS
- **Certificate Validation**: Verify server certificates
- **Firewall Rules**: Limit network access as needed
- **VPN Usage**: Consider VPN for sensitive operations

## 🚀 Advanced Usage

### Custom Configurations

Create profile-specific configurations:

```bash
# Create development profile
mkdir -p ~/.config/FabreXLens
cat > ~/.config/FabreXLens/fabrexlens.dev.toml << EOF
[fabrex]
base_url = "https://dev-fabrex.example.com"
timeout = 30

[ui]
theme = "dark"
auto_refresh = false
EOF

# Use development profile
fabrexlens --profile dev
```

### Environment Variables

Override configuration with environment variables:

```bash
export FABREXLENS_FABREX_BASE_URL="https://prod-fabrex.example.com"
export FABREXLENS_UI_AUTO_REFRESH_INTERVAL=60
export FABREXLENS_LOG_LEVEL=debug

fabrexlens
```

### Scripting Integration

FabreXLens can be integrated into monitoring and automation scripts:

```python
#!/usr/bin/env python3
import subprocess
import json
import sys

def get_fabric_status():
    try:
        result = subprocess.run(
            ['fabrexlens', '--headless', 'fabrics', 'list', '--json'],
            capture_output=True,
            text=True,
            timeout=30
        )
        return json.loads(result.stdout)
    except subprocess.TimeoutExpired:
        print("Timeout getting fabric status", file=sys.stderr)
        return None
    except json.JSONDecodeError:
        print("Failed to parse JSON response", file=sys.stderr)
        return None

if __name__ == "__main__":
    status = get_fabric_status()
    if status:
        print(f"Found {len(status.get('fabrics', []))} fabrics")
    else:
        sys.exit(1)
```

## 🎛️ Customization

### UI Themes

FabreXLens supports multiple themes:

- **System**: Follows your OS theme
- **Light**: Traditional light theme
- **Dark**: Easy on the eyes theme

### Keyboard Shortcuts

- `Ctrl+R` / `Cmd+R`: Manual refresh
- `Ctrl+L` / `Cmd+L`: Clear logs
- `Ctrl+,` / `Cmd+,`: Open settings
- `F11`: Toggle fullscreen
- `Esc`: Close dialogs

### Performance Tuning

For large deployments, consider:

- **Increase refresh intervals** to reduce API load
- **Use CLI tools** for bulk operations
- **Enable caching** in configuration
- **Monitor memory usage** and adjust as needed

## 🔍 Troubleshooting

### Connection Issues

**Symptoms**: Services show as disconnected or data doesn't load

**Solutions**:
1. Verify network connectivity to endpoints
2. Check credentials: `fabrexlens auth-list`
3. Reinitialize credentials: `fabrexlens auth-init --domain <service>`
4. Check firewall settings and DNS resolution

### Performance Issues

**Symptoms**: UI is slow or unresponsive

**Solutions**:
1. Reduce auto-refresh interval
2. Close unused tabs
3. Restart the application
4. Check system resources (CPU, memory)

### Authentication Problems

**Symptoms**: "Authentication failed" errors

**Solutions**:
1. Verify credentials are correct
2. Check if credentials expired
3. Try re-initializing: `fabrexlens auth-init --domain <service>`
4. Check keychain permissions

For more detailed troubleshooting, see the [Troubleshooting Guide](troubleshooting.md).

## 📚 Next Steps

- **[Configuration Guide](configuration.md)**: Advanced configuration options
- **[Troubleshooting Guide](troubleshooting.md)**: Solutions to common problems
- **[Contributing Guide](contributing.md)**: How to contribute to FabreXLens

---

💡 **Tip**: Join our [GitHub Discussions](https://github.com/Digital-Data-Co/FabreXLens/discussions) to share tips and ask questions!

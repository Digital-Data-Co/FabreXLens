# Troubleshooting Guide

This guide helps you resolve common issues with FabreXLens. If you can't find a solution here, check our [GitHub Issues](https://github.com/Digital-Data-Co/FabreXLens/issues) or create a new issue.

## 🚨 Quick Diagnosis

### Health Check Command

Run this first to diagnose issues:

```bash
# Check overall application health
fabrexlens --headless health

# Check with verbose output
fabrexlens --verbose --headless health
```

### Log Analysis

Check the application logs:

```bash
# View recent logs
fabrexlens logs tail --lines 50

# Export logs for analysis
fabrexlens logs export --output fabrexlens-debug.log --since "1 hour ago"
```

## 🔌 Connection Issues

### Services Show as Disconnected

**Symptoms:**
- Dashboard shows "Disconnected" status
- Data doesn't load or refresh
- Authentication errors in logs

**Possible Causes & Solutions:**

1. **Network Connectivity:**
   ```bash
   # Test basic connectivity
   curl -I https://your-fabrex-endpoint.com/health

   # Check DNS resolution
   nslookup your-fabrex-endpoint.com

   # Test with different network (VPN, etc.)
   ```

2. **Firewall/Security Groups:**
   - Ensure outbound HTTPS (443) is allowed
   - Check corporate proxy settings
   - Verify IP allowlists if applicable

3. **SSL/TLS Issues:**
   ```bash
   # Test SSL certificate
   openssl s_client -connect your-endpoint.com:443 -servername your-endpoint.com

   # If using custom CAs, check configuration
   fabrexlens config show | grep certificate
   ```

### Authentication Failures

**Symptoms:**
- "Invalid credentials" errors
- 401/403 HTTP responses
- Keychain access denied

**Solutions:**

1. **Reinitialize Credentials:**
   ```bash
   # Remove old credentials
   fabrexlens auth-remove --domain fabrex

   # Reinitialize
   fabrexlens auth-init --domain fabrex
   ```

2. **Check Credential Storage:**
   ```bash
   # List stored credentials
   fabrexlens auth-list

   # Test keychain access (macOS)
   security find-generic-password -s "FabreXLens" -a "fabrex"

   # Test keychain access (Linux with GNOME)
   secret-tool lookup service FabreXLens domain fabrex
   ```

3. **Token Expiration:**
   - API tokens may have expired
   - Reinitialize with fresh credentials
   - Check token validity period with your API provider

### Timeout Errors

**Symptoms:**
- Requests hang or timeout
- Slow dashboard loading
- Network timeout errors in logs

**Solutions:**

1. **Increase Timeout Values:**
   ```toml
   # In configuration file
   [fabrex]
   timeout = 60  # Increase from default 30

   [gryf]
   timeout = 60
   ```

2. **Network Issues:**
   ```bash
   # Test network latency
   ping your-endpoint.com

   # Check MTU issues
   tracepath your-endpoint.com
   ```

3. **Server-Side Issues:**
   - Contact your infrastructure admin
   - Check server logs for performance issues

## 🖥️ GUI Issues

### Application Won't Start

**Symptoms:**
- GUI doesn't launch
- Crashes immediately on startup
- "Segmentation fault" or similar errors

**Solutions:**

1. **Check Dependencies:**
   ```bash
   # Linux: Check required libraries
   ldd target/release/fabrexlens | grep "not found"

   # macOS: Check for required frameworks
   otool -L fabrexlens
   ```

2. **Graphics Drivers:**
   ```bash
   # Check OpenGL/Vulkan support
   glxinfo | grep "OpenGL version"

   # Test with software rendering
   export LIBGL_ALWAYS_SOFTWARE=1
   fabrexlens
   ```

3. **Run in Headless Mode:**
   ```bash
   # Test CLI functionality
   fabrexlens --headless health
   ```

### UI Performance Issues

**Symptoms:**
- Slow response times
- High CPU/memory usage
- UI freezing or stuttering

**Solutions:**

1. **Reduce Refresh Frequency:**
   ```bash
   # Disable auto-refresh temporarily
   fabrexlens --config <(echo '[ui]'; echo 'auto_refresh = false')
   ```

2. **Adjust UI Settings:**
   ```toml
   [ui]
   refresh_interval = 120  # Increase interval
   font_scale = 0.8       # Reduce font scaling
   ```

3. **Resource Monitoring:**
   ```bash
   # Monitor system resources
   top -p $(pgrep fabrexlens)

   # Check memory usage
   ps aux | grep fabrexlens
   ```

### Window/Display Issues

**Symptoms:**
- Blank or corrupted display
- Incorrect window sizing
- HiDPI scaling problems

**Solutions:**

1. **HiDPI Displays:**
   ```bash
   # Adjust scaling factor
   export QT_SCALE_FACTOR=1.5
   fabrexlens

   # Or in config
   [ui]
   font_scale = 1.5
   ```

2. **Wayland/X11 Issues (Linux):**
   ```bash
   # Force X11 backend
   export QT_QPA_PLATFORM=xcb
   fabrexlens

   # Or Wayland
   export QT_QPA_PLATFORM=wayland
   fabrexlens
   ```

3. **Reset Window Settings:**
   ```bash
   # Remove window state cache
   rm -rf ~/.cache/fabrexlens/
   ```

## 🔐 Security & Permissions

### Keychain Access Denied

**Symptoms:**
- "Permission denied" when storing credentials
- Authentication prompts keep appearing

**Platform-Specific Solutions:**

**macOS:**
```bash
# Check keychain permissions
security find-generic-password -s "FabreXLens"

# Reset keychain access
# Go to Keychain Access.app → Preferences → Reset My Default Keychain
```

**Linux (GNOME):**
```bash
# Check secret service
busctl --user list | grep org.freedesktop.secrets

# Restart secret service
systemctl --user restart gnome-keyring-daemon
```

**Linux (KDE):**
```bash
# Check kwallet
kwallet-query -l

# Unlock wallet
kwallet-query -u
```

**Windows:**
```powershell
# Check credential manager permissions
# Control Panel → Credential Manager → Windows Credentials
```

### Certificate Validation Errors

**Symptoms:**
- SSL/TLS handshake failures
- "Certificate verification failed" errors

**Solutions:**

1. **Add Custom CA Certificates:**
   ```toml
   [security]
   certificate_validation = "strict"
   ca_certificates = "/path/to/ca-bundle.crt"
   ```

2. **Disable Certificate Validation (Development Only):**
   ```toml
   [security]
   certificate_validation = "disabled"
   ```

3. **Update System Certificates:**
   ```bash
   # Ubuntu/Debian
   sudo update-ca-certificates

   # CentOS/RHEL
   sudo update-ca-trust

   # macOS
   sudo update-ca-certificates
   ```

## 📊 Data & API Issues

### Incorrect or Missing Data

**Symptoms:**
- Empty dashboards
- Missing fabric/workload information
- Stale data not updating

**Solutions:**

1. **Manual Refresh:**
   ```bash
   # Force refresh from CLI
   fabrexlens --headless fabrics list
   fabrexlens --headless workloads list
   ```

2. **Check API Permissions:**
   - Verify user has read access to all endpoints
   - Check API key scopes/permissions
   - Review audit logs on the server side

3. **API Endpoint Issues:**
   ```bash
   # Test API endpoints directly
   curl -H "Authorization: Bearer YOUR_TOKEN" https://api-endpoint.com/v1/fabrics

   # Check API documentation
   # Verify endpoint URLs in configuration
   ```

### High API Error Rates

**Symptoms:**
- Frequent API failures
- Rate limiting errors
- 429 Too Many Requests

**Solutions:**

1. **Increase Retry Configuration:**
   ```toml
   [fabrex]
   retries = 5
   timeout = 60

   [gryf]
   retries = 5
   timeout = 60
   ```

2. **Reduce Request Frequency:**
   ```toml
   [ui]
   refresh_interval = 120  # Increase from 60
   ```

3. **Check Rate Limits:**
   - Review API provider's rate limiting policies
   - Implement exponential backoff
   - Consider API quota upgrades

## 🔧 Build & Installation Issues

### Build Failures

**Symptoms:**
- Cargo build fails
- Missing dependencies
- Compiler errors

**Solutions:**

1. **Update Rust:**
   ```bash
   rustup update
   rustc --version  # Should be 1.75+
   ```

2. **Install System Dependencies:**
   ```bash
   # Ubuntu/Debian
   sudo apt-get install build-essential pkg-config libssl-dev

   # CentOS/RHEL
   sudo yum groupinstall "Development Tools"
   sudo yum install openssl-devel

   # macOS
   xcode-select --install
   ```

3. **Clear Build Cache:**
   ```bash
   cargo clean
   rm -rf ~/.cargo/registry/cache/
   ```

### Installation Problems

**Symptoms:**
- "Command not found" errors
- Permission denied during installation
- PATH issues

**Solutions:**

1. **Check Installation:**
   ```bash
   # Verify binary exists and is executable
   ls -la /usr/local/bin/fabrexlens
   file /usr/local/bin/fabrexlens

   # Check permissions
   chmod +x /usr/local/bin/fabrexlens
   ```

2. **Update PATH:**
   ```bash
   # Add to PATH if needed
   export PATH="$HOME/bin:$PATH"

   # Make permanent
   echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
   ```

3. **macOS Gatekeeper:**
   ```bash
   # Remove quarantine attribute
   xattr -d com.apple.quarantine fabrexlens

   # Or allow in System Settings
   # System Settings → Privacy & Security → Allow Anyway
   ```

## 📋 Frequently Asked Questions (FAQ)

### General Questions

**Q: Does FabreXLens store my credentials?**
A: Yes, credentials are securely stored in your system's keychain/credential manager. They are encrypted and only accessible by FabreXLens.

**Q: Can I use FabreXLens in a CI/CD pipeline?**
A: Yes, use `--headless` mode with appropriate environment variables for configuration.

**Q: How do I backup my configuration?**
A: Copy your configuration files from `~/.config/fabrexlens/` (Linux/macOS) or `%APPDATA%\FabreXLens\` (Windows).

### Performance Questions

**Q: Why is FabreXLens using so much CPU?**
A: This can happen with frequent auto-refresh. Increase the refresh interval or disable auto-refresh.

**Q: Can I run multiple instances?**
A: Yes, but use different profiles to avoid credential conflicts.

**Q: How do I reduce memory usage?**
A: Disable auto-refresh, close unused tabs, and restart periodically.

### Compatibility Questions

**Q: Which operating systems are supported?**
A: Linux (x86_64), macOS (x86_64/ARM64), and Windows (x86_64).

**Q: Do I need administrative privileges?**
A: No, FabreXLens runs as a regular user. Administrative access may be needed only for system-wide installation.

**Q: Can I use FabreXLens offline?**
A: No, FabreXLens requires network connectivity to your GigaIO infrastructure.

## 🆘 Getting Help

### Support Resources

1. **Documentation:**
   - [Installation Guide](installation.md)
   - [User Guide](user-guide.md)
   - [Configuration Guide](configuration.md)

2. **Community Support:**
   - [GitHub Discussions](https://github.com/Digital-Data-Co/FabreXLens/discussions)
   - [GitHub Issues](https://github.com/Digital-Data-Co/FabreXLens/issues)

3. **Debug Information:**
   ```bash
   # Collect system information
   fabrexlens --verbose --headless config show > debug-config.txt
   fabrexlens logs export --output debug-logs.txt --since "24 hours ago"

   # Include in bug reports:
   # - OS version and architecture
   # - FabreXLens version (--version)
   # - Configuration (sanitized)
   # - Error logs
   # - Steps to reproduce
   ```

### Escalation Process

1. **Check existing issues** on GitHub
2. **Create a new issue** with detailed information
3. **Include debug logs** and configuration
4. **Specify your environment** (OS, versions, etc.)

For security-related issues, please mark them as confidential when creating the GitHub issue.

---

💡 **Pro Tip**: Enable debug logging (`FABREXLENS__LOGGING__LEVEL=debug`) for more detailed troubleshooting information.

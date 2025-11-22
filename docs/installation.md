# Installation Guide

This guide covers installing FabreXLens on different platforms. You can either download pre-built binaries or build from source.

## 📦 Pre-built Binaries (Recommended)

Pre-built binaries are available for all major platforms from our [GitHub Releases](https://github.com/Digital-Data-Co/FabreXLens/releases) page.

### System Requirements

- **Memory**: 256MB RAM minimum, 512MB recommended
- **Storage**: 50MB free space
- **Operating System**:
  - Linux: Ubuntu 18.04+, CentOS 7+, or equivalent
  - macOS: 10.15+ (Intel) or 11.0+ (Apple Silicon)
  - Windows: Windows 10 version 1903+ or Windows 11

### Linux (AMD64)

1. **Download the binary:**
   ```bash
   # Download the latest release
   curl -LO https://github.com/Digital-Data-Co/FabreXLens/releases/latest/download/fabrexlens-linux-amd64.tar.gz
   ```

2. **Verify the download:**
   ```bash
   # Download checksum
   curl -LO https://github.com/Digital-Data-Co/FabreXLens/releases/latest/download/fabrexlens-linux-amd64.tar.gz.sha256

   # Verify
   sha256sum -c fabrexlens-linux-amd64.tar.gz.sha256
   ```

3. **Extract and install:**
   ```bash
   # Extract
   tar -xzf fabrexlens-linux-amd64.tar.gz

   # Move to a directory in your PATH (optional)
   sudo mv fabrexlens /usr/local/bin/
   # OR add to your local bin directory
   mkdir -p ~/bin && mv fabrexlens ~/bin/
   ```

4. **Verify installation:**
   ```bash
   fabrexlens --version
   ```

### macOS (Intel/Apple Silicon)

1. **Download the appropriate binary:**
   ```bash
   # For Intel Macs
   curl -LO https://github.com/Digital-Data-Co/FabreXLens/releases/latest/download/fabrexlens-macos-amd64.tar.gz

   # For Apple Silicon Macs
   curl -LO https://github.com/Digital-Data-Co/FabreXLens/releases/latest/download/fabrexlens-macos-arm64.tar.gz
   ```

2. **Verify the download:**
   ```bash
   # Download checksum (use the same name as your download)
   curl -LO https://github.com/Digital-Data-Co/FabreXLens/releases/latest/download/fabrexlens-macos-amd64.tar.gz.sha256
   # OR for Apple Silicon:
   curl -LO https://github.com/Digital-Data-Co/FabreXLens/releases/latest/download/fabrexlens-macos-arm64.tar.gz.sha256

   # Verify
   shasum -a 256 -c fabrexlens-macos-*.tar.gz.sha256
   ```

3. **Extract and install:**
   ```bash
   # Extract
   tar -xzf fabrexlens-macos-*.tar.gz

   # Move to Applications or your PATH
   mv fabrexlens /usr/local/bin/
   # OR
   mv fabrexlens ~/bin/
   ```

4. **For first run (macOS security):**
   ```bash
   # Right-click the binary and select "Open" on first run
   # OR from command line:
   xattr -d com.apple.quarantine fabrexlens
   ```

### Windows (AMD64)

1. **Download the binary:**
   ```powershell
   # PowerShell
   Invoke-WebRequest -Uri "https://github.com/Digital-Data-Co/FabreXLens/releases/latest/download/fabrexlens-windows-amd64.zip" -OutFile "fabrexlens-windows-amd64.zip"
   ```

2. **Verify the download:**
   ```powershell
   # Download checksum
   Invoke-WebRequest -Uri "https://github.com/Digital-Data-Co/FabreXLens/releases/latest/download/fabrexlens-windows-amd64.zip.sha256" -OutFile "fabrexlens-windows-amd64.zip.sha256"

   # Verify (requires PowerShell 4.0+)
   $expected = Get-Content "fabrexlens-windows-amd64.zip.sha256" | ForEach-Object { $_.Split()[0] }
   $actual = Get-FileHash "fabrexlens-windows-amd64.zip" -Algorithm SHA256
   if ($expected -eq $actual.Hash) { Write-Host "Checksum verified" } else { Write-Host "Checksum mismatch!" }
   ```

3. **Extract and install:**
   ```powershell
   # Extract to a folder
   Expand-Archive -Path "fabrexlens-windows-amd64.zip" -DestinationPath "C:\Program Files\FabreXLens"

   # OR extract to user directory
   Expand-Archive -Path "fabrexlens-windows-amd64.zip" -DestinationPath "$env:USERPROFILE\FabreXLens"
   ```

4. **Add to PATH (optional):**
   ```powershell
   # Add to user PATH
   $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
   $newPath = "$env:USERPROFILE\FabreXLens;$userPath"
   [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
   ```

5. **Verify installation:**
   ```cmd
   fabrexlens.exe --version
   ```

## 🏗️ Building from Source

If you prefer to build from source or need a custom build:

### Prerequisites

- **Rust**: 1.75 or later
- **Cargo**: Latest stable version
- **System dependencies**:
  - Linux: `build-essential` or equivalent
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Windows: Visual Studio Build Tools or Rust's MSVC toolchain

### Install Rust

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Build the Application

```bash
# Clone the repository
git clone https://github.com/Digital-Data-Co/FabreXLens.git
cd FabreXLens

# Build in release mode
cargo build --release

# The binary will be at:
# Linux/macOS: target/release/fabrexlens
# Windows: target/release/fabrexlens.exe
```

### Run from Source

```bash
# Run directly (development mode)
cargo run

# Run with specific options
cargo run -- --help
```

## 🔧 Post-Installation Setup

After installation, you need to configure FabreXLens with your credentials:

### Initial Setup

```bash
# Initialize credentials for each service
fabrexlens auth-init --domain fabrex
fabrexlens auth-init --domain gryf
fabrexlens auth-init --domain supernode
fabrexlens auth-init --domain redfish
```

### Launch the Application

```bash
# Launch the GUI
fabrexlens

# OR run in headless mode
fabrexlens --headless
```

## 🔄 Upgrading

To upgrade to a new version:

1. **Download the new release** from [GitHub Releases](https://github.com/Digital-Data-Co/FabreXLens/releases)
2. **Replace the binary** with the new version
3. **Restart FabreXLens** if it's running

Your configuration and credentials will be preserved.

## 🐛 Verification

To verify your installation is working:

```bash
# Check version
fabrexlens --version

# Show help
fabrexlens --help

# Test credential setup (will prompt if needed)
fabrexlens auth-init --domain fabrex --dry-run
```

## 🆘 Troubleshooting Installation

### Common Issues

**"command not found"**
- Ensure the binary is in your PATH
- Try using the full path to the binary

**"Permission denied"**
- On Unix systems: `chmod +x fabrexlens`
- On macOS: Check System Preferences → Security & Privacy

**"DLL load failed" (Windows)**
- Ensure you have the Visual C++ Redistributables installed
- Try running as Administrator

**Build failures**
- Ensure Rust 1.75+ is installed: `rustc --version`
- Update Rust: `rustup update`
- Clear build cache: `cargo clean`

For more help, see the [Troubleshooting Guide](troubleshooting.md) or [open an issue](https://github.com/Digital-Data-Co/FabreXLens/issues).

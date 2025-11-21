# Building FabreXLens as a Standalone Binary

This guide explains how to build FabreXLens as a standalone, distributable binary.

## Prerequisites

- Rust toolchain (1.75+)
- `cargo` installed and on your PATH
- Platform-specific build tools (see below)

## Quick Build

### Release Build (Optimized)

```bash
cargo build --release
```

The binary will be located at: `target/release/fabrexlens`

### Release Build with Debug Symbols

For profiling or debugging release builds:

```bash
cargo build --profile release-with-debug
```

The binary will be located at: `target/release-with-debug/fabrexlens`

## Platform-Specific Notes

### macOS

The release build produces a standalone binary. For distribution:

1. **Code Signing** (optional but recommended):
   ```bash
   codesign --deep --force --verify --verbose --sign "Developer ID Application: Your Name" target/release/fabrexlens
   ```

2. **Create a .app bundle** (optional):
   - Use tools like `cargo-bundle` or `create-dmg` for macOS app distribution
   - Install: `cargo install cargo-bundle`
   - Build: `cargo bundle --release`

### Linux

The binary is statically linked where possible. For maximum portability:

```bash
# Build with static linking (requires musl target)
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

### Windows

The binary is a standalone `.exe` file. For distribution:

1. Build normally: `cargo build --release`
2. The binary is at: `target/release/fabrexlens.exe`
3. Consider using `cargo-wix` or Inno Setup for installer creation

## Binary Size Optimization

The release profile is configured for minimal binary size:

- **LTO (Link Time Optimization)**: Enabled for maximum optimization
- **Strip symbols**: Enabled to reduce binary size
- **Optimization level**: `z` (optimize for size)
- **Panic strategy**: `abort` (smaller binary, no unwinding)

Current optimizations reduce the binary size significantly compared to default release builds.

## Static Linking

The project is configured to use static linking where possible:

- **reqwest**: Uses `rustls-tls` instead of OpenSSL (no external SSL library needed)
- **All Rust dependencies**: Statically linked
- **System libraries**: May still be dynamically linked (platform-dependent)

For fully static binaries on Linux, use the `musl` target as shown above.

## Distribution

### Standalone Binary

The release binary is self-contained and can be distributed as-is. Users need:

- **macOS**: No additional dependencies (if code signed)
- **Linux**: May require system libraries (use musl target for full static linking)
- **Windows**: No additional dependencies

### Configuration Files

Users can optionally create configuration files at:

- **macOS/Linux**: `~/.config/FabreXLens/fabrexlens.toml`
- **Windows**: `%APPDATA%\FabreXLens\fabrexlens.toml`

See `README.md` for configuration details.

## Troubleshooting

### Build Fails with Linker Errors

- **macOS**: Install Xcode Command Line Tools: `xcode-select --install`
- **Linux**: Install build essentials: `sudo apt-get install build-essential` (Debian/Ubuntu)
- **Windows**: Install Visual Studio Build Tools or use `rustup` with MSVC toolchain

### Binary is Large

The release profile is already optimized for size. Further reduction options:

1. Use UPX compression (may trigger antivirus warnings):
   ```bash
   upx --best target/release/fabrexlens
   ```

2. Remove debug symbols manually:
   ```bash
   strip target/release/fabrexlens
   ```

### Runtime Errors

If the binary fails to run on a different system:

1. Check for missing system libraries: `ldd target/release/fabrexlens` (Linux)
2. Use musl target for Linux for fully static binaries
3. Ensure all required system libraries are present

## CI/CD Integration

Example GitHub Actions workflow for cross-platform builds:

```yaml
name: Build Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - uses: actions/upload-artifact@v3
        with:
          name: fabrexlens-${{ matrix.os }}
          path: target/release/fabrexlens*
```


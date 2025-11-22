# Release Process

This document describes how to create releases for FabreXLens.

## Automatic Releases

Releases are automatically created when you push a tag starting with `v` (e.g., `v0.1.0`, `v1.2.3`).

### Creating a Release

1. **Update version in Cargo.toml** (optional but recommended):
   ```bash
   # Edit Cargo.toml and update the version field
   ```

2. **Commit your changes**:
   ```bash
   git add .
   git commit -m "Prepare release v0.1.0"
   ```

3. **Create and push a tag**:
   ```bash
   git tag -a v0.1.0 -m "Release version 0.1.0"
   git push origin main
   git push origin v0.1.0
   ```

4. **GitHub Actions will automatically**:
   - Run all tests
   - Build binaries for all platforms (Linux, macOS, Windows)
   - Create a public GitHub release
   - Attach all build artifacts
   - Generate release notes from git commits

### Release Tag Format

- **Stable releases**: `v1.0.0`, `v1.2.3`, `v2.0.0`
- **Pre-releases**: `v1.0.0-alpha.1`, `v1.0.0-beta.1`, `v1.0.0-rc.1`

Pre-releases (containing `-alpha`, `-beta`, or `-rc`) will be marked as pre-releases on GitHub.

### Release Assets

Each release includes:

- **Binaries** for all platforms:
  - `fabrexlens-linux-x86_64` (glibc)
  - `fabrexlens-linux-x86_64-musl` (statically linked)
  - `fabrexlens-macos-x86_64` (Intel)
  - `fabrexlens-macos-arm64` (Apple Silicon)
  - `fabrexlens-windows-x86_64.exe`

- **Checksums** (SHA256) for verification
- **Archives** (`.tar.gz` for Unix, `.zip` for Windows)

### Manual Release Creation

You can also create a release manually through the GitHub UI:

1. Go to **Releases** → **Draft a new release**
2. Select or create a tag (e.g., `v0.1.0`)
3. Fill in the release title and description
4. Click **Publish release**

The workflow will automatically attach build artifacts to your manually created release.

### Release Notes

Release notes are automatically generated from git commits since the last tag. The format includes:

- List of commits since the previous release
- Links to download binaries for each platform
- Checksums for verification

You can customize the release notes by editing them in the GitHub release page after creation.

## Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for new functionality in a backwards compatible manner
- **PATCH** version for backwards compatible bug fixes

Example: `v1.2.3` = Major 1, Minor 2, Patch 3




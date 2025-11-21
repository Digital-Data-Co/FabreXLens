# GitHub Actions Workflows

This directory contains GitHub Actions workflows for automated building, testing, and releasing FabreXLens.

## Workflows

### `ci.yml` - Continuous Integration

**Triggers:**
- Push to `main`, `master`, or `develop` branches
- Pull requests to `main`, `master`, or `develop` branches

**What it does:**
- Quick code checks (formatting, clippy, tests)
- Fast feedback for PRs
- Runs on Ubuntu only for speed

**Duration:** ~2-5 minutes

### `build.yml` - Full Build and Release

**Triggers:**
- Push to `main`, `master`, or `develop` branches
- Pull requests to `main`, `master`, or `develop` branches
- Manual workflow dispatch
- Release creation (tags)

**What it does:**
- Runs full test suite
- Builds release binaries for:
  - Linux (x86_64, glibc and musl)
  - macOS (x86_64 and ARM64)
  - Windows (x86_64)
- Creates checksums for all binaries
- Uploads artifacts
- Creates GitHub releases when tags are pushed

**Duration:** ~15-30 minutes

## Build Targets

The build workflow creates binaries for:

| Platform | Target | Artifact Name |
|----------|--------|---------------|
| Linux (glibc) | `x86_64-unknown-linux-gnu` | `fabrexlens-linux-x86_64` |
| Linux (musl) | `x86_64-unknown-linux-musl` | `fabrexlens-linux-x86_64-musl` |
| macOS Intel | `x86_64-apple-darwin` | `fabrexlens-macos-x86_64` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `fabrexlens-macos-arm64` |
| Windows | `x86_64-pc-windows-msvc` | `fabrexlens-windows-x86_64` |

## Artifacts

All build artifacts are available in the Actions tab:
- Binaries are uploaded as artifacts (retained for 30 days)
- SHA256 checksums are included for verification
- Release builds create downloadable archives

## Creating a Release

1. Create and push a tag:
   ```bash
   git tag -a v0.1.0 -m "Release version 0.1.0"
   git push origin v0.1.0
   ```

2. Create a GitHub release:
   - Go to the Releases page
   - Click "Draft a new release"
   - Select the tag you just created
   - The workflow will automatically attach build artifacts

Alternatively, the workflow will automatically create a release when you create a release in the GitHub UI.

## Manual Workflow Dispatch

You can manually trigger the build workflow:
1. Go to Actions → Build and Test
2. Click "Run workflow"
3. Select branch and click "Run workflow"

## Caching

Both workflows use GitHub Actions caching to speed up builds:
- Cargo registry cache
- Cargo git cache
- Build artifacts cache

Cache keys are based on `Cargo.lock` hash, so dependencies are cached until lockfile changes.

## Troubleshooting

### Build fails on a specific platform
- Check the Actions logs for the specific job
- Common issues:
  - Missing system dependencies (handled automatically)
  - Network timeouts (retry the workflow)
  - Disk space (GitHub provides 14GB)

### Tests fail
- Check the test output in the CI workflow
- Run tests locally: `cargo test --verbose`

### Release creation fails
- Ensure you have write permissions to the repository
- Check that the release tag exists
- Verify `GITHUB_TOKEN` is available (automatic for public repos)


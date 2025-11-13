# FabreXLens

FabreXLens is a Rust-based desktop and command-line companion for GigaIO FabreX fabrics, Gryf workloads, and Supernode infrastructure. It provides a secure onboarding flow, rich observability UI, and automation hooks for reassignment and lifecycle operations powered by the FabreX, Gryf, and Redfish APIs.

## Highlights

- Secure credential capture using the system keychain with optional API token support.
- egui/eframe desktop UI with live dashboards, topology insights, and reassignment workflows.
- Asynchronous polling pipeline with configurable auto-refresh intervals and event logging.
- Strongly typed API clients for FabreX, Gryf, Supernodes, and Redfish with integration tests.
- Flexible configuration via CLI flags, profile files, or environment variables.

## Prerequisites

- Rust 1.75+ with `cargo` on your PATH.
- Access to GigaIO FabreX, Gryf, and Supernode endpoints plus API credentials.
- Operating system keychain support (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux).

## Getting Started

1. Install the Rust toolchain if needed: `curl https://sh.rustup.rs -sSf | sh`
2. Clone the repository and enter the folder:
   ```bash
   git clone git@github.com:Digital-Data-Co/FabreXLens.git
   cd FabreXLens
   ```
3. Provision credentials for each service (repeat per domain as needed):
   ```bash
   cargo run -- auth-init --domain fabrex
   cargo run -- auth-init --domain gryf
   cargo run -- auth-init --domain supernode
   cargo run -- auth-init --domain redfish
   ```
4. Launch the desktop UI:
   ```bash
   cargo run
   ```

The UI will automatically begin polling once credentials are available. Use the toolbar to toggle auto-refresh, adjust the interval, or trigger manual refreshes.

## Configuration

FabreXLens reads settings from (in priority order):

1. Command-line flags (`--config`, `--profile`, `--headless`).
2. Environment variables prefixed with `FABREXLENS__` (for example `FABREXLENS__FABREX_BASE_URL`).
3. Profile or default config files resolved under the OS config directory (`fabrexlens.toml`, `fabrexlens.dev.toml`, etc.).

See `src/config.rs` for the full schema and defaults.

## Project Structure

```
src/
  app.rs          # egui application shell & background workers
  cli.rs          # clap-based command-line interface
  config.rs       # layered configuration loader
  services/
    auth.rs       # credential management & keyring integration
    api/          # FabreX / Gryf / Supernode / Redfish clients
  ui/             # dashboard rendering primitives
.docs/
  architecture.md # high-level system design notes
  api/            # drop official API PDF references here
```

## Documentation

- `docs/architecture.md` outlines the core modules, runtime flow, and background polling model.
- Place official FabreX / Gryf / Supernode / Redfish PDF references inside `docs/api/` (they are gitignored by default).
- The CLI usage is discoverable via `cargo run -- --help`.

## Development Tips

- Run `cargo check` regularly to catch compile-time regressions.
- Use `cargo test` to execute mock-backed client integration tests.
- Detox credentials during development with `cargo run -- auth-init --domain <target> --scope <name>` and `keyring` GUI utilities.

## Roadmap

- Extend observability views with historical trend charts.
- Integrate streaming log tailing from Gryf workloads.
- Add packaged releases (Homebrew, winget) once the UI stabilises.

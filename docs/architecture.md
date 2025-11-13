# FabreXLens Architecture

## Runtime Overview

FabreXLens boots as a single binary that exposes both a clap-powered CLI and an egui/eframe desktop experience. The `main` entrypoint parses CLI arguments, optionally executes subcommands (e.g. `auth-init`), then feeds configuration and keyring handles into the GUI runtime (`src/app.rs`).

The UI layer owns a `FabreXLensApp` state machine that coordinates:

- A background worker thread hosting a Tokio runtime.
- Crossbeam channels for command (`AppCommand`) dispatch from the UI to the worker and event (`AppEvent`) propagation back.
- A secure `CredentialManager` abstraction that wraps the OS keyring, caches tokens, and builds `AuthContext` instances for API requests.

## Background Worker

The worker thread maintains a cloneable `ServiceContext` that vends strongly typed HTTP clients for FabreX, Gryf, Supernode, and Redfish services. It reacts to commands sent by the UI:

- `RefreshDashboard` performs an immediate telemetry fetch and emits a `DashboardUpdated`/`DashboardFailed` event.
- `SubmitReassignment` invokes the FabreX reassignment endpoint and forwards the result.
- `StartPolling`/`UpdatePolling`/`StopPolling` manage an async loop (backed by `tokio::time::interval`) that auto-refreshes metrics at a configurable cadence while credentials are available.

Polling is cancellable via a Tokio oneshot channel, ensuring a clean teardown when the UI disables auto-refresh or the process exits.

## UI Composition

The egui layer is split into:

- `DashboardState` (`src/ui/dashboard.rs`) which tracks the latest snapshot, loading/error flags, and renders tables for fabrics, workloads, and supernodes, plus aggregate statistics.
- `ReassignmentForm` for endpoint transfer workflows, leveraging combo boxes populated from the snapshot.
- A telemetry log view that surfaces background events (refresh results, credential issues, reassignment outcomes) for observability.

Top-level controls include manual refresh, credential rechecks, auto-refresh toggles, interval dials, and status banners. The UI reacts to worker events by updating state, logs, and status messages without blocking the rendering thread.

## API Clients

`src/services/api` houses modular clients built around a shared `HttpClient` wrapper. Key capabilities:

- Auth context support for bearer/basic schemes.
- Paginated GET helpers with typed DTOs.
- Reassignment and action endpoints that model FabreX, Gryf, and Supernode operations.
- Mock-backed integration tests (`httpmock`) to verify request/response wiring.

The clients pair with `CredentialManager` to assemble authenticated requests and manage token caches.

## Configuration & Extensibility

Configuration flows through `src/config.rs`, which merges defaults, profile files (per `ProjectDirs`), CLI overrides, and environment variables. The design keeps service base URLs, polling intervals, and application branding editable without code changes.

Future work can plug into the existing command/event mesh by extending `AppCommand`/`AppEvent` and adding worker handlers. The modular client layout supports feature growth (e.g. new FabreX endpoints) with isolated modules and tests.

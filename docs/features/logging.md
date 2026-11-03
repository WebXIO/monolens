# Logging

## Purpose

Rust and the web view write to one log, so a problem can be followed across the IPC boundary.

## How it works

- Rust logs with the `log` macros. `tauri-plugin-log` writes at `Debug` in development builds and `Info` in release builds (`lib.rs`), to its default targets: stdout and a file in the app's log directory (on Linux under `~/.local/share/org.monolens/logs`).
- What Rust logs:
  - the config path at start, a migration of `connections.json`
  - tasks being spawned, cancelled, completed, failed or panicking
  - a new client with its hosts, TLS, replica set and whether it authenticates, never the credential
  - failed finds with the server's message, a failing `buildInfo`
- The web view logs through `useLogger(context)`. It returns a `Logger` with the methods `trace` to `error` and a `LogStrategy`. The default `TauriLogStrategy` forwards to `@tauri-apps/plugin-log`, so UI messages land in the same log with the prefix `[WebView::<context>]`. Objects are written as JSON, functions as `[Function: name]`.
- The capability `log:default` lets the web view use the plugin. In tests the IPC is mocked, so logging code runs without Tauri.

## Decisions

- **One log for both sides through `tauri-plugin-log`** (`80c3925`).
- **A strategy behind the logger**, so the target can change without touching the callers (`80c3925`).
- **No credentials in logs** (`8d6bcac`, PR #32); since `5f6efaa` no connection string at all, only hosts and settings, and since `3c87afd` the dialog logs no form values.
- **`Info` in release builds** (`119bf4e`): less noise for users; `Debug` while developing.

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-11 | Log plugin, `Logger` with `TauriLogStrategy` | `80c3925` |
| 2026-02-22 | Credentials removed from logs | `8d6bcac` |
| 2026-11-03 | The dialog logs no form values | `3c87afd` |
| 2026-11-03 | Client settings logged instead of the URI | `5f6efaa` |
| 2026-11-03 | `Info` in release builds | `119bf4e` |

## Limits and open points

- Rotation and size limits are the plugin's defaults, nothing is configured. Issue #18 asks for more logs, a log file and rotation.

## Where the code is

| What | Path |
|---|---|
| Plugin setup | `src-tauri/src/lib.rs` |
| UI logger | `src/domains/logger/Logger.ts`, `src/domains/logger/strategies/`, `src/composables/useLogger.ts` |
| Client log line | `connect` in `src-tauri/src/drivers/mongo_official_driver.rs` |

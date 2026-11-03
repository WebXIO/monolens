# Logging

## Purpose

Rust and the web view write to one log, so a problem can be followed across the IPC boundary.

## How it works

- Rust logs with the `log` macros. `tauri-plugin-log` is set to level `Debug` in `lib.rs` and uses its default targets: stdout and a file in the app's log directory (on Linux under `~/.local/share/org.monolens/logs`).
- What Rust logs:
  - the config path at start
  - tasks being spawned, cancelled, completed, failed or panicking
  - a malformed URI, a filter that can't become BSON, a cursor that fails
  - in debug builds only, the built connection URI with the password replaced by `***`
- The web view logs through `useLogger(context)`. It returns a `Logger` with the methods `trace` to `error` and a `LogStrategy`. The default `TauriLogStrategy` forwards to `@tauri-apps/plugin-log`, so UI messages land in the same log with the prefix `[WebView::<context>]`. Objects are written as JSON, functions as `[Function: name]`.
- The capability `log:default` lets the web view use the plugin.

## Decisions

- **One log for both sides through `tauri-plugin-log`** (`80c3925`).
- **A strategy behind the logger**, so the target can change without touching the callers (`80c3925`).
- **No credentials in logs** (`8d6bcac`, PR #32): the URI only in debug builds and masked.

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-11 | Log plugin, `Logger` with `TauriLogStrategy` | `80c3925` |
| 2026-02-22 | Credentials removed from logs | `8d6bcac` |

## Limits and open points

- The level is `Debug` in release builds too. PR #32 ("remove log when in production") only took the URI out.
- `ConnectionDialog` logs the form values and the connection at `trace`, password included. `trace` is below the `Debug` filter today, so nothing is written. Lowering the level to `Trace` would put passwords in the log.
- Rotation and size limits are the plugin's defaults, nothing is configured. Issue #18 asks for more logs, a log file and rotation.

## Where the code is

| What | Path |
|---|---|
| Plugin setup | `src-tauri/src/lib.rs` |
| UI logger | `src/domains/logger/Logger.ts`, `src/domains/logger/strategies/`, `src/composables/useLogger.ts` |
| Masked URI | `build_connection_string` in `src-tauri/src/drivers/mongo_official_driver.rs` |

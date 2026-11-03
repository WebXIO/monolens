# Background tasks

## Purpose

Testing, connecting and finding documents can take long, up to the server selection timeout. They run as background tasks in Rust, so the UI stays responsive and the user can cancel them.

## How it works

**Rust** (`task_manager/`)

- `TaskManager` keeps two maps keyed by a uuid: the tokio `JoinHandle` of each task and its `AbortHandle`.
- A `start_*` command passes its future to `spawn_task`, which spawns it and returns the id at once. The future resolves to `Result<serde_json::Value, serde_json::Value>`: the result, or the serialized error.
- `await_task_result(id)` takes the `JoinHandle` out of the map and awaits it:
  - `Ok(value)` is the result.
  - The future's error becomes `TaskError::Failed(value)`.
  - An aborted task becomes `TaskError::Cancelled`, a panic becomes `Failed` with kind `Panic`.
  - Afterwards the abort handle is removed.
- `cancel_task(id)` calls `abort()` on the handle. An unknown id is `TaskError::NotFound`.
- `TaskError` serializes like every other error, with `kind`, `message` and `user_message`. For `Failed` the `message` is the JSON text of the original error.

The commands that use it are `start_test_connection`, `start_connect` and `start_find_documents`.

**UI** (`composables/useAbortableCommand.ts`)

- `abortableInvoke(startCommand, args, { signal })` invokes the start command, then `await_task_result`. If the `AbortSignal` fires in between, it calls `cancel_task`.
- Errors are unwrapped:
  - an aborted signal or kind `Cancelled` throws `TaskCancelledError`
  - kind `Failed` throws the original error, parsed back from `message`, so the caller gets its `user_message`
  - anything else is rethrown as it is
- `ConnectionServiceIPC.testConnection`, `connect` and `findDocuments` use it. The store keeps one `AbortController` for the test and one per connecting connection; the query tab keeps one per tab, and a new query cancels the one still running.

## Decisions

- **A task registry of our own instead of a timeout** (`a14c8fc`, issue #27). The user decides when to give up, and the same mechanism serves every slow command.
- **The error travels as JSON and is unwrapped in the UI**, so the toast shows the driver's message and not "The operation failed".

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-21 | `TaskManager`, `cancel_task`, `await_task_result`, `abortableInvoke`; test and connect as tasks | `a14c8fc` |
| 2026-02-23 | `start_find_documents` as a task | `cbc9b83` |
| 2026-11-03 | The query tab goes through `abortableInvoke` and can be cancelled | `16b6cd7` |

## Limits and open points

- A task that is never awaited stays in both maps. Today every start is awaited right after.
- Aborting drops the future. A MongoDB operation that already reached the server is not killed on the server.
- No tests for `TaskManager` and `abortableInvoke` yet (the IPC wrappers above them are tested).

## Where the code is

| What | Path |
|---|---|
| Registry | `src-tauri/src/task_manager/task_manager.rs` |
| Commands | `src-tauri/src/task_manager/commands.rs` |
| Error | `src-tauri/src/task_manager/task_error.rs` |
| UI helper | `src/composables/useAbortableCommand.ts` |
| Callers | `src/domains/connections/services/ConnectionServiceIPC.ts`, `src/stores/connectionStore.ts`, `src/components/tabs/views/TabViewQuery.vue` |

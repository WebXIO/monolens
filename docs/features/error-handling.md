# Error handling

## Purpose

Driver errors are long and technical. The user gets a short sentence that says what to check, and the technical text stays available to the code and the logs.

## How it works

**One shape for every error across IPC**: `{ kind, message, user_message }`.

| Type | Where | `kind` | `user_message` |
|---|---|---|---|
| `DriverError` | `drivers/errors.rs`, every database command | The variant: `ConnectionFailed`, `ClientNotInitialized`, `PingFailed`, `ListDatabasesFailed`, `ListCollectionsFailed`, `ServerStatusFailed`, `UriMalformed`, `UnknownDriver`, `FindDocumentsFailed` | Chosen per variant. For connect, ping, list and server status it looks into the raw message: one containing "authentication", "unauthorized" or "auth error" gives "Authentication failed. Check your username, password, …". For ping, "timeout" or "timed out" gives "Connection timed out. Check your host, port, and firewall settings." |
| `CommandError` | `connector/commands/error.rs`, the connection CRUD commands | `repository`, `io`, `serialization`, `tauri` | One fixed sentence per command, e.g. "Failed to load connections." |
| `TaskError` | `task_manager/task_error.rs` | `NotFound`, `Cancelled`, `Failed` | Fixed. For `Failed` the `message` is the original error as JSON, which `abortableInvoke` unwraps |

Inside the connector `RepositoryError` (not found, io, serialization, credential, migration) and `CredentialError` (keyring, join) are used. The commands turn them into a `CommandError`, with their text as `message`.

**In the UI**

- `extractErrorMessage(e)` returns, in this order, `user_message`, `message`, `Error.message`, a string error itself, or "An unexpected error occurred.".
- `vue-sonner` shows it as a toast: `<Toaster rich-colors theme="dark" position="top-center">` in `App.vue`. `connectionStore` toasts every failed call. `TabViewQuery` toasts query parse errors.
- `Result<T>` and `tryCatch` (`utils/result.ts`) turn a rejected `invoke` into `{ data: null, error }` for the CRUD calls.

## Decisions

- **Three fields, the UI shows `user_message`** (`b5a9455`, issue #16). The raw messages overflowed the dialog and told the user little.
- **Toasts instead of error boxes** (`37819e1`, issue #36). The "Show details" toggle that `b5a9455` added to the test display went away with them, so the technical `message` is no longer shown in the UI.

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-05 | `CommandError` for the connector | `4100b21` |
| 2026-02-09 | `DriverError` | `1f358d4` |
| 2026-02-17 | Long errors no longer overflow the dialog | `ae22de5` |
| 2026-02-18 | `RepositoryError` and `CredentialError` | `84839a6` |
| 2026-02-20 | `user_message` on every error, short text and details in the dialog | `b5a9455` |
| 2026-02-21 | `TaskError` | `a14c8fc` |
| 2026-02-23 | Toasts, the error boxes and the details toggle removed | `37819e1` |

## Limits and open points

- Auth and timeout errors are recognised by substrings of the driver's English message. A changed driver text breaks the classification silently.
- `FindDocumentsFailed` drops the driver's message (`map_err(|_| …)`). Neither the user nor the log learns why a query failed, except for a filter that can't be turned into BSON, which is logged as a warning.
- Query errors from the server are not shown at all, see [query-tab.md](query-tab.md#limits-and-open-points). Errors of `list_collections` are only logged.
- A keyring failure while connecting reads as a host problem, see [credential-storage.md](credential-storage.md#limits-and-open-points).
- All messages are English. Issue #31 asks for i18n (German and English first).

## Where the code is

| What | Path |
|---|---|
| Driver errors | `src-tauri/src/drivers/errors.rs` |
| Command and repository errors | `src-tauri/src/connector/commands/error.rs` |
| Task errors | `src-tauri/src/task_manager/task_error.rs` |
| UI | `src/utils/errorMessage.ts`, `src/utils/result.ts`, `src/composables/useAbortableCommand.ts`, `src/App.vue` |

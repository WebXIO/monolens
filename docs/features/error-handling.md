# Error handling

## Purpose

Driver errors are long and technical. The user gets a short sentence that says what to check, and the technical text stays available to the code and the logs.

## How it works

**One shape for every error across IPC**: `{ kind, message, user_message }`.

| Type | Where | `kind` | `user_message` |
|---|---|---|---|
| `DriverError` | `drivers/errors.rs`, every database command | `ConnectionFailed`, `ClientNotInitialized`, `PingFailed`, `ListDatabasesFailed`, `ListCollectionsFailed`, `ServerStatusFailed`, `UriMalformed`, `UnknownDriver`, `FindDocumentsFailed`, `InvalidQuery`, `ConnectionNotFound`, `StorageFailed`, `CredentialsUnavailable` | Chosen per variant. Connect, ping, list and server status look into the raw message: "authentication", "unauthorized" or "auth error" give "Authentication failed. Check …"; a ping timeout gives "Connection timed out. …". A failed find shows the server's message ("The query failed: unknown top level operator: $foo"), or "Not allowed to read this collection" without the rights. `InvalidQuery` is already a sentence for the user ("The sort must be an object like { field: value }, not a number") |
| `CommandError` | `connector/commands/error.rs`, the connection CRUD commands | `repository`, `invalid`, `io`, `serialization`, `tauri` | One sentence per command, e.g. "Failed to load connections."; for `invalid` (`RepositoryError::Invalid`) the reason why a connection was refused |
| `TaskError` | `task_manager/task_error.rs` | `NotFound`, `Cancelled`, `Failed` | Fixed. For `Failed` the `message` is the original error as JSON, which `abortableInvoke` unwraps |

Inside the connector `RepositoryError` (not found, invalid, io, serialization, credential, migration) and `CredentialError` (not found, keyring, join) are used.

**In the UI**

- `extractErrorMessage(e)` returns, in this order, `user_message`, `message`, `Error.message`, a string error itself, or "An unexpected error occurred.".
- `vue-sonner` shows it as a toast: `<Toaster rich-colors theme="dark" position="top-center">` in `App.vue`. The connection store toasts every failed call; the query tab toasts parse errors (naming the input: "Sort: …") and failed queries; the sidebar toasts failed collection lists.
- `Result<T>` and `tryCatch` (`utils/result.ts`) turn a rejected `invoke` into `{ data: null, error }` for the calls that aren't tasks.

## Decisions

- **Three fields, the UI shows `user_message`** (`b5a9455`, issue #16). The raw messages overflowed the dialog and told the user little.
- **Toasts instead of error boxes** (`37819e1`, issue #36). The "Show details" toggle of `b5a9455` went away with them.
- **Query errors keep the server's text** (`5a30f74`): it names the mistake in the query.
- **Credentials and storage get their own variants** (`119bf4e`): a keyring failure read as "verify your host" before.

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
| 2026-11-03 | `CredentialError::NotFound` | `a7b4c1a` |
| 2026-11-03 | `RepositoryError::Invalid` with its reason for the user | `5f6efaa` |
| 2026-11-03 | `ConnectionNotFound`, `StorageFailed`, `CredentialsUnavailable`; errors of the collection list shown | `119bf4e` |
| 2026-11-03 | `FindDocumentsFailed` with the server's message, `InvalidQuery` | `5a30f74` |
| 2026-11-03 | Query errors shown as toasts | `16b6cd7` |

## Limits and open points

- Auth and timeout errors are recognised by substrings of the driver's English message. A changed driver text breaks the classification silently (the unit tests in `errors.rs` cover today's texts).
- All messages are English. Issue #31 asks for i18n (German and English first).

## Where the code is

| What | Path |
|---|---|
| Driver errors | `src-tauri/src/drivers/errors.rs` |
| Command and repository errors | `src-tauri/src/connector/commands/error.rs` |
| Task errors | `src-tauri/src/task_manager/task_error.rs` |
| UI | `src/utils/errorMessage.ts`, `src/utils/result.ts`, `src/composables/useAbortableCommand.ts`, `src/App.vue` |

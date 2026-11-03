# Connection test

## Purpose

Before or after saving a connection, the user can check that the server answers and the credentials work. The test runs in stages, so a failure shows where it stopped.

## How it works

1. In `ConnectionDialog`, Test Connection validates the form and calls `connectionStore.testConnection` with the form values (including the typed password).
2. The store chooses a test id (`crypto.randomUUID()`), empties `testStages`, listens to the Tauri event `test-connection-progress`, and calls `ConnectionServiceIPC.testConnection`. That starts `start_test_connection(connection, testId)` through `abortableInvoke`, with `id: ''` because the connection may not be saved yet.
3. In Rust the task creates its own driver through the factory, not a pooled one, builds the client, and runs `test_connection`. It first reports every stage of `TEST_STAGES` as `pending`, then each result, as events `{ testId, stageIndex, stage }`:

   | # | Stage | Check | Result |
   |---|---|---|---|
   | 0 | Initialize | the client exists | passed / failed |
   | 1 | Ping Database | `ping` on the authentication database (default `admin`); the first command also authenticates | passed / failed, the test stops |
   | 2 | Reading status | `serverStatus` | passed; *skipped* with "No permission to read the server status" when MongoDB answers unauthorized (code 13); failed otherwise, the test stops |
   | 3 | Detecting version | `buildInfo`, which needs no privilege | passed with the detail "MongoDB 8.0.x"; failed without stopping the test |
   | 4 | Connected | — | passed |

4. The store takes only events with its own test id and builds the list from them; when the task finishes, the returned stages replace it.
5. `TestStagesDisplay` draws a horizontal stepper: a green check for passed, a red cross for failed, a yellow dash for skipped, a spinner on the stage that runs, the detail under the title, and "Starting the test…" before the first event.
6. A failing stage is reported as failed before the error returns. The store shows the error's `user_message` as a toast ("Authentication failed. …").
7. While the test runs, Cancel Test replaces the button; it aborts the task and shows "Test cancelled.". Closing the dialog clears the test state.

## Decisions

- **Stages streamed as events** (`a14c8fc`), and since `2e7d3e1` with a test id, so a listener only takes its own test.
- **A stepper** (`9229c45`) for readability.
- **Optional** (`963da88`, issue #26): saving does not need a passed test.
- **A cancellable task** (`a14c8fc`, issue #27).
- **A missing `serverStatus` privilege skips the stage** (`2e7d3e1`): a user who may only read one database can connect and read, so the test must not fail for them. The version comes from `buildInfo`, which every user may run.
- **The titles exist only in Rust** (`2e7d3e1`): the driver reports all stages as pending first.

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-09 | `test_connection` command | `1f358d4` |
| 2026-02-11 | Test button in the connection dialog | `01eb47b` |
| 2026-02-17 | Long errors no longer overflow | `ae22de5` |
| 2026-02-20 | Short error descriptions next to the detailed ones | `b5a9455` |
| 2026-02-20 | Test is optional | `963da88` |
| 2026-02-21 | Test as a cancellable task with stage events | `a14c8fc` |
| 2026-02-21 | Stages shown as a stepper | `9229c45` |
| 2026-02-23 | Errors as toasts | `37819e1` |
| 2026-11-03 | Stage states with skipped, version from `buildInfo`, failed stages marked, test id, titles only in Rust; the synchronous `test_connection` command removed | `2e7d3e1`, `119bf4e` |

## Limits and open points

- There is no timeout of our own. A host that doesn't answer runs into the server selection timeout: the connection's option, else the driver's default (30 s), unless the user cancels.
- The test checks the server, not the rights on a particular database.

## Where the code is

| What | Path |
|---|---|
| Stages | `test_connection` and `TEST_STAGES` in `src-tauri/src/drivers/mongo_official_driver.rs`, `TestStage`, `StageStatus` in `src-tauri/src/drivers/driver.rs` |
| Command and event | `src-tauri/src/database/commands/start_test_connection.rs` |
| Store | `testConnection`, `cancelTest` in `src/stores/connectionStore.ts` |
| UI | `src/components/connection/ConnectionDialog.vue`, `TestStagesDisplay.vue` |
| Tests | integration tests in `mongo_official_driver.rs`, `src/stores/test/connectionStore.test.ts` |

# Connection test

## Purpose

Before or after saving a connection, the user can check that the server answers and the credentials work. The test runs in stages, so a failure shows where it stopped.

## How it works

1. In `ConnectionDialog`, Test Connection validates the form and calls `connectionStore.testConnection` with the form values (including the typed password).
2. The store sets `testStages` to five pending stages, listens to the Tauri event `test-connection-progress`, and calls `ConnectionServiceIPC.testConnection`. That starts `start_test_connection` through `abortableInvoke`, with `id: ''` because the connection may not be saved yet.
3. In Rust the task creates its own driver through the factory, not the pooled one, builds the client, and runs `test_connection`. After each stage it emits `test-connection-progress` with `{ taskId: "", stageIndex, stage }`:

   | # | Stage | Check |
   |---|---|---|
   | 0 | Initialize | the client exists |
   | 1 | Ping Database | `ping` on the authentication database (default `admin`) |
   | 2 | Reading status | `serverStatus` |
   | 3 | Detecting version | `true` if `serverStatus` has a `version` string, else `false` (the test goes on) |
   | 4 | Connected | always `true` once reached |

4. The store writes each event into `testStages[stageIndex]`. When the task finishes, the returned stages replace the list.
5. `TestStagesDisplay` draws the stages as a horizontal stepper: a green check for passed, a red cross for failed, a spinner on the stage that is running, and coloured separators.
6. A failing stage returns its error. The store shows the `user_message` as a toast ("Authentication failed. …", "Connection timed out. …") and stops the spinner.
7. While the test runs, Cancel Test replaces the button. It aborts the task (`cancel_task`) and shows "Test cancelled.". Closing the dialog clears the test state.

## Decisions

- **Stages streamed as events** (`a14c8fc`): each stage shows when it really finishes, instead of all at the end.
- **A stepper** (`9229c45`) for readability.
- **Optional** (`963da88`, issue #26): saving does not need a passed test.
- **A cancellable task** (`a14c8fc`, issue #27): an unreachable host could block the UI.

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

## Limits and open points

- **A failing stage is not marked as failed.** The error returns before the stage's status is set or emitted, so the stepper stops at that stage and the reason only appears in the toast. The red cross only appears for "Detecting version".
- **`serverStatus` needs a privilege** (e.g. the `clusterMonitor` role). A user who may only read one database fails at "Reading status", although connecting and reading would work.
- The detected version is not shown anywhere.
- The event has an empty `taskId`, so a listener can't tell two tests apart. The UI only runs one at a time.
- The stage titles exist twice: in the driver and in `connectionStore._createPendingStages`.
- There is no timeout of our own. A host that doesn't answer runs into the driver's default server selection timeout (30 s), unless the user cancels.

## Where the code is

| What | Path |
|---|---|
| Stages | `test_connection` in `src-tauri/src/drivers/mongo_official_driver.rs`, `TestStage` in `src-tauri/src/drivers/driver.rs` |
| Command and event | `src-tauri/src/database/commands/start_test_connection.rs` |
| Store | `testConnection`, `cancelTest` in `src/stores/connectionStore.ts` |
| UI | `src/components/connection/ConnectionDialog.vue`, `TestStagesDisplay.vue` |

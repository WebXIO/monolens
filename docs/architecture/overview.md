# Architecture overview

Monolens is an open-source MongoDB GUI for the desktop (homepage: monolens.org, Apache-2.0). It is a Tauri 2 app: a Rust core talks to MongoDB, and a Vue 3 web view shows the UI. A user saves connections (host and port with options, or a connection string), opens one or several, browses their databases and collections, and opens a collection in a tab. There the user filters, sorts, projects and pages through documents and sees them as a table or as JSON. The app only reads data so far.

This file describes the state after the working-state round of 2026-11-03 ([plans/implemented/working-state.md](../plans/implemented/working-state.md)).

## The parts

```
 +------------------------------------------------------------+
 |  Web view (Vue 3, Pinia, Tailwind 4, shadcn-vue/reka-ui)   |
 |   ConnectionGrid / Dialog    AppSidebar     TabsContainer   |
 |   connectionStore (sessions) tabsStore      TabViewQuery    |
 |   CommandRegistry (shortcuts)  Logger -> tauri-plugin-log   |
 +------------------------------------------------------------+
               |  invoke("<command>", args)        ^ event
               v                                   | test-connection-progress
 +------------------------------------------------------------+
 |  Rust core (Tauri 2, tokio)                                 |
 |   connector/        saved connections, keyring, migration   |
 |   database/         test, connect, disconnect, list, find   |
 |   task_manager/     cancellable background tasks            |
 |   connection_handler/  one driver per connection id         |
 |   drivers/          DatabaseDriver, MongoDB, ClientOptions  |
 +------------------------------------------------------------+
      |                    |                    |
      v                    v                    v
  connections.json     OS keyring           MongoDB servers
  (app config dir)     (passwords)          (mongodb crate 3.9)
```

| Part | Technology | Path |
|---|---|---|
| Rust core | Tauri 2.12, tokio, `mongodb` 3.9 (bson 2.15), `keyring` 4.2, `percent-encoding`, `thiserror`, `tauri-plugin-log`, `tauri-plugin-opener` | `src-tauri/src` |
| UI | Vue 3.5, Pinia 4, Tailwind 4 with shadcn-vue components on reka-ui, TanStack Form + zod 4, vue-sonner, lucide icons (`@lucide/vue`) | `src` |
| Query editor and JSON view | CodeMirror 6 (`@codemirror/lang-json`, `theme-one-dark`, `autocomplete`) | `src/components/query`, `src/components/documents/json` |
| Table view | TanStack Table 9 and TanStack Virtual 3 | `src/components/documents/table`, `src/composables/table` |
| Tests | vitest 5 in happy-dom with Tauri's IPC mock; Rust unit tests and integration tests against a real MongoDB | `test/` folders next to the code, `#[cfg(test)]` modules |
| Generated UI kit | shadcn-vue (`components.json`, style `new-york`) | `src/components/ui` |
| Tooling | pnpm 12 (pinned), TypeScript 6, Vite 8, graphify | `package.json`, `.claude/` |

The app is one window of 1200 × 900 (`src-tauri/tauri.conf.json`), identifier `org.monolens`. In development Vite serves the UI on port 1420 and `pnpm tauri dev` loads it.

## The Rust core

`lib.rs` builds the app. On macOS it replaces Tauri's default menu with one without "Close Window" (`menu.rs`). In `setup` it creates three pieces of managed state:

- `AppState` with the connection repository: a `FileRepository` on `<app config dir>/connections.json`, with a `CredentialService` (the keyring, service `monolens-connections`) behind the `CredentialStore` trait.
- `ConnectionHandler`: a map from connection id to a shared `DatabaseDriver`.
- `TaskManager`: background tasks by uuid.

The log plugin writes at `Debug` in development builds and `Info` in release builds.

### Commands

| Command | Module | What it does |
|---|---|---|
| `get_connections` | connector | List saved connections, without passwords |
| `create_connection` | connector | Check and save a new connection; password and credentials of a connection string into the keyring |
| `update_connection` | connector | Check and replace a connection, update or delete its password, drop its driver |
| `delete_connection` | connector | Drop its driver, remove the connection and its password |
| `get_connection_password` | connector | Read the password from the keyring (edit dialog) |
| `start_test_connection(connection, testId)` | database | Test form data as a task, streams `test-connection-progress` events with the test id |
| `start_connect(connectionId)` | database | Build the driver from the saved connection and list the databases, as a task |
| `disconnect(connectionId)` | database | Drop the driver |
| `list_collections(connectionId, databaseName)` | database | Collection names of one database |
| `start_find_documents(connectionId, query)` | database | Find with filter, sort, projection, skip and limit, plus the count, as a task |
| `cancel_task` | task_manager | Abort a task |
| `await_task_result` | task_manager | Wait for a task and return its result |

### Long-running work

`start_test_connection`, `start_connect` and `start_find_documents` don't do the work in the command. They hand a future to `TaskManager.spawn_task`, which runs it on tokio and returns a uuid at once. The UI then calls `await_task_result` with that id and can call `cancel_task` to abort it. See [background-tasks.md](../features/background-tasks.md).

### Drivers and the connection pool

`DatabaseDriver` is the trait every database driver implements: `connect`, `disconnect`, `test_connection`, `list_databases`, `list_collections`, `find_documents`. `DatabaseDriverFactory` picks the implementation by `ConnectorKind`; there is one, `MongoDbOfficialDriver`. It builds its client from `ClientOptions` (`drivers/client_options.rs`) and converts between JSON and BSON in `drivers/documents.rs`.

`ConnectionHandler.get_or_connect(repository, id)` returns the driver of a connection, loading the saved connection and its password and connecting it on first use. A driver leaves the pool on `disconnect`, on update or delete of its connection, and when its first command fails. See [connect-and-explorer.md](../features/connect-and-explorer.md).

## The UI

- `main.ts` registers the domains (`src/domains/index.ts`), then the plugins (Pinia, then the shortcut plugin), and mounts `App.vue`.
- `src/domains` is a small provider registry. `registerDomains` creates `CommandRegistry` and `ConnectionServiceIPC` once and provides each under `$providers.<ClassName>`. `useDomain(Class)` reads one back through `inject`, or through `app.config.globalProperties` when given the app (plugins run outside components).
- The stores are Pinia setup stores:
  - `connectionStore` holds the saved connections, one session per open connection (status, databases, cached collections) and the state of the connection test.
  - `tabsStore` holds the open tabs, each with its connection, database and collection, and their history.
- `App.vue` shows `ConnectionGrid` while no connection is open. Once one is connected, it shows `AppSidebar` (one node per open connection) and `TabsContainer`.
- Errors reach the user as `vue-sonner` toasts; destructive actions ask through `ConfirmDialog` (`useConfirm`).

## How a query travels

1. The user clicks a collection in the sidebar. `tabsStore` finds the open query tab for that connection, database and collection, or adds one, and switches to it.
2. The tab's `TabViewQuery` mounts and runs its query: filter `{ }`, no sort or projection, skip 0, limit 50.
3. `parseQueryTexts` parses filter, sort and projection with the shell-syntax parser. Shell functions become extended-JSON objects, `ObjectId("…")` becomes `{ $oid: "…" }`; a parse error names the input and nothing is sent.
4. The view calls `ConnectionServiceIPC.findDocuments(connectionId, query)`, which starts `start_find_documents` and waits on `await_task_result`; a new query or Cancel aborts it.
5. In Rust the task gets the connection's driver from the pool (building it with the password if needed). The driver turns filter, sort and projection into BSON, runs `find` with skip, limit and `allowDiskUse`, and at the same time the count: estimated for an empty filter, exact otherwise.
6. The result is `{ documents, totalCount, totalCountEstimated }`. Documents come as relaxed extended JSON (dates as ISO strings, a large `Int64` as `{ "$numberLong" }`).
7. The view shows the documents in the table or the JSON view and "1–50 of 137". Paging repeats the executed query with another skip.

## Data the app keeps

| What | Where | Written by |
|---|---|---|
| Saved connections, without passwords | `<app config dir>/connections.json`, on Linux `~/.config/org.monolens/connections.json`: `{ "version": 2, "connections": { "<uuid>": {…} } }`. A migrated version 1 file stays next to it as `connections.v1.json` | `FileRepository` |
| Passwords of connections with authentication | OS keyring, service `monolens-connections`, user `connection:<uuid>:basic_password` | `CredentialService` |
| Column order of the table view | Web view `localStorage`, key `monolens:columnOrder:<connection id>:<database>:<collection>` | `useDocumentTable` |
| Logs | stdout and the log directory of `tauri-plugin-log`, on Linux under `~/.local/share/org.monolens/logs` | Rust `log` macros and the UI's `Logger` |

Open connections, tabs, queries and results live only in memory.

## Errors

Every error that crosses IPC has the same three fields:

| Field | Meaning |
|---|---|
| `kind` | The variant, e.g. `PingFailed`, `FindDocumentsFailed`, `invalid`, `Cancelled` |
| `message` | The technical message |
| `user_message` | A short text for the user, e.g. "Authentication failed. Check your username, password, and authentication database." |

`DriverError`, `CommandError` and `TaskError` serialize this way. `extractErrorMessage` in the UI prefers `user_message`. See [error-handling.md](../features/error-handling.md).

## Building and running

```bash
pnpm install            # pnpm 12.17.0, pinned; a plain pnpm switches to it
pnpm tauri dev          # vite on :1420 plus the Rust app
pnpm test:frontend --run
pnpm build              # vue-tsc --noEmit, then vite build (also the beforeBuildCommand of `tauri build`)
pnpm tauri build        # installers for the current platform
cd src-tauri && cargo test                  # unit tests
cd src-tauri && MONGODB_TEST_HOST=… MONGODB_TEST_PORT=… MONGODB_TEST_USER=… MONGODB_TEST_PASSWORD=… cargo test -- --ignored
```

Rust and the Tauri prerequisites must be installed (see the root README). On Ubuntu 26.04: `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`. `7ee4b70` (2026-02-15) notes that the build worked for deb, rpm and AppImage.

Checked on 2026-11-03 after the working-state round, Node 24.9, pnpm 12.17, Rust 1.93:

- `pnpm test:frontend --run`: 141 tests pass.
- `pnpm build` passes.
- `cargo test`: 81 unit tests pass; the 11 integration tests pass against a `mongo:8` container (root, a user with `read` on one database).
- `cargo clippy`: 2 warnings (`module_inception`).
- Not checked: `pnpm tauri build`, the app itself in a Tauri window, `mongodb+srv://` (needs DNS records), TLS against a server, macOS and Windows.

Before the round (`859c252`): 27 tests passed, `pnpm build` failed on an unused import in a test, and `cargo check` only seemed to pass because it reused build results from March: the Tauri prerequisites were not installed on the machine.

## Branches

The repository is `github.com/WebXIO/monolens`. Feature branches come off `develop` and are merged through GitHub pull requests. `develop` was merged into `main` once, in PR #35 on 2026-02-22; everything after it is only on `develop`.

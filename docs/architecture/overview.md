# Architecture overview

Monolens is an open-source MongoDB GUI for the desktop (homepage: monolens.org, Apache-2.0). It is a Tauri 2 app: a Rust core talks to MongoDB, and a Vue 3 web view shows the UI. A user saves connections, connects to one, browses its databases and collections, and opens a collection in a tab. There it filters documents with a shell-style query and sees them as a table or as JSON. The app only reads data so far.

This file describes `develop` as of 2026-11-03 (`859c252`). `main` is at `efee3a2` (2026-02-22) and has none of the query, tab view or document display work.

## The parts

```
 +------------------------------------------------------------+
 |  Web view (Vue 3, Pinia, Tailwind 4, shadcn-vue/reka-ui)   |
 |   ConnectionGrid / Dialog    AppSidebar     TabsContainer   |
 |   connectionStore            tabsStore      TabViewQuery    |
 |   CommandRegistry (shortcuts)  Logger -> tauri-plugin-log   |
 +------------------------------------------------------------+
               |  invoke("<command>", args)        ^ event
               v                                   | test-connection-progress
 +------------------------------------------------------------+
 |  Rust core (Tauri 2, tokio)                                 |
 |   connector/        CRUD of saved connections               |
 |   database/         test, connect, list, find               |
 |   task_manager/     cancellable background tasks            |
 |   connection_handler/  one driver per connection id         |
 |   drivers/          DatabaseDriver trait, MongoDB driver    |
 +------------------------------------------------------------+
      |                    |                    |
      v                    v                    v
  connections.json     OS keyring           MongoDB server
  (app config dir)     (passwords)          (mongodb crate 3.5)
```

| Part | Technology | Path |
|---|---|---|
| Rust core | Tauri 2.10, tokio, `mongodb` 3.5 (bson 2.15), `keyring` 3.6, `thiserror`, `tauri-plugin-log`, `tauri-plugin-opener` | `src-tauri/src` |
| UI | Vue 3.5, Pinia 4, Tailwind 4 with shadcn-vue components on reka-ui, vee-validate + zod, vue-sonner, lucide icons (`@lucide/vue`) | `src` |
| Query editor and JSON view | CodeMirror 6 (`@codemirror/lang-json`, `theme-one-dark`, `autocomplete`) | `src/components/query`, `src/components/documents/json` |
| Table view | TanStack Table 9 and TanStack Virtual 3 | `src/components/documents/table`, `src/composables/table` |
| Frontend tests | vitest 5 in happy-dom | `test/` folders next to the code |
| Generated UI kit | shadcn-vue (`components.json`, style `new-york`) | `src/components/ui` |

The app is one window of 1200 × 900 (`src-tauri/tauri.conf.json`), identifier `org.monolens`. In development Vite serves the UI on port 1420 and `pnpm tauri dev` loads it.

## The Rust core

`lib.rs` builds the app. In `setup` it creates three pieces of managed state:

- `AppState` with the connection repository: a `FileRepository` on `<app config dir>/connections.json`, with a `CredentialService` for the service name `monolens-connections`.
- `ConnectionHandler`: a map from connection id to a shared `DatabaseDriver`.
- `TaskManager`: background tasks by uuid.

The log plugin is set to `Debug` for the whole app.

### Commands

Every call from the UI is a Tauri command. These are all of them:

| Command | Module | What it does | Used by the UI |
|---|---|---|---|
| `get_connections` | connector | List saved connections, without passwords | yes |
| `get_connection` | connector | One saved connection | wrapped in `ConnectionServiceIPC`, never called |
| `create_connection` | connector | Save a new connection, password into the keyring | yes |
| `update_connection` | connector | Replace a connection, update or delete its keyring entry | yes |
| `delete_connection` | connector | Remove a connection and its keyring entry | yes |
| `get_connection_password` | connector | Read the password from the keyring | edit dialog |
| `start_test_connection` | database | Test a connection as a task, streams `test-connection-progress` events | yes |
| `start_connect` | database | Connect as a task and list the databases | yes |
| `list_collections` | database | Collection names of one database | sidebar |
| `start_find_documents` | database | Find with filter, skip and limit as a task, plus a count | query tab |
| `cancel_task` | task_manager | Abort a task | yes |
| `await_task_result` | task_manager | Wait for a task and return its result | yes |
| `test_connection` | database | Synchronous test, the version before tasks existed | no |
| `get_databases` | database | Synchronous connect and list, the version before tasks existed | no |

`ConnectionServiceIPC` also wraps `save_connection` and `save_connections`. Those commands do not exist in Rust any more (removed in `8a9638a`), so the wrappers would fail. Nothing calls them.

### Long-running work

`start_test_connection`, `start_connect` and `start_find_documents` don't do the work in the command. They hand a future to `TaskManager.spawn_task`, which runs it on tokio and returns a uuid at once. The UI then calls `await_task_result` with that id and can call `cancel_task` to abort it. See [background-tasks.md](../features/background-tasks.md).

### Drivers and the connection pool

`DatabaseDriver` is the trait every database driver implements: `connect`, `disconnect`, `test_connection`, `list_databases`, `list_collections`, `find_documents`. `DatabaseDriverFactory` picks the implementation by `ConnectorKind`. There is one kind, `MongoDb`, and one driver, `MongoDbOfficialDriver`.

`ConnectionHandler.get_or_connect` returns the driver for a connection id, creating and connecting it on first use. The driver stays in the map for the rest of the process: `ConnectionHandler.disconnect` exists, but no command calls it. See [connect-and-explorer.md](../features/connect-and-explorer.md#limits-and-open-points).

## The UI

- `main.ts` registers the domains (`src/domains/index.ts`), then the plugins (Pinia, then the shortcut plugin), and mounts `App.vue`.
- `src/domains` is a small provider registry. `registerDomains` creates `CommandRegistry`, `ConnectionServiceIPC` and `ConnectionRepositoryIPC` once and provides each under `$providers.<ClassName>`. `useDomain(Class)` reads one back through `inject`, or through `app.config.globalProperties` when given the app (plugins run outside components). The repository is registered but nothing uses it.
- The stores are Pinia setup stores:
  - `connectionStore` holds the saved connections, the active connection, its database names, and the state of the connection test.
  - `tabsStore` holds the open tabs and their history.
- `App.vue` shows `ConnectionGrid` while no connection is active. Once one is, it shows `AppSidebar` (the explorer) and `TabsContainer`.
- Errors reach the user as `vue-sonner` toasts (`<Toaster rich-colors theme="dark" position="top-center">` in `App.vue`).

## How a query travels

1. The user clicks a collection in the sidebar. `tabsStore` finds the open query tab for that connection, database and collection, or adds one, and switches to it.
2. `TabViewQuery` mounts and runs `execute()` with the default filter `{ }` and limit 50.
3. `parseMongoDbQuery` turns the shell-style text into a plain object. Shell functions become extended-JSON objects, `ObjectId("…")` becomes `{ $oid: "…" }`. A parse error is a toast and nothing is sent.
4. The view calls `start_find_documents` with the active connection, the tab's database and collection, the filter, `skip: 0` and the limit, then waits on `await_task_result`.
5. In Rust the task loads the password from the keyring if the connection uses authentication, gets the pooled driver, and runs `find` with `skip`, `limit` and `allowDiskUse`. bson's serializer turns the extended-JSON objects of the filter back into BSON types. A second call, `count_documents`, counts all matches.
6. The result is `{ documents, totalCount }`. Each document is turned into JSON by serde (`ObjectId` as `{ "$oid": … }`, dates as `{ "$date": { "$numberLong": … } }`).
7. The view shows the documents in the table or the JSON view. The field names of the first document feed the autocomplete of the query input.

## Data the app keeps

| What | Where | Written by |
|---|---|---|
| Saved connections, without passwords | `<app config dir>/connections.json`, on Linux `~/.config/org.monolens/connections.json`: `{ "version": 1, "connections": { "<uuid>": {…} } }` | `FileRepository` |
| Passwords of connections with authentication | OS keyring, service `monolens-connections`, user `connection:<uuid>:basic_password` | `CredentialService` |
| Column order of the table view | Web view `localStorage`, key `monolens:columnOrder:<connection id>:<database>:<collection>` | `useDocumentTable` |
| Logs | stdout and the log directory of `tauri-plugin-log` (its default targets), on Linux under `~/.local/share/org.monolens/logs` | Rust `log` macros and the UI's `Logger` |

Open tabs, queries and results live only in memory.

## Errors

Every error that crosses IPC has the same three fields:

| Field | Meaning |
|---|---|
| `kind` | The variant, e.g. `PingFailed`, `repository`, `Cancelled` |
| `message` | The technical message |
| `user_message` | A short text for the user, e.g. "Authentication failed. Check your username, password, and authentication database." |

`DriverError`, `CommandError` and `TaskError` serialize this way. `extractErrorMessage` in the UI prefers `user_message`. See [error-handling.md](../features/error-handling.md).

## Building and running

```bash
pnpm install
pnpm tauri dev          # vite on :1420 plus the Rust app
pnpm test:frontend      # vitest
pnpm build              # vue-tsc --noEmit, then vite build (also the beforeBuildCommand of `tauri build`)
pnpm tauri build        # installers for the current platform
```

Rust and the Tauri prerequisites must be installed (see the root README). `7ee4b70` (2026-02-15) notes that the build worked for deb, rpm and AppImage.

Checked on 2026-11-03 on `develop`, Node 24.9, pnpm 11.3, Rust 1.93:

- `pnpm test:frontend --run`: 3 files, 27 tests pass.
- `cargo check`: passes without warnings.
- `pnpm build` fails: `src/utils/queryParser/test/tokens.test.ts(3,1): error TS6133: 'T' is declared but its value is never read.` The tsconfig includes the test files and sets `noUnusedLocals`. `vite build` alone passes. So `pnpm tauri build` fails too until the import is removed.
- There are no Rust tests and no CI.

## Branches

The repository is `github.com/WebXIO/monolens`. Feature branches come off `develop` and are merged through GitHub pull requests. Branch names mix `feat/<topic>` and `feat_<topic>`. `develop` was merged into `main` once, in PR #35 on 2026-02-22. The 16 commits after it are only on `develop`.

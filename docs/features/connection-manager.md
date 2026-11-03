# Connection manager

## Purpose

A user saves the MongoDB servers they work with as connections: a name, host, port and optional username and password. They can edit and delete them, and pick one to connect to. It is the first screen of the app.

## How it works

**The connection**

| Field | Meaning |
|---|---|
| `id` | uuid v4, created in Rust when the connection is saved |
| `name` | Free text |
| `uri` | The host, e.g. `localhost`. Despite the name it is not a connection string, see the limits |
| `port` | 1 to 65535, default 27017 |
| `connector` | `MongoDb`, the only kind |
| `connectionType` | Always `Standalone`. The enum also has replica set, sharded cluster and DNS seed list, but no UI offers them |
| `authentication` | `kind` `NONE` or `BASIC`, `username`, `password`, `database` (the authentication database) |

**Where the user manages them**

- `ConnectionGrid` is the start screen while no connection is active. It shows one card per connection with name and `host:port`. Edit and delete appear on hover, and the card has a Connect button, which turns into Cancel while that connection connects. The last card adds a connection.
- `ConnectionSelector` sits in the header while a connection is active. Its dropdown lists the connections with a check next to the active one, switches to another one and has "Add new connection…". While a connection connects, an X next to it cancels.
- `ConnectionDialog` creates and edits:
  - Fields: name, host, port, a "Use Authentication" checkbox, and with it username, password (with a show/hide toggle) and auth database (default `admin`). TanStack Form validates the values with a zod schema when the user tests or saves (name and host required, port 1 to 65535); errors appear under the fields.
  - Test Connection and Create/Save stay disabled until every field shown is filled, so with authentication the password is required. Save is also disabled while a test runs.
  - Testing is optional, see [connection-test.md](connection-test.md).
  - Closing the dialog resets the form and the test state.
  - In edit mode the dialog loads the stored password from the keyring, see [credential-storage.md](credential-storage.md).

**Saving, editing, deleting**

- Create calls `create_connection` with the connection without id (`CreateConnection`). Rust creates the id, moves the password into the keyring and writes the connection without it to the file. The store adds the returned connection, then the grid or the selector reloads the whole list.
- Edit calls `update_connection(id, connection)`. With `BASIC` and a password, Rust writes the password to the keyring. With `NONE` it deletes the keyring entry. The password is stripped before the file is written. The store replaces the entry in its list with what the form sent.
- Delete asks with the browser's `confirm()`, then calls `delete_connection`. It removes the entry from the file and the keyring; keyring errors are ignored. If the deleted connection was active, the store clears the active connection and its databases.

**Storage**

- `FileRepository` keeps `<app config dir>/connections.json` (on Linux `~/.config/org.monolens/connections.json`) as `{ "version": 1, "connections": { "<id>": { … } } }`.
- Every call reads the whole file, and every change writes the whole file (pretty JSON). A missing or empty file means no connections.
- A file with another `version` than `STORAGE_VERSION` (1) fails with `MigrationError`, and the list can't be loaded. There is no migration code yet.

Errors come back as `CommandError` with a user message such as "Failed to load connections." and are shown as toasts.

## Decisions

- **The id is created in Rust** (`8a9638a`). The UI sends `CreateConnection` without an id, so it can't invent or reuse one.
- **A versioned map instead of an array** (`84839a6`, issue #24): O(1) access by id, and the version allows migrating the file later.
- **Passwords are not in the file** (`eb8fdd9`, `84839a6`): they live in the OS keyring, see [credential-storage.md](credential-storage.md).
- **Testing is optional** (`963da88`, issue #26).

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-05 | Connection model, file repository, `get_connections` and `save_connections` | `4100b21` |
| 2026-02-11 | CRUD commands, the grid, the dialog and the dropdown | `01eb47b` |
| 2026-02-15 | The connection store becomes a Pinia store and uses the registered `ConnectionServiceIPC` | `09d7e1f`, `aa891e0` |
| 2026-02-15 | `CreateConnection` without id, `save_connections` removed | `8a9638a` |
| 2026-02-17 | Long error messages no longer overflow the dialog | `ae22de5` |
| 2026-02-18 | Test and save stay disabled until all fields are filled | `0b11049` |
| 2026-02-18 | Fix: a wrong error message on every delete | `4fc2b1e` |
| 2026-02-18 | Storage as a versioned map keyed by id | `84839a6` |
| 2026-02-20 | Saving no longer requires a test | `963da88` |
| 2026-02-21 | Cancel a connect from the dropdown, new dialog layout | `571e414`, `c780905` |
| 2026-02-23 | Errors as toasts | `37819e1` |

## Limits and open points

- **Host and port only.** The driver always builds `mongodb://<host>:<port>/?authSource=<db>`. The host field's placeholder says "localhost or mongodb+srv://...", but SRV and full connection strings don't work: the prefix is added again. TLS, replica set name, timeouts, read preference and other URI options can't be set. Issue #4 asked for a timeout setting and is closed, but no timeout exists.
- **Connection types are not wired up.** `connectionType` is always `Standalone` and nothing reads it. The TypeScript enum sends `"SharedCluster"` for `ShardedCluster`, which Rust's `ShardedCluster` would not accept.
- **No storage migration.** The version is checked, but a mismatch is an error.
- **Edits don't reach a pooled driver.** After a connection was used in this session, a changed host, port or password only takes effect after a restart, see [connect-and-explorer.md](connect-and-explorer.md#limits-and-open-points).
- Delete uses the browser's `confirm()`, not a dialog of the app.
- `App.vue` and `ConnectionGrid` both load the connections on mount, so they are loaded twice at start.
- Dead code: `ConnectionRepositoryIPC` is registered but unused. The `saveConnection`/`saveConnections` wrappers call commands that no longer exist. `get_connection` is wrapped but never called. The Rust commands `test_connection` and `get_databases` are not used by the UI.
- Open issues: #40 quick connections in the dropdown (at most 5, "Browse more", edit and delete there), #45 stage types (Develop, Demo, Production, with a confirmation before changing production data), #28 start an in-memory MongoDB.

## Where the code is

| What | Path |
|---|---|
| Commands | `src-tauri/src/connector/commands/` |
| Models | `src-tauri/src/connector/models/`, `src/domains/connections/models/` |
| Storage | `src-tauri/src/connector/repositories/file_repository.rs`, `storage_wrapper.rs` |
| IPC wrapper | `src/domains/connections/services/ConnectionServiceIPC.ts` |
| Store | `src/stores/connectionStore.ts` |
| UI | `src/components/connection/ConnectionGrid.vue`, `ConnectionDialog.vue`, `ConnectionSelector.vue` |

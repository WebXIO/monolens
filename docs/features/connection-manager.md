# Connection manager

## Purpose

A user saves the MongoDB servers they work with as connections: by host and port with options, or by a `mongodb://` / `mongodb+srv://` connection string, with an optional username and password. They can edit and delete them, and open one or several of them. It is the first screen of the app.

## How it works

**The connection** (`connections.json` version 2)

| Field | Meaning |
|---|---|
| `id` | uuid v4, created in Rust when the connection is saved |
| `name` | Free text, required |
| `connector` | `MongoDb`, the only kind |
| `target` | `{ "kind": "Host", host, port }` or `{ "kind": "Uri", uri }`; a stored connection string never has user info |
| `authentication` | `kind` `NONE` or `BASIC`, `username`, `password` (never stored, see [credential-storage.md](credential-storage.md)), `database` (the authentication database) |
| `options` | For host targets only: `tls`, `tlsCaFile`, `tlsAllowInvalidCertificates`, `replicaSet`, `serverSelectionTimeoutSecs` (empty is the driver's default, 30 s). A connection string carries its own options |

**Where the user manages them**

- `ConnectionGrid` is the start screen while no connection is open. It shows one card per connection with name and address (`host:port` or the string). Edit and delete appear on hover. Connect turns into Cancel while that connection connects; the grid gives way once it is connected.
- The header dropdown "Connections" lists all saved connections, marks the open and connecting ones, opens further connections and has "Add new connection…". See [connect-and-explorer.md](connect-and-explorer.md).
- The sidebar has an edit button on every open connection.
- `ConnectionDialog` creates and edits:
  - Name, then a switch between "Host and port" and "Connection string".
  - Host mode: host, port, and collapsible options (TLS with CA file and "allow invalid certificates", replica set, timeout in seconds). The options open by themselves when a connection has any.
  - Connection string mode: one field. A hint says that a username and password in the string are moved out on save.
  - "Use Authentication" with username, password (show/hide toggle) and auth database. Empty means `admin` for a host, and the string's `authSource` (else `admin`) for a connection string.
  - TanStack Form validates the values with a zod schema (`connectionForm.ts`) when the user tests or saves, and shows what is wrong under each field: a missing name or host, a host that is really a connection string, a port outside 1–65535, a timeout outside 1–3600, a string without `mongodb://` or `mongodb+srv://`, missing username or password with authentication.
  - Testing is optional, see [connection-test.md](connection-test.md). In edit mode the dialog loads the stored password.

**Saving, editing, deleting**

- Before storing, and before a test, Rust checks the connection again (`models/normalize.rs`; it also refuses `host:port` in the host field) and moves credentials out of a connection string: the username into `authentication`, the password into the keyring (`models/uri.rs`). A refusal is `RepositoryError::Invalid`, and the UI shows its text, e.g. "Enter mongodb:// and mongodb+srv:// addresses as a connection string, not as a host."
- Create calls `create_connection` with the connection without id; Rust creates the id and returns the stored connection, which the store adds.
- Edit calls `update_connection(id, connection)`. With `BASIC` and a password, Rust writes the password; with `NONE` it deletes it; the file never gets one. The Rust core drops the connected driver of that connection; the store reloads the list and an open connection reconnects with the new settings.
- Delete asks with the app's confirmation dialog (`useConfirm`), disconnects an open connection (its tabs close), then calls `delete_connection`, which removes file entry, keyring entry and driver.

**Storage**

- `FileRepository` keeps `<app config dir>/connections.json` (on Linux `~/.config/org.monolens/connections.json`) as `{ "version": 2, "connections": { "<id>": { … } } }`. Every call reads the whole file, every change writes it.
- A version 1 file (until 2026-11-03: the host in `uri`, plus `port` and an unused `connectionType`) is migrated on the first load (`repositories/migrations.rs`) and kept next to it as `connections.v1.json`. A version 1 `uri` that is a connection string becomes a string target with its password moved to the keyring; a `host:port` loses the port after the host, the port field counted before too.
- Another version, or no version, is an error.

## Decisions

- **The id is created in Rust** (`8a9638a`).
- **A versioned map instead of an array** (`84839a6`, issue #24): O(1) access by id, and a version to migrate the file. Used for the first time with version 2 (`5f6efaa`).
- **Passwords are not in the file** (`eb8fdd9`, `84839a6`), see [credential-storage.md](credential-storage.md).
- **Testing is optional** (`963da88`, issue #26).
- **Host and port with options, or a connection string** (`5f6efaa`, `29aceb2`, issue #4): a string carries all its options, the option fields apply to host targets only. `connectionType` was never read and is gone; the topology comes from the string or the replica set option.
- **Credentials leave a connection string on save** (`5f6efaa`): no password in `connections.json`, and the stored string can be shown.
- **The form validates instead of disabling buttons** (`29aceb2`): the user sees what is missing.
- **TanStack Form with zod 4** (`3c87afd`): `@vee-validate/zod` only supports zod 3, vee-validate 5 has been a beta since 2025. Rejected: vee-validate 5 beta, an adapter of our own for vee-validate 4.
- **A dialog of the app for deleting** (`29aceb2`) instead of the browser's `confirm()`.

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
| 2026-11-03 | The dialog on TanStack Form and zod 4 | `3c87afd` |
| 2026-11-03 | Model version 2 with targets and options, migration of version 1, validation and moving credentials out of strings in Rust | `5f6efaa` |
| 2026-11-03 | Dialog with both modes and options, form logic in `connectionForm.ts`, confirmation dialog for deleting | `29aceb2` |
| 2026-11-03 | Several connections at once: the dropdown opens further connections, edit from the sidebar | `afd84c7` |
| 2026-11-03 | Loaded once at start; reloaded from Rust after an edit | `d5ff9e3` |
| 2026-11-03 | Review fixes: the auth database stays empty by default (a filled `admin` overrode a string's `authSource`), the edit dialog fills on every opening, `host:port` refused, the test normalizes like saving | `8fe2ca0` |

## Limits and open points

- Authentication is username and password only (MongoDB's default SCRAM). No X.509, LDAP, Kerberos, AWS or OIDC, and no client certificates.
- The CA file is a typed path; there is no file picker.
- Options are fields for host targets only. For anything else (read preference, several hosts, …) use a connection string.
- `mongodb+srv://` resolves DNS records when connecting; it was not tested against a real srv cluster.
- Open issues: #40 quick connections in the dropdown (at most 5, "Browse more", edit and delete there), #45 stage types (Develop, Demo, Production, with a confirmation before changing production data), #28 start an in-memory MongoDB.

## Where the code is

| What | Path |
|---|---|
| Commands | `src-tauri/src/connector/commands/` |
| Models, checks, connection strings | `src-tauri/src/connector/models/` (`connection.rs`, `options.rs`, `normalize.rs`, `uri.rs`), `src/domains/connections/models/` |
| Storage and migration | `src-tauri/src/connector/repositories/file_repository.rs`, `migrations.rs`, `storage_wrapper.rs` |
| IPC wrapper | `src/domains/connections/services/ConnectionServiceIPC.ts` |
| Store | `src/stores/connectionStore.ts` |
| UI | `src/components/connection/ConnectionGrid.vue`, `ConnectionDialog.vue`, `connectionForm.ts`, `ConnectionSelector.vue`, `src/components/ConfirmDialog.vue`, `src/composables/useConfirm.ts` |

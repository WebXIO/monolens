# Working state: bugs fixed, querying complete, several connections at once

> **Status:** implemented on 2026-11-03 on `fix/working_state` (`520f438` to `d5ff9e3`, docs in the following commit). The feature files describe the result; the most changed are [connection-manager.md](../../features/connection-manager.md), [connect-and-explorer.md](../../features/connect-and-explorer.md) and [query-tab.md](../../features/query-tab.md).
>
> Deviations:
> - **A package update came in between**, at Kilian's request after step 1: pnpm 12 pinned (`e19ec32`), all npm packages (`3493da8`, with the TanStack Table 9 migration and `@lucide/vue` for the deprecated `lucide-vue-next`), forms moved from vee-validate to TanStack Form because `@vee-validate/zod` only supports zod 3 (`3c87afd`), all crates with keyring 4 (`955c08f`). TypeScript stays on 6, vue-tsc can't run on 7.
> - Steps 2 and 3 swapped places. Step 4 only brought the credential trait; model v2, migration and `ClientOptions` came together in step 5 with the TypeScript model, so every commit leaves a working app. Removing `get_connection` in step 6 also removed `ConnectionRepositoryIPC` and the `save_connection(s)` wrappers of step 12.
> - Not in the plan: the user info of a stored connection string goes back into it in memory before parsing (the driver checks `authMechanism` against it); the store reloads the list from Rust after an edit (the form's copy could hold credentials of a string); `update_connection` keeps the id of the path; closing the last tab leaves no stale active tab; the dialog validates instead of disabling its buttons.
> - The Tauri Linux prerequisites were not installed on the development machine, so the first `cargo check` only reused builds from March; they were installed during the round.
>
> Verified: 141 frontend tests, 81 Rust unit tests, 11 integration tests against a `mongo:8` container (root and a user with `read` on one database), the UI in a browser with a stand-in for the IPC. **Not verified**: the app in a Tauri window, `pnpm tauri build`, `mongodb+srv://`, TLS against a server, the migration of a real `connections.json` (unit-tested with its shape), macOS (`Cmd` and the menu) and Windows.

## Context

Monolens was picked up again on 2026-11-03 after a pause since March. The documentation written that day (`83ee6a3`) lists what `develop` does and what is broken. Kilian's goal for this round: a working state, with queries and everything around them, as the base for developing the tool further. Writing data comes after.

What is broken or missing today, read from the code. The feature files have the details:

- **Build**: `pnpm build` and `pnpm tauri build` fail on an unused import in `tokens.test.ts` ([app-shell.md](../../features/app-shell.md)).
- **Connections**:
  - The Rust pool never drops a driver, so edits and a corrected password need a restart ([connect-and-explorer.md](../../features/connect-and-explorer.md)).
  - The host field suggests `mongodb+srv://`, but only host and port work ([connection-manager.md](../../features/connection-manager.md)).
  - `connectionType` is never read, and its TypeScript value `"SharedCluster"` doesn't match Rust's.
  - A keyring failure reads as a host problem ([credential-storage.md](../../features/credential-storage.md)).
- **Connection test**: a failing stage isn't marked failed, and a user without the `serverStatus` privilege fails although connecting would work ([connection-test.md](../../features/connection-test.md)).
- **Tabs and shortcuts** ([tabs.md](../../features/tabs.md), [shortcuts.md](../../features/shortcuts.md)):
  - Every tab's container keys the view by the active tab, so views multiply and run the query several times.
  - `Ctrl+Enter` runs in every view (#42).
  - `CommandRegistry.unsubscribe` calls the subscriber.
  - Tabs survive a switch of connection and then query the new server.
- **Queries** ([query-tab.md](../../features/query-tab.md)):
  - Errors from the server are swallowed.
  - A query can't be cancelled.
  - No paging, sort or projection; the total is counted but never shown.
  - The autocomplete only knows the first document's fields.
  - Several shell functions silently become filters that match nothing (`ISODate("2024-01-01")`, a bad `ObjectId`, `BinData`).
- **Errors and logs**: `FindDocumentsFailed` drops the server's message ([error-handling.md](../../features/error-handling.md)). The log level is `Debug` in release builds, and the dialog logs passwords at `trace` ([logging.md](../../features/logging.md)).
- **Display**: `Int64` above 2^53 loses precision ([document-views.md](../../features/document-views.md)).
- **Dead code** ([overview.md](../../architecture/overview.md#commands)):
  - wrappers for Rust commands that no longer exist
  - an unused repository layer
  - two unused synchronous commands
  - leftovers of the Vite template

## Decisions

| # | Decision | Why | Status |
|---|---|---|---|
| 1 | **Several connections at once.** Each open connection is a node in the sidebar with its databases and collections. Tabs of all open connections sit side by side, and each tab belongs to its connection | Kilian's choice. Working with two servers (e.g. staging and local) is common | agreed |
| 2 | **Querying gets paging, the total count, sort and projection** | Kilian's choice. Issue #12 listed them and was closed without them | agreed |
| 3 | **Connection strings plus options.** A connection is either host and port with option fields (TLS, replica set, timeout), or a `mongodb://` / `mongodb+srv://` connection string | Kilian's choice. Closes issue #4 properly | agreed |
| 4 | **Also in this round:** remove dead code; a connection test that goes on without the `serverStatus` privilege; a dialog of the app instead of `confirm()` for deleting; `Cmd` instead of `Ctrl` on macOS | Kilian's choice | agreed |
| 5 | Commands that talk to MongoDB take a **connection id**, not the connection object. Rust loads the saved connection and its password itself. The connection test keeps taking the form data, because it may not be saved | The UI's copy can be stale after an edit, and the password no longer passes through the web view for these calls. The pool is keyed by id anyway | agreed |
| 6 | **A driver leaves the pool** on disconnect, on update and delete of its connection, and when its first command after creation fails. The next command builds a new one | Fixes "edits need a restart" and "a wrong password sticks" | agreed |
| 7 | **Connection model v2** with `target` (`Host { host, port }` or `Uri { uri }`) and `options`. `connectionType` is dropped. `STORAGE_VERSION` becomes 2; a v1 file is migrated on load and written back. Keyring entries stay as they are (keyed by id) | The topology is expressed by the connection string or the replica set option. `connectionType` was never read | agreed |
| 8 | **URI mode keeps everything but the credentials in the URI.** On save, Rust moves credentials found in the URI out: the username to `authentication.username`, the password to the keyring. The stored URI has no user info. The option fields apply to host mode only | One place for each setting, and no password in `connections.json` | agreed |
| 9 | **The client is built from `ClientOptions`**, not from a URL string. Host mode uses the builder, URI mode `ClientOptions::parse`; then the credential is set | Replaces the string building with `url`, its percent-encoding, and the `mongodb://` prefix that broke SRV | agreed |
| 10 | **Host-mode options:** TLS on/off, CA file, allow invalid certificates, replica set name, server selection timeout in seconds. Empty means the driver's default (30 s) | What issue #4 and decision 3 ask for, without client certificates (X.509 is out of scope) | agreed |
| 11 | **Disconnect closes the tabs of that connection.** `Mod+Shift+D` disconnects the connection of the active tab. Deleting an open connection disconnects it first | With several connections, a tab without its connection has nothing to show | agreed |
| 12 | **Sidebar per connection:** refresh, edit and disconnect on each connection node; collections are cached per connection in the store. The header dropdown opens further connections and has "Add new connection…". Editing an open connection drops its driver and reloads its databases; its tabs stay | Editing must work while connected, and the grid only shows when nothing is open | agreed |
| 13 | **A tab shows the collection, with connection and database as second line.** The view is keyed by its own tab id, gets the tab id as a prop and reads its own context | Fixes the multiplied views and the wrong connection; the label tells two `users` collections apart | agreed |
| 14 | **The count runs next to the find:** `estimated_document_count` for an empty filter, `count_documents` otherwise | Counting a big collection with `countDocuments` scans it. The estimate reads metadata | agreed |
| 15 | **Documents leave Rust as relaxed extended JSON** (dates as ISO strings), except `Int64` beyond ±2^53, which stays `{ "$numberLong": "…" }` | Readable dates in the JSON view, and no silent precision loss | agreed |
| 16 | **Parser fixes and additions:** `ISODate`, `Date` and `new Date` with a date-only or zone-less string become UTC RFC 3339, an invalid date is a parse error; `ObjectId` must be 24 hex characters; `BinData` subtype as hex; `RegExp(…)` and `/re/flags` literals become `$regularExpression`, so they also work inside `$in`; `NumberInt`; `UUID(…)` as binary subtype 4; dotted keys without quotes; numbers with exponents; nothing may follow the value | Shell syntax users type every day, and every case that silently matched nothing becomes an error or works | agreed |
| 17 | **Query errors show the server's message**, e.g. "unknown operator: $foo". `FindDocumentsFailed` carries it | It names the mistake in the query | agreed |
| 18 | **Connection test stages:** Initialize, Ping (also authenticates), Reading status (`serverStatus`, *skipped* without the privilege, the test goes on), Detecting version (`buildInfo`, needs no privilege; the version goes into the title), Connected. A failing stage is marked failed. The driver first emits all stages as pending, so the titles exist only in Rust. Events carry a test id from the UI | Decision 4's lenient stage, plus the bugs of the stepper | agreed |
| 19 | **`DriverError::CredentialsUnavailable`** for keyring failures, with its own message | The message pointed at host and port | agreed |
| 20 | **Log level `Info` in release, `Debug` in debug builds.** The dialog no longer logs form values | Less noise, no password even at `trace` | agreed |
| 21 | **Bindings as `mod+…`:** `Ctrl` on Linux and Windows, `Cmd` on macOS. On macOS Tauri's default menu binds `Cmd+W` to "Close Window"; `lib.rs` builds the default menu without that item on macOS | Decision 4. Without the menu change `Cmd+W` closes the app instead of the tab | agreed |
| 22 | **A `ConfirmDialog` with a `useConfirm()` composable**, built from the existing shadcn `Dialog` | Decision 4; also needed later for deleting data | agreed |
| 23 | **Tests:** vitest with `happy-dom` and Tauri's `mockIPC` for stores and `abortableInvoke`; Rust unit tests with an in-memory credential store behind a `CredentialStore` trait and temp directories; Rust integration tests against a local MongoDB marked `#[ignore]`, run with `cargo test -- --ignored` and `MONGODB_TEST_URI` | The rule "every piece of logic gets unit tests". The keyring and MongoDB can't be in the default test run | agreed |

## How it works afterwards

**`connections.json`, version 2**

```json
{
  "version": 2,
  "connections": {
    "<id>": {
      "id": "<id>",
      "name": "Local",
      "connector": "MongoDb",
      "target": { "kind": "Host", "host": "localhost", "port": 27017 },
      "authentication": { "kind": "BASIC", "username": "root", "password": null, "database": "admin" },
      "options": {
        "tls": false,
        "tlsCaFile": null,
        "tlsAllowInvalidCertificates": false,
        "replicaSet": null,
        "serverSelectionTimeoutSecs": null
      }
    }
  }
}
```

A connection string looks like `"target": { "kind": "Uri", "uri": "mongodb+srv://cluster0.example.net/?retryWrites=true" }`, and its `options` are ignored. The v1 → v2 migration turns `uri` and `port` into a `Host` target, drops `connectionType` and sets default options.

**Commands**

| Command | Change |
|---|---|
| `start_test_connection(connection, testId)` | Takes the form data as before, plus the test id the events echo |
| `start_connect(connectionId)` | By id; drops the driver again if listing the databases fails |
| `disconnect(connectionId)` | New: drops the pooled driver |
| `list_collections(connectionId, databaseName)` | By id, loads the password |
| `start_find_documents(connectionId, databaseName, collectionName, filter, sort, projection, skip, limit)` | Sort and projection added; the count runs alongside |
| `update_connection`, `delete_connection` | Also drop the pooled driver |
| `test_connection`, `get_databases`, `get_connection` | Removed (unused) |

**UI**

```
+--------------------------------------------------------------+
| [logo] | Connections v                                        |
+------------------+-------------------------------------------+
| EXPLORER         | users        orders        users          |
| v Local   ⟳ ✎ ⏏ | Local·shop   Local·shop    Staging·crm    |
|   v shop         +-------------------------------------------+
|     users        | Filter [{ age: { $gt: 18 } }         ] ▶  |
|     orders       | Sort   [{ createdAt: -1 }]  Projection [] |
| > Staging ⟳ ✎ ⏏ | 51–100 of 1,234   ‹ ›   Limit 50   ▦ {}   |
+------------------+-------------------------------------------+
```

The grid shows only while no connection is open.

## Steps

Every step is one commit with its tests. They run in this order, because later steps use the earlier ones.

1. **Build**: remove the unused import in `tokens.test.ts`. The test named "throws" gets a name and an assertion.
2. **Shortcuts**:
   - `unsubscribe` uses `indexOf`.
   - `eventToKeyString` emits `mod`, and the bindings become `mod+w`, `mod+shift+d`, `mod+enter`.
   - On macOS, `lib.rs` builds Tauri's default menu without "Close Window".
   - Tests for the registry and the key strings.
3. **Query parser**: decision 16, with tests per case.
4. **Rust: credential store and model v2**:
   - `CredentialStore` trait with the keyring implementation and an in-memory one for tests.
   - `ConnectionTarget`, `ConnectionOptions`.
   - `STORAGE_VERSION` 2 with the v1 migration in `FileRepository.load_storage`.
   - Moving credentials out of URIs on create and update.
   - Tests for migration, save, update, delete and credential moving, with temp directories.
5. **Rust: client options**: build `ClientOptions` from host mode or a URI, set the credential and TLS options (decisions 9 and 10). Unit tests for the options built.
6. **Rust: commands by id and the pool**:
   - Decisions 5 and 6.
   - `disconnect`.
   - `hydrate` everywhere.
   - `CredentialsUnavailable`.
   - The log level.
   - Removing `test_connection`, `get_databases` and `get_connection`.
   - Unit tests for pool invalidation (with a fake driver from the factory) and `DriverError::user_message`.
   - Integration tests marked `#[ignore]`.
7. **Connection test**: decision 18 in the driver and the command; the store builds its stages from the events; `TestStagesDisplay` shows skipped and failed stages.
8. **Rust: find**: sort, projection, the count strategy, relaxed extended JSON (decisions 14, 15, 17). Integration tests.
9. **UI: connections**:
   - TypeScript models v2.
   - `ConnectionDialog` with a mode switch (host and port with options, or connection string) and credentials in both modes.
   - `ConfirmDialog` and `useConfirm` for deleting.
   - The dialog's password logs removed.
10. **UI: several connections**:
    - `connectionStore` with open sessions per connection (status, databases, collections, abort controller).
    - The sidebar with one node per connection.
    - The header dropdown.
    - Tabs with `{ connectionId, database, collection }`, keyed by their own id, closed on disconnect.
    - `mod+enter` only in the active tab.
    - Store tests with `mockIPC`.
11. **UI: query tab**:
    - Through `ConnectionServiceIPC.findDocuments` with cancel.
    - Error toasts.
    - Sort and projection inputs.
    - Paging with "x–y of N".
    - The page resets when filter, sort, projection or limit change.
    - Autocomplete from all documents.
12. **Cleanup**:
    - `ConnectionRepository(IPC)` and the `save_connection(s)` wrappers.
    - The double load of the connections.
    - The guard in the JSON view's watcher.
    - Template leftovers: favicon, title, `vite.svg`, `tauri.svg`, `vue.svg`.
13. **Docs**:
    - Update the feature files (building steps, "How it works", limits), `overview.md`, `decisions.md`, `CLAUDE.md` (gotchas that are gone) and the root README feature list.
    - This plan gets its status note and moves to `implemented/`.

## Verification

- Automated, after every step: `pnpm test:frontend --run`, `pnpm build`, `cargo test`, `cargo clippy`.
- From step 6 on, also `cargo test -- --ignored` against `docker run --rm -p 27017:27017 mongo:8`.
- Manually in `pnpm tauri dev` against local containers:
  - without authentication
  - with authentication: root, plus a user with only `readWrite` on one database for the lenient stage
  - a one-node replica set (`--replSet rs0`, `rs.initiate()`) in host mode with the replica set name
  - TLS with a self-signed certificate, once with the CA file and once with "allow invalid certificates"
  - two connections at once, tabs of both, disconnecting one
  - paging, sort, projection, a broken query (toast), cancelling a slow query
  - editing an open connection (new password takes effect without restart), deleting an open connection
- Migration: back up `~/.config/org.monolens/connections.json` (three v1 connections), start the new build, check the file is v2 and the connections still connect.
- Can't be verified here:
  - `mongodb+srv://`, which needs DNS SRV records, e.g. an Atlas cluster, if Kilian has one.
  - `Cmd` and the menu, which need a Mac.

## Out of scope

- Writing data (#5, #6, #7, #9, #43) and nested objects on double click (#39).
- Saving column widths.
- A CSP, i18n (#31), log file rotation (#18).
- Query builder (#11, #41), export as code (#13).
- Stage types (#45), quick connections (#40), database statistics (#14), in-memory MongoDB (#28).
- Authentication other than username and password, and client certificates.
- CI.

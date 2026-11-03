# Decision log

Decisions that shape the app, newest first within each area. Each entry says what was decided, why, and what was rejected. Details live in the feature files.

This log was written on 2026-11-03 from the commit history, the pull requests and the GitHub issues. Where they don't record a reason, the entry says so. Add new decisions as they are made.

## Platform and structure

| Date | Decision | Why | Rejected |
|---|---|---|---|
| 2026-11-03 | All packages to their latest release, except TypeScript 7 and Tauri 3 (`3493da8`, `955c08f`) | The project was picked up again; old versions would only make later updates bigger. vue-tsc can't run on TypeScript 7 (no `lib/tsc`), Tauri 3 is an alpha | Staying on the old versions; TypeScript 7 without type-checking the templates |
| 2026-11-03 | pnpm pinned to 12 in `packageManager` and `devEngines`; `pnpm-workspace.yaml` allows the build scripts of esbuild and vue-demi (`e19ec32`) | node_modules came from pnpm 10 while the machines run newer versions; the same setup as smtp-relay | No pin; pinning the installed 11.3 |
| 2026-11-03 | A `CredentialStore` trait and a driver factory in `ConnectionHandler` (`a7b4c1a`, `119bf4e`) | Repository and pool can be tested without the keyring and without MongoDB; the commands no longer downcast the repository | |
| 2026-02-05 | Database access goes through a `DatabaseDriver` trait with a factory per `ConnectorKind` (`4100b21`) | Room for more drivers or a second MongoDB implementation. The commit only lists the parts | |
| 2026-02-05 | Dependency injection on both sides: Tauri managed state in Rust, a provider registry (`src/domains`) with `useDomain` in Vue (`4100b21`, `aa891e0`) | Services are created once and can be swapped behind an interface (`ConnectionService`, `ConnectionRepository`, `LogStrategy`). The history records no further reason | |
| 2026-02-02 | Tauri 2 with a Rust core and a Vue 3 web view | The README calls Rust and Tauri a "high-performance backend with secure system-level access", Vue and Tailwind "modern, responsive, and maintainable". The goal is a free, European alternative to proprietary tools like Studio 3T | |

## Connections and credentials

| Date | Decision | Why | Rejected |
|---|---|---|---|
| 2026-11-03 | Commands that talk to MongoDB take a connection id; Rust loads the saved connection and its password (`119bf4e`) | The UI's copy can be stale after an edit, and the password no longer passes through the web view for these calls | Sending the connection object as before |
| 2026-11-03 | A driver leaves the pool on disconnect, edit, delete and when its first command fails (`119bf4e`) | A driver is created before the server is contacted, so a wrong password or an edited host stuck until restart | Keeping drivers for the life of the process |
| 2026-11-03 | Connection model version 2: `target` (host and port, or a connection string) and `options` for host targets; `connectionType` dropped; v1 files migrated and kept as `connections.v1.json` (`5f6efaa`) | Connection strings and options (issue #4) were out of reach with a host field and a URL built by hand; `connectionType` was never read | Keeping `connectionType`; options for both modes, overriding the string |
| 2026-11-03 | A connection string keeps all its options; credentials in it move out on save, the password to the keyring, and go back in only in memory before the driver parses the string (`5f6efaa`) | One place per setting and no password in `connections.json`; the driver checks `authMechanism` against the user info | Storing the string as typed |
| 2026-11-03 | The driver gets `ClientOptions` instead of a URL string (`5f6efaa`) | No hand-made percent-encoding, and srv strings work | Fixing the URL building |
| 2026-11-03 | Host options: TLS, CA file, invalid certificates, replica set, server selection timeout (`5f6efaa`, `29aceb2`) | What issue #4 asked for, without client certificates | More fields (read preference, …); those go into a connection string |
| 2026-02-22 | No credentials in logs. The connection URI is logged only in debug builds, with the password masked (`8d6bcac`, PR #32) | The URI carries the password | |
| 2026-02-20 | The edit dialog shows the stored password; it is fetched from the keyring only when the dialog opens (`dc020e7`, issue #29). Partly reverses "credentials never reach the frontend" of PR #23 | Users wanted to see and fix the password they saved | Keeping the password write-only |
| 2026-02-20 | Testing a connection is optional; a connection can be saved untested (`963da88`, issue #26) | The issue gives no reason | Requiring a successful test before saving |
| 2026-02-18 | Credentials stay in Rust. `get_connections` and `get_connection` never return a password, the commands that connect load it from the keyring themselves (`00b3284`, `84839a6`, PR #23, issue #24) | Passwords should not be exposed to the web view, and the grid shouldn't hit the keyring for every connection | Hydrating passwords when listing (the first version) |
| 2026-02-18 | Connections are stored as a versioned JSON object keyed by id: `{ "version": 1, "connections": { … } }` (`84839a6`, issue #24) | O(1) access by id, and a version to migrate the file when its shape changes | A plain array of connections (the first version) |
| 2026-11-03 | keyring 4 with its default `v1` mode: Keychain, Credential Manager, Secret Service. Reverses the keyutils cache of `aad2e90` | All packages were updated. keyring 4 moved its stores into separate crates; `v1` keeps the API. Passwords saved with keyring 3 stay readable (checked) | `keyring-core` with chosen store crates (keyutils plus Secret Service as before), staying on keyring 3.6 |
| 2026-02-18 | On Linux the keyring uses `linux-native-sync-persistent` with `vendored` (`aad2e90`) | keyutils for fast kernel-level access during a session, Secret Service for persistence. `vendored` bundles OpenSSL, which the keyring needs on Linux | `sync-secret-service` alone (the first setting) |
| 2026-02-17 | Passwords are stored in the OS keyring through the `keyring` crate (`eb8fdd9`, issue #15) | Uses the native store on Windows, macOS and Linux | Tauri Stronghold: added on 2026-02-15 (`8b54cd3`) and reverted on 2026-02-17 (`02a2c3a`). The revert says only that Stronghold "is not needed anymore" |
| 2026-02-15 | The password is set on the URL with `Url::set_password`, which percent-encodes it (`8a9638a`, issue #21) | Passwords with special characters failed with "password must be URL encoded" | |
| 2026-02-05 | Only basic authentication (username, password, authentication database) for the MVP (issue #3) | "For MVP, Basic is sufficient" | Other mechanisms for now |

## Running operations

| Date | Decision | Why | Rejected |
|---|---|---|---|
| 2026-11-03 | The connection test skips `serverStatus` when the user lacks the privilege, reads the version from `buildInfo`, marks a failing stage failed, and its events carry a test id from the UI (`2e7d3e1`) | A user who may only read one database failed the test although connecting worked; a failed stage stayed grey; two tests couldn't be told apart | Dropping the status stage |
| 2026-11-03 | `CredentialsUnavailable`, `ConnectionNotFound`, `StorageFailed`; a failed find carries the server's message (`119bf4e`, `5a30f74`) | A keyring failure read as a host problem; a broken query only said "check your permissions" | |
| 2026-11-03 | Logs at `Debug` in development builds, `Info` in release builds (`119bf4e`) | Release builds logged everything at debug | |
| 2026-02-23 | Errors are shown as toasts (vue-sonner) instead of an error box per component (`37819e1`, issue #36) | Cleaner, less duplicated code and styling, fits the theme | The error boxes of before |
| 2026-11-03 | The connection test waits 10 s for a server unless the connection sets a timeout; connecting keeps the driver's 30 s; the waiting stage shows its seconds (`37f0aea`) | A wrong port sat at "Ping Database" for 30 s without a sign of progress | One shorter timeout for connecting too; a fixed timeout overriding the connection's |
| 2026-02-21 | Testing, connecting and finding documents run as background tasks with an id. The UI waits with `await_task_result` and can `cancel_task` (`a14c8fc`, issue #27) | A slow connection froze the UI, and there was no way to give up | A timeout only |
| 2026-02-21 | The connection test streams each stage as a Tauri event (`a14c8fc`) | Each stage shows when it really finishes, not all at once | |
| 2026-02-20 | Every error carries `kind`, the technical `message` and a `user_message`; the UI shows the `user_message` (`b5a9455`) | Raw driver errors were long and unreadable (issue #16) | Showing the raw error |
| 2026-02-09 | One driver per connection id is kept for the life of the process (`1f358d4`) | Reuse the MongoDB client's connection pool across commands. The history records no further reason | |

## Query and data display

| Date | Decision | Why | Rejected |
|---|---|---|---|
| 2026-11-03 | Several connections open at once; disconnecting closes the connection's tabs; each tab belongs to its connection and shows connection and database (`afd84c7`) | Working with two servers is common; tabs used to survive a switch of connection and query the new server | Closing all tabs on switching; keeping tabs per connection while only one is open |
| 2026-11-03 | Every tab has exactly one view, keyed by its own id; `mod+enter` runs only in the active tab (`afd84c7`) | Views multiplied and ran duplicate queries, `Ctrl+Enter` ran everywhere (issue #42) | |
| 2026-11-03 | Paging, the total count, sort and projection in the query tab; paging repeats the executed query (`5a30f74`, `16b6cd7`) | Issue #12 listed them; typing must not change what the next page means | |
| 2026-11-03 | An empty filter is counted with `estimatedDocumentCount`, marked as estimate; a filter exactly (`5a30f74`) | `countDocuments` scans a big collection | Always counting exactly; no count |
| 2026-11-03 | Documents leave Rust as relaxed extended JSON, an `Int64` beyond ±2^53 as `{ "$numberLong" }` (`5a30f74`) | Readable dates in the JSON view, no silent change of large integers | Canonical extended JSON (every number wrapped) |
| 2026-11-03 | The query parser checks every shell function and produces only shapes bson reads as types; regexes as `$regularExpression` (`094ed26`) | Invalid or date-only values silently matched nothing | |
| 2026-03-11 | The table renders only the visible rows (TanStack Virtual), caches formatted cells and sizes columns from a sample (`0aca486`, PR #44, issue #39) | Bigger result sets render fast | |
| 2026-03-02 | Documents are shown in a table by default, with JSON as the second view. The table's column order is saved per connection, database and collection (`ead8967`) | The commit gives no reason for the default | |
| 2026-03-01 | The query input is a single-line CodeMirror editor with MongoDB autocomplete (`9f677e1`) | The history gives no reason | |
| 2026-02-26 | Shell-style queries are parsed by a parser of our own (`src/utils/queryParser`) into extended JSON, which bson turns into BSON in Rust (`0c738f1`) | The history gives no reason. It accepts unquoted keys, single quotes and shell functions like `ObjectId(...)` that JSON doesn't | |
| 2026-02-26 | Open tabs stay mounted (`v-show` and `KeepAlive`) so a tab keeps its query and results (`573dfc0`) | Switching tabs lost the state | `v-if` (the first version) |

## UI

| Date | Decision | Why | Rejected |
|---|---|---|---|
| 2026-11-03 | Forms on TanStack Form with zod 4 and shadcn-vue's `Field` (`3c87afd`) | `@vee-validate/zod` only supports zod 3; vee-validate 5 has been a beta since 2025; shadcn-vue documents TanStack Form | vee-validate 5 beta; an adapter of our own for vee-validate 4; keeping zod 3 |
| 2026-11-03 | A `ConfirmDialog` with `useConfirm()` instead of the browser's `confirm()` (`29aceb2`) | It fits the app and will be needed for deleting data | |
| 2026-11-03 | Shortcuts on `mod` (Cmd on macOS, Ctrl elsewhere); on macOS the default menu without "Close Window" (`95a3e7b`) | Mac users expect Cmd; Cmd+W would otherwise close the app. Not checked on a Mac | |
| 2026-11-03 | The sidebar has one node per open connection with refresh, edit and disconnect; the grid gives way once a connection is connected (`afd84c7`) | Editing must work while connected; the view switches only on success (issue #27) | |
| 2026-02-23 | Roboto is bundled with the app (`226eceb`, issue #30) | The font of the built app looked wrong | The system font |
| 2026-02-18 | The default window is 1200 × 900 (`b35246a`) | The history gives no reason | |

## Process

| Date | Decision | Why | Rejected |
|---|---|---|---|
| 2026-11-03 | CI on GitHub Actions: frontend tests, build, `cargo test` and `cargo clippy` on Linux, macOS and Windows, for pushes to `develop` and `main` and every pull request; the integration tests on Linux only, against a `mongo:8` service container; no installers (`b69ba0b`, `9e6cc5c`) | The app is only developed on Linux; nobody could build it on macOS or Windows. The integration tests only ran when someone remembered them, and service containers exist only on Linux runners | Building installers in CI (no signing yet); integration tests on every platform |
| 2026-11-03 | Tests: vitest in happy-dom with Tauri's `mockIPC`; Rust unit tests with an in-memory credential store and temp directories; integration tests against a real MongoDB marked `#[ignore]` | The rule "every piece of logic gets unit tests"; keyring and MongoDB can't be in the default run | Mounting components in tests (logic moved into modules instead) |
| 2026-11-03 | `docs/` documents features, architecture, decisions and plans, the same way as in smtp-relay. `CLAUDE.md` holds the working rules: `/code-review` before every push, no Claude co-author trailers, unit tests for every piece of logic, lowercase conventional commits | The project is being revived. Its state had to be written down first, and work on it should follow the same rules as the other projects | |
| 2026-11-03 | Claude Code gets a knowledge graph of the code and the docs through graphify. Only the stored doc extraction (`graphify-out/cache/semantic/`) is committed, each clone builds the graph itself | Same setup as smtp-relay: the cache saves the model cost of reading the docs again, and `graph.json` is rewritten whole on every rebuild | Committing `graph.json`, ignoring `graphify-out/` completely |
| 2026-02-05 | Feature branches off `develop`, merged through GitHub pull requests; `develop` goes to `main` from time to time | The history records no reason | |

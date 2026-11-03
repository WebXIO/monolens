# Decision log

Decisions that shape the app, newest first within each area. Each entry says what was decided, why, and what was rejected. Details live in the feature files.

This log was written on 2026-11-03 from the commit history, the pull requests and the GitHub issues. Where they don't record a reason, the entry says so. Add new decisions as they are made.

## Platform and structure

| Date | Decision | Why | Rejected |
|---|---|---|---|
| 2026-02-05 | Database access goes through a `DatabaseDriver` trait with a factory per `ConnectorKind` (`4100b21`) | Room for more drivers or a second MongoDB implementation. The commit only lists the parts | |
| 2026-02-05 | Dependency injection on both sides: Tauri managed state in Rust, a provider registry (`src/domains`) with `useDomain` in Vue (`4100b21`, `aa891e0`) | Services are created once and can be swapped behind an interface (`ConnectionService`, `ConnectionRepository`, `LogStrategy`). The history records no further reason | |
| 2026-02-02 | Tauri 2 with a Rust core and a Vue 3 web view | The README calls Rust and Tauri a "high-performance backend with secure system-level access", Vue and Tailwind "modern, responsive, and maintainable". The goal is a free, European alternative to proprietary tools like Studio 3T | |

## Connections and credentials

| Date | Decision | Why | Rejected |
|---|---|---|---|
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
| 2026-02-23 | Errors are shown as toasts (vue-sonner) instead of an error box per component (`37819e1`, issue #36) | Cleaner, less duplicated code and styling, fits the theme | The error boxes of before |
| 2026-02-21 | Testing, connecting and finding documents run as background tasks with an id. The UI waits with `await_task_result` and can `cancel_task` (`a14c8fc`, issue #27) | A slow connection froze the UI, and there was no way to give up | A timeout only |
| 2026-02-21 | The connection test streams each stage as a Tauri event (`a14c8fc`) | Each stage shows when it really finishes, not all at once | |
| 2026-02-20 | Every error carries `kind`, the technical `message` and a `user_message`; the UI shows the `user_message` (`b5a9455`) | Raw driver errors were long and unreadable (issue #16) | Showing the raw error |
| 2026-02-09 | One driver per connection id is kept for the life of the process (`1f358d4`) | Reuse the MongoDB client's connection pool across commands. The history records no further reason | |

## Query and data display

| Date | Decision | Why | Rejected |
|---|---|---|---|
| 2026-03-11 | The table renders only the visible rows (TanStack Virtual), caches formatted cells and sizes columns from a sample (`0aca486`, PR #44, issue #39) | Bigger result sets render fast | |
| 2026-03-02 | Documents are shown in a table by default, with JSON as the second view. The table's column order is saved per connection, database and collection (`ead8967`) | The commit gives no reason for the default | |
| 2026-03-01 | The query input is a single-line CodeMirror editor with MongoDB autocomplete (`9f677e1`) | The history gives no reason | |
| 2026-02-26 | Shell-style queries are parsed by a parser of our own (`src/utils/queryParser`) into extended JSON, which bson turns into BSON in Rust (`0c738f1`) | The history gives no reason. It accepts unquoted keys, single quotes and shell functions like `ObjectId(...)` that JSON doesn't | |
| 2026-02-26 | Open tabs stay mounted (`v-show` and `KeepAlive`) so a tab keeps its query and results (`573dfc0`) | Switching tabs lost the state | `v-if` (the first version) |

## UI

| Date | Decision | Why | Rejected |
|---|---|---|---|
| 2026-02-23 | Roboto is bundled with the app (`226eceb`, issue #30) | The font of the built app looked wrong | The system font |
| 2026-02-18 | The default window is 1200 × 900 (`b35246a`) | The history gives no reason | |

## Process

| Date | Decision | Why | Rejected |
|---|---|---|---|
| 2026-11-03 | `docs/` documents features, architecture, decisions and plans, the same way as in smtp-relay. `CLAUDE.md` holds the working rules: `/code-review` before every push, no Claude co-author trailers, unit tests for every piece of logic, lowercase conventional commits | The project is being revived. Its state had to be written down first, and work on it should follow the same rules as the other projects | |
| 2026-11-03 | Claude Code gets a knowledge graph of the code and the docs through graphify. Only the stored doc extraction (`graphify-out/cache/semantic/`) is committed, each clone builds the graph itself | Same setup as smtp-relay: the cache saves the model cost of reading the docs again, and `graph.json` is rewritten whole on every rebuild | Committing `graph.json`, ignoring `graphify-out/` completely |
| 2026-02-05 | Feature branches off `develop`, merged through GitHub pull requests; `develop` goes to `main` from time to time | The history records no reason | |

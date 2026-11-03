# Connect and explorer

## Purpose

Connecting opens a saved connection. The sidebar then shows the server's databases and their collections, and a click on a collection opens it in a query tab.

## How it works

**Connecting**

1. The user clicks Connect on a card of the grid, or picks a connection in the header dropdown.
2. `connectionStore.connectTo` sets `isDatabasesLoading`, creates an `AbortController` and calls `ConnectionServiceIPC.connect`. That runs `start_connect` as a task, see [background-tasks.md](background-tasks.md).
3. The Rust task loads the password from the keyring and calls `ConnectionHandler.get_or_connect`:
   - If the pool already has a driver for this connection id, it is returned as it is.
   - Otherwise the factory creates one, `connect` builds the MongoDB `Client`, and the driver goes into the pool.
4. The task lists the database names. The `Client` connects lazily, so this is the first time the server is contacted.
5. On success the store sets `databases` and `activeConnectionId`, and `App.vue` switches from the grid to the sidebar and the tabs. A cancel is only logged. Any other error is a toast.

While connecting, the grid card shows Cancel and the dropdown shows the name being connected, with an X to cancel. All other connect buttons are disabled.

**The explorer** (`AppSidebar`)

- The header row "Explorer" has Refresh and Disconnect.
- Refresh runs `connectTo` again for the active connection. It gets the same pooled driver and lists the databases again.
- A click on a database expands or collapses it. The first expand calls `list_collections` and keeps the names for this session; collapsing and expanding again uses them. An error is only logged.
- A click on a collection opens its query tab, or switches to it if one is open for the same connection, database and collection. See [tabs.md](tabs.md).
- The collection of the active tab is highlighted. When the active tab changes, its database is expanded.
- Disconnect, the button or `Ctrl+Shift+D`, only resets the UI: `activeConnectionId` and `databases` are cleared and the grid shows again. The Rust driver stays in the pool, and the open tabs stay in `tabsStore`.

## Decisions

- **One driver per connection id for the whole process** (`1f358d4`). The MongoDB client keeps its own connection pool, so later commands reuse it.
- **Connecting is a cancellable task** (`a14c8fc`, `571e414`, issue #27). A slow or unreachable server froze the UI before.
- **Connect lists the databases at once**, so the sidebar can be drawn when the switch happens and nothing changes before the connection works (issue #27).

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-09 | The connection pool (`ConnectionHandler`) | `1f358d4` |
| 2026-02-11 | Connect lists the databases, sidebar with Refresh | `01eb47b` |
| 2026-02-11 | Collections of a database (`list_collections`) | `140963a` |
| 2026-02-15 | Disconnect button | `09d7e1f` |
| 2026-02-17 | Disconnect shortcut `Ctrl+Shift+D` | `748c02a` |
| 2026-02-18 | Password loaded only when connecting | `00b3284` |
| 2026-02-21 | Connect as a cancellable task, cancel in the grid and in the dropdown | `a14c8fc`, `571e414` |
| 2026-02-22 | Collection of the active tab highlighted, its database expanded | `fdf801a` |

## Limits and open points

Read from the code on 2026-11-03, not reproduced in the running app:

- **The pool never lets go of a driver.** `ConnectionHandler.disconnect` exists, but no command calls it. A driver is put in the pool before the server was ever contacted, and later calls get it back unchanged. So after a connect with a wrong password, or after editing host, port or password of a connection used in this session, the old client stays until the app restarts. Issue #25 (closed) described the same symptom: "broken forever".
- **Tabs survive Disconnect and a switch of connection.** `TabViewQuery` queries the *active* connection with the tab's database and collection. After connecting to another server, the old tabs show again and run against the new one.
- `list_collections` doesn't load the password. It only works because `start_connect` already pooled the driver.
- Refresh lists the databases again but keeps the cached collection names of expanded databases.
- Errors of `list_collections` are logged, not shown.
- No timeout of our own: an unreachable host takes the driver's default server selection timeout (30 s) unless the user cancels.
- Connecting needs the right to list databases. MongoDB returns only the databases a user may access when it lacks the `listDatabases` privilege. There is no field to type a database name directly.
- Open issues: #14 database statistics (performance, connected users).

## Where the code is

| What | Path |
|---|---|
| Connect command | `src-tauri/src/database/commands/start_connect.rs` |
| Pool | `src-tauri/src/connection_handler/connection_handler.rs` |
| Listing | `list_databases`, `list_collections` in `src-tauri/src/drivers/mongo_official_driver.rs`, `src-tauri/src/database/commands/list_collections.rs` |
| Store | `connectTo`, `cancelConnect`, `disconnect` in `src/stores/connectionStore.ts` |
| UI | `src/components/AppSidebar.vue`, `src/components/connection/ConnectionGrid.vue`, `ConnectionSelector.vue`, `src/App.vue` |

# Connect and explorer

## Purpose

Connecting opens a saved connection; several can be open at once. The sidebar shows each open connection with its databases and collections, and a click on a collection opens it in a query tab.

## How it works

**Connecting**

1. The user clicks Connect on a card of the grid, or picks a connection in the header dropdown "Connections".
2. `connectionStore.connectTo` adds a session for the connection (status `connecting`) and calls `ConnectionServiceIPC.connect(id)`, which runs `start_connect(connectionId)` as a task, see [background-tasks.md](background-tasks.md).
3. In Rust, `ConnectionHandler.open` loads the saved connection and its password from the repository, builds a driver from `ClientOptions` (see [connection-manager.md](connection-manager.md)), puts it in the pool and lists the databases. The client connects lazily, so listing is the first contact with the server. If it fails (wrong password, host down), the driver leaves the pool again.
4. On success the session holds the databases and turns `connected`. The grid gives way to the sidebar and the tabs once the first connection is connected, not while it connects (issue #27). A cancel removes the session; any other error removes it and shows a toast.

**Several connections**

- `connectionStore.sessions` has one entry per open connection: status, databases, the collections of every database opened so far, and whether it is refreshing. Opening an open connection again does nothing.
- The header dropdown lists all saved connections with a dot for open ones and a spinner for connecting ones, and opens the others next to them.

**The explorer** (`AppSidebar`)

- One node per open connection, in the order they were opened, with its name and three buttons: refresh, edit, disconnect. While it connects, a spinner and a cancel button.
- Refresh lists the databases again and forgets the cached collection names.
- Edit opens the connection dialog. After saving, the Rust core has dropped the old driver and the connection reconnects with the new settings; its tabs stay.
- Disconnect, or `Mod+Shift+D` for the active tab's connection, closes the connection's tabs, removes its session and calls `disconnect(connectionId)`, which drops the driver.
- A click on a database expands it. The first expand calls `list_collections(connectionId, database)`, which builds the driver with the password if needed; errors are toasts. The names stay cached until refresh.
- A click on a collection opens its query tab, or switches to the one open for the same connection, database and collection. See [tabs.md](tabs.md).
- The collection of the active tab is highlighted; when the active tab changes, its connection and database open.

**The driver pool** (`ConnectionHandler`)

- One driver per connection id, shared without a lock: the mongodb client runs several commands of one connection at the same time, so a slow query in one tab doesn't hold up another tab or the sidebar. The pool is only locked to look up and insert, not while a driver loads its password and connects.
- A driver leaves the pool on `disconnect`, when its connection is edited or deleted, and when its first command fails. The next command builds it again from the saved settings.
- `with_factory` lets tests use a fake driver.

## Decisions

- **One driver per connection id** (`1f358d4`), reused across commands. **It is dropped when it may be wrong** (`119bf4e`): a driver is created before the server is contacted, so a wrong password or an edited host otherwise stuck until restart (issue #25 described the symptom).
- **Commands take the connection id** (`119bf4e`): Rust loads the saved connection and password itself.
- **Connecting is a cancellable task** (`a14c8fc`, `571e414`, issue #27), and the grid stays until the connection works.
- **Several connections at once** (`afd84c7`): working with two servers, e.g. staging and local, is common.
- **Disconnecting closes the connection's tabs** (`afd84c7`): a tab without its connection has nothing to show; before, tabs survived and queried whichever server was active.

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-09 | The connection pool (`ConnectionHandler`) | `1f358d4` |
| 2026-02-11 | Connect lists the databases, sidebar with Refresh | `01eb47b` |
| 2026-02-11 | Collections of a database (`list_collections`) | `140963a` |
| 2026-02-15 | Disconnect button | `09d7e1f` |
| 2026-02-17 | Disconnect shortcut | `748c02a` |
| 2026-02-18 | Password loaded only when connecting | `00b3284` |
| 2026-02-21 | Connect as a cancellable task, cancel in the grid and in the dropdown | `a14c8fc`, `571e414` |
| 2026-02-22 | Collection of the active tab highlighted, its database expanded | `fdf801a` |
| 2026-11-03 | Commands by connection id, the pool drops drivers, `disconnect` command, errors of `list_collections` shown | `119bf4e` |
| 2026-11-03 | Several connections at once, sidebar node per connection with refresh, edit and disconnect | `afd84c7` |

## Limits and open points

- Connecting needs the right to list databases. Without the `listDatabases` privilege MongoDB returns only the databases the user may access; there is no field to name a database directly.
- A host that doesn't answer waits for the server selection timeout (the connection's option, else 30 s), unless the user cancels.
- Open connections and tabs are not restored after a restart.
- Open issues: #14 database statistics (performance, connected users).

## Where the code is

| What | Path |
|---|---|
| Commands | `src-tauri/src/database/commands/start_connect.rs`, `list_collections.rs`, `disconnect.rs` |
| Pool | `src-tauri/src/connection_handler/connection_handler.rs` |
| Listing | `list_databases`, `list_collections` in `src-tauri/src/drivers/mongo_official_driver.rs` |
| Store | `connectTo`, `cancelConnect`, `refresh`, `disconnect`, `listCollections`, `sessions` in `src/stores/connectionStore.ts` |
| UI | `src/components/AppSidebar.vue`, `src/components/connection/ConnectionGrid.vue`, `ConnectionSelector.vue`, `src/App.vue` |

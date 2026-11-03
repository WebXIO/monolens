# Tabs

## Purpose

Several collections can be open at once, each in its own tab with its own query and results. Closing a tab returns to the one used before.

## How it works

- A tab is `{ id, context: { connection, database, collection }, type }`. `id` is a `crypto.randomUUID()`. `type` is a `TabKind`; only `QUERY` is ever created.
- `tabsStore` keeps the tabs in a `Map`, the `activeTab` id, and `tabHistory`, the ids in the order they were last active.
  - `addTab` creates a tab.
  - `hasContext` finds an open tab of the same type for the same connection id, database and collection. The sidebar uses it so a collection is opened only once.
  - `switchTab` moves the id to the end of the history and makes it active.
  - `deleteTab` removes the tab and its history entry. If it was active, the last tab in the history becomes active.
  - `closeActiveTab` deletes the active tab. `Ctrl+W` is bound to it.
- `TabsContainer` draws the tab strip: the collection name and a close icon per tab, the active one underlined. Below it each tab gets a container that is shown with `v-show` only while its tab is active. Inside it a `KeepAlive` holds the view for the tab kind, keyed by the active tab id.
- The view for every kind is `TabViewQuery`, see [query-tab.md](query-tab.md). It reads its context from `tabsStore.getActiveTab()`, not from a prop.

## Decisions

- **Tabs stay mounted** (`573dfc0`): `v-show` and `KeepAlive` instead of `v-if`, so a tab keeps its query and results.
- **A history instead of "the tab to the left"** (`be509cf`): closing a tab goes back to the one used before.
- **One tab per collection and kind**: the sidebar switches to an open tab instead of adding a second one.

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-17 | Tab store, tab kinds, the sidebar opens tabs | `7717002` |
| 2026-02-17 | Close button | `bd72722` |
| 2026-02-17 | `Ctrl+W` closes the active tab | `748c02a` |
| 2026-02-17 | Tab history | `be509cf` |
| 2026-02-22 | Collection of the active tab highlighted in the sidebar | `fdf801a` |
| 2026-02-24 | `TabViewQuery` as the view of a tab | `e0256f3` |
| 2026-02-26 | Tabs stay mounted | `573dfc0` |

## Limits and open points

Read from the code on 2026-11-03, not reproduced in the running app:

- **The views multiply.** The view in every tab's container is keyed by the *active* tab id, not by its own tab. Each container therefore creates an instance for every tab that becomes active, and every new instance runs its query on mount. With n tabs open, opening or revisiting a tab can run up to n queries. The visible container keeps its state, the hidden copies only cost queries.
- **`Ctrl+Enter` runs the query in every instance** (issue #42), see [shortcuts.md](shortcuts.md#limits-and-open-points).
- **Closing a tab can run a query and unsubscribe the wrong view.** The unmounted views call `CommandRegistry.unsubscribe`, whose `findIndex(subscriber)` calls `execute` as a predicate, see [shortcuts.md](shortcuts.md#limits-and-open-points).
- **Tabs outlive their connection.** Disconnect and switching connections don't close them, and the view queries the active connection with the tab's database and collection names.
- The tab shows only the collection name. Collections with the same name in two databases look the same.
- Tabs are not saved across restarts.
- `INDEXES`, `AGGREGATION`, `EXPORTER`, `IMPORTER` and `STATISTICS` exist as kinds and are all mapped to `TabViewQuery`. Nothing creates them.

## Where the code is

| What | Path |
|---|---|
| Store | `src/stores/tabsStore.ts` |
| Models | `src/domains/tabs/models/`, `src/domains/context/models/context.ts` |
| Strip and containers | `src/components/tabs/TabsContainer.vue` |
| Opening from the sidebar | `handleCollectionSelect` in `src/components/AppSidebar.vue` |

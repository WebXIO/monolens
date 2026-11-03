# Tabs

## Purpose

Several collections can be open at once, also from different connections, each in its own tab with its own query and results. Closing a tab returns to the one used before.

## How it works

- A tab is `{ id, context: { connectionId, database, collection }, type }`. `id` is a `crypto.randomUUID()`. `type` is a `TabKind`; only `QUERY` is created.
- `tabsStore` keeps the tabs in a `Map`, the `activeTab` id, and `tabHistory`, the ids in the order they were last active.
  - `addTab` creates a tab.
  - `hasContext` finds an open tab of the same type for the same connection, database and collection. The sidebar uses it so a collection is opened only once.
  - `switchTab` moves the id to the end of the history and makes it active.
  - `deleteTab` removes the tab and its history entry. If it was active, the last tab in the history becomes active, or none when no tab is left.
  - `closeTabsOf(connectionId)` closes every tab of a connection; disconnecting calls it.
  - `closeActiveTab` deletes the active tab. `Mod+W` is bound to it.
- `TabsContainer` draws the tab strip: the collection name, below it the connection's name and the database, and a close icon; the active one is underlined. Below the strip every tab has exactly one view, shown with `v-show` while its tab is active and kept mounted otherwise, so it keeps its query and results.
- The view gets its `tabId` and reads its own tab, never the active one. The view for every kind is `TabViewQuery`, see [query-tab.md](query-tab.md).

## Decisions

- **Tabs stay mounted** (`573dfc0`): `v-show` instead of `v-if`, so a tab keeps its query and results.
- **A history instead of "the tab to the left"** (`be509cf`): closing a tab goes back to the one used before.
- **One tab per collection and kind**: the sidebar switches to an open tab instead of adding a second one.
- **One view per tab, keyed by its own id** (`afd84c7`). Before, every tab's container rendered a view keyed by the active tab, so views multiplied: each new copy ran its query, and `Ctrl+Enter` ran in all of them (issue #42).
- **A tab belongs to its connection** (`afd84c7`): closing the connection closes its tabs, and the label shows connection and database.

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
| 2026-11-03 | `Mod+W` (Cmd on macOS) | `95a3e7b` |
| 2026-11-03 | Context by connection id, one view per tab, connection and database in the label, `closeTabsOf`, no stale active tab | `afd84c7` |

## Limits and open points

- Tabs are not saved across restarts.
- `INDEXES`, `AGGREGATION`, `EXPORTER`, `IMPORTER` and `STATISTICS` exist as kinds and are all mapped to `TabViewQuery`. Nothing creates them.
- Every open tab keeps its view and results in memory; many tabs with large results cost memory.

## Where the code is

| What | Path |
|---|---|
| Store | `src/stores/tabsStore.ts` (tests in `src/stores/test/tabsStore.test.ts`) |
| Models | `src/domains/tabs/models/`, `src/domains/context/models/context.ts` |
| Strip and views | `src/components/tabs/TabsContainer.vue` |
| Opening from the sidebar | `handleCollectionSelect` in `src/components/AppSidebar.vue` |
